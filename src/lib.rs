use std::collections::VecDeque;
use std::sync::Arc;
use std::time::Duration;

use quanta::{Clock, Instant};

/// The budgeting configuration.
///
/// This determines the window, buckets, and the allowed budget for each project.
#[derive(Debug)]
pub struct BudgetingConfig {
    /// The "backoff" duration within which no flip-flopping of the "exceeded" state happens.
    ///
    /// In other words, a state change will persist for at least this duration before it changes again.
    backoff_duration: Duration,

    /// The total time window for which the budgeting happens.
    budgeting_window: Duration,

    /// The size of each bucket.
    bucket_size: Duration,

    /// Number of buckets to keep track of
    num_buckets: usize,

    /// The total allowed budget before a project is flagged as exceeding it.
    allowed_budget: f64,

    /// The [`Timer`] used to select the proper bucket.
    timer: Timer,
}

impl BudgetingConfig {
    /// Creates a new [`BudgetingConfig`] with the provided configuration.
    pub fn new(
        backoff_duration: Duration,
        budgeting_window: Duration,
        bucket_size: Duration,
        allowed_budget: f64,
    ) -> Self {
        let num_buckets = (budgeting_window.as_secs() / bucket_size.as_secs()) as usize;
        let timer = Timer::new(Clock::new());

        Self {
            backoff_duration,
            budgeting_window,
            bucket_size,
            num_buckets,
            allowed_budget,
            timer,
        }
    }

    /// Overrides the [`Timer`] that is being used by this configuration.
    #[cfg(test)]
    fn with_timer(mut self, timer: Timer) -> Self {
        self.timer = timer;
        self
    }

    /// Returns the current time, truncated to `bucket_size`.
    pub fn truncated_now(&self) -> Instant {
        self.timer.truncated_now(self.bucket_size)
    }
}

/// A [`Timer`] that is mockable and allows us to get a truncated [`Instant`].
#[derive(Debug)]
pub struct Timer {
    /// The [`Clock`] thats being used for this timer.
    clock: Clock,
    /// Whenever this [`Timer`] was constructed.
    start_time: Instant,
}

impl Timer {
    /// Creates a new [`Timer`]
    pub fn new(clock: Clock) -> Self {
        let start_time = clock.recent();
        Self { clock, start_time }
    }

    /// Returns [`Instant::recent()`] truncated to a multiple of the given [`Duration`].
    pub fn truncated_now(&self, duration: Duration) -> Instant {
        let now = self.clock.recent();

        let elapsed = now - self.start_time;
        let duration_secs = duration.as_secs();
        let truncated_offset =
            Duration::from_secs((elapsed.as_secs() / duration_secs) * duration_secs);

        self.start_time + truncated_offset
    }
}

/// Per-project (per-anything, really) budget tracking.
///
/// This allows the recorded budget to be recorded, and allows checking whether
/// the total budget (within the configured time window) has been exceeded.
#[derive(Debug)]
pub struct ProjectStats {
    /// Configuration that governs the budgeting and bucketing.
    config: Arc<BudgetingConfig>,

    /// Whether this project exceeded its budget.
    exceeds_budget: bool,

    /// The deadline after which a projects state can change, to avoid rapid flip-flopping.
    backoff_deadline: Option<Instant>,

    /// The buckets that are used to keep track of the spent budget.
    budget_buckets: VecDeque<(Instant, f64)>,
}

impl ProjectStats {
    /// Create a new per-project tracker based on the given [`BudgetingConfig`].
    pub fn new(config: Arc<BudgetingConfig>) -> Self {
        let budget_buckets = VecDeque::with_capacity(config.num_buckets);
        Self {
            config,
            exceeds_budget: false,
            backoff_deadline: None,
            budget_buckets,
        }
    }

    /// Checks whether this project exceeds its budgets.
    ///
    /// This will also update internal state when checking.
    pub fn exceeds_budget(&mut self) -> bool {
        self.update_aggregated_state(self.config.truncated_now())
    }

    /// Records spent budget.
    ///
    /// This will also update internal state when checking.
    pub fn record_budget_spend(&mut self, spent_budget: f64) -> bool {
        let now = self.config.truncated_now();

        if let Some(latest) = self.budget_buckets.front_mut() {
            if latest.0 >= now {
                latest.1 += spent_budget;
            } else {
                if self.budget_buckets.len() >= self.config.num_buckets {
                    self.budget_buckets.pop_back();
                }
                self.budget_buckets.push_front((now, spent_budget));
            }
        } else {
            self.budget_buckets.push_front((now, spent_budget));
        }

        self.update_aggregated_state(now)
    }

    /// Updates the internal state, calculating whether this project exceeds its budget.
    ///
    /// On state update, this will register a "backoff" timer to avoid rapid flip-flopping.
    fn update_aggregated_state(&mut self, now: Instant) -> bool {
        if let Some(deadline) = self.backoff_deadline {
            if deadline > now {
                return self.exceeds_budget;
            }
            self.backoff_deadline = None;
        }

        let lowest_time = now - self.config.budgeting_window;
        let total_spent_budget: f64 = self
            .budget_buckets
            .iter()
            .filter_map(|b| (b.0 >= lowest_time).then_some(b.1))
            .sum();

        let exceeds_budget = total_spent_budget > self.config.allowed_budget;

        if self.exceeds_budget != exceeds_budget {
            self.exceeds_budget = exceeds_budget;
            self.backoff_deadline = Some(now + self.config.backoff_duration);
        }

        exceeds_budget
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_truncated_time() {
        let (clock, mock) = Clock::mock();
        mock.increment(Duration::from_millis(500));
        let timer = Timer::new(clock);

        let duration = Duration::from_secs(1);
        let now = timer.truncated_now(duration);

        mock.increment(Duration::from_millis(750));

        let still_now = timer.truncated_now(duration);
        assert_eq!(now, still_now);

        mock.increment(Duration::from_millis(750));

        let advanced_now = timer.truncated_now(duration);
        assert!(advanced_now > now);
        assert_eq!(advanced_now.duration_since(now), duration);
    }

    #[test]
    fn test_budgeting() {
        let (clock, mock) = Clock::mock();
        mock.increment(Duration::from_secs(100));

        let config = BudgetingConfig::new(
            Duration::from_secs(10),
            Duration::from_secs(5),
            Duration::from_secs(1),
            100.,
        )
        .with_timer(Timer::new(clock));

        let mut stats = ProjectStats::new(Arc::new(config));

        stats.record_budget_spend(40.);
        let is_blocked = stats.record_budget_spend(10.);
        assert!(!is_blocked);

        mock.increment(Duration::from_millis(1500));

        let is_blocked = stats.record_budget_spend(45.);
        assert!(!is_blocked);

        mock.increment(Duration::from_millis(750));

        let is_blocked = stats.record_budget_spend(10.);
        assert!(is_blocked);

        mock.increment(Duration::from_secs(6));

        // The budgeting window itself is already passed, but we are in backoff
        assert!(stats.exceeds_budget());

        mock.increment(Duration::from_secs(3));

        // The budgeting window itself is already passed, but we are in backoff
        assert!(stats.exceeds_budget());

        mock.increment(Duration::from_secs(2));

        // the backoff deadline has passed, we are unblocked
        assert!(!stats.exceeds_budget());
    }
}

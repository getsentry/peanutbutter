use std::collections::VecDeque;
use std::sync::Arc;
use std::time::Duration;

use quanta::Instant;

use crate::config::BudgetingConfig;

/// Per-project (per-anything, really) budget tracking.
///
/// This allows the recorded budget to be recorded, and allows checking whether
/// the total budget (within the configured time window) has been exceeded.
#[derive(Debug)]
pub struct ProjectStats {
    /// Configuration that governs the budgeting and bucketing.
    config: Arc<BudgetingConfig>,

    /// Whether this project exceeds its budget in the current window.
    exceeds_budget: bool,

    /// The deadline after which a projects state can change, to avoid rapid flip-flopping.
    backoff_deadline: Option<Instant>,

    /// The buckets that are used to keep track of the spent budget.
    budget_buckets: VecDeque<(Instant, f64)>,
}

impl ProjectStats {
    /// Create a new per-project tracker based on the given [`BudgetingConfig`].
    pub fn new(config: Arc<BudgetingConfig>) -> Self {
        // One extra bucket may temporarily exist when spending is recorded.
        let budget_buckets = VecDeque::with_capacity(config.num_buckets + 1);
        Self {
            config,
            exceeds_budget: false,
            backoff_deadline: None,
            budget_buckets,
        }
    }

    /// Checks whether this project exceeds its budgets.
    pub fn exceeds_budget(&mut self) -> bool {
        let now = self.config.now();
        let truncated_now = self.config.truncated_now(now);
        self.check_budget(now, truncated_now)
    }

    /// Records spent budget.
    ///
    /// This will also update internal state when checking.
    pub fn record_spending(&mut self, spent: f64) -> bool {
        let now = self.config.now();
        let truncated_now = self.config.truncated_now(now);

        match self.budget_buckets.front_mut() {
            Some(latest) if latest.0 >= truncated_now => latest.1 += spent,
            _ => self.budget_buckets.push_front((truncated_now, spent)),
        }

        if self.budget_buckets.len() > self.config.num_buckets {
            self.budget_buckets.pop_back();
        }

        self.check_budget(now, truncated_now)
    }

    /// Checks whether all of the buckets are outside the current `budgeting_window`.
    ///
    /// This means that these stats can be cleaned up.
    pub fn is_stale(&self, now: Instant) -> bool {
        let truncated_now = self.config.truncated_now(now);
        if let Some(deadline) = self.backoff_deadline {
            // we are in backoff, so no cleanup should happen
            if deadline > now {
                return false;
            }
        }

        let earliest_time = truncated_now - self.config.budgeting_window;
        self.budget_buckets.iter().all(|b| b.0 < earliest_time)
    }

    /// Checks whether this project exceeds its allotted budget.
    ///
    /// On state update, this will register a "backoff" timer to avoid rapid flip-flopping.
    fn check_budget(&mut self, now: Instant, truncated_now: Instant) -> bool {
        if let Some(deadline) = self.backoff_deadline {
            if deadline > now {
                return self.exceeds_budget;
            }
            self.backoff_deadline = None;
        }

        let spent_budget = self.spent_budget(now, truncated_now);

        let exceeds_budget = spent_budget > self.config.budget;

        if self.exceeds_budget != exceeds_budget {
            self.exceeds_budget = exceeds_budget;
            self.backoff_deadline = Some(now + self.config.backoff_duration);
        }

        exceeds_budget
    }

    /// Returns the spent budget, averaged *per-second*.
    fn spent_budget(&self, now: Instant, truncated_now: Instant) -> f64 {
        let earliest_time = truncated_now - self.config.budgeting_window;
        let total_spent_budget: f64 = self
            .budget_buckets
            .iter()
            .filter_map(|b| (b.0 >= earliest_time).then_some(b.1))
            .sum();

        // The configured budget is meant as a per-second budget.
        // To calculate that, we want to divide by the real passed time,
        // to avoid any artifacts resulting from the bucketing as much as possible.
        let adjustment = now - truncated_now;
        let adjusted_time_window = if adjustment == Duration::ZERO {
            // If `adjustment` is `0`, the `budgeting_window` is already exactly correct.
            self.config.budgeting_window
        } else {
            // If `adjustment` is not `0`, we have started a new, incomplete bucket.
            // We subtract that bucket's size and add the adjustment instead.
            self.config.budgeting_window - self.config.bucket_size + adjustment
        };

        total_spent_budget / adjusted_time_window.as_secs_f64()
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use quanta::Clock;

    use crate::config::Timer;

    use super::*;

    #[test]
    fn test_budgeting() {
        let (clock, mock) = Clock::mock();
        mock.increment(Duration::from_secs(100));
        let timer = Timer::new(clock);

        let config = BudgetingConfig::new(
            Duration::from_secs(10),
            Duration::from_secs(5),
            Duration::from_secs(1),
            20.,
        )
        .with_timer(timer.clone());

        let mut stats = ProjectStats::new(Arc::new(config));

        stats.record_spending(40.);
        let is_blocked = stats.record_spending(10.);
        assert!(!is_blocked);

        mock.increment(Duration::from_millis(1500));

        let is_blocked = stats.record_spending(45.);
        assert!(!is_blocked);

        mock.increment(Duration::from_millis(750));

        let is_blocked = stats.record_spending(20.);
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

        // after *another* backoff, these stats are stale
        mock.increment(Duration::from_secs(10));
        assert!(stats.is_stale(timer.now()));
    }

    #[test]
    fn test_adjusted_budget() {
        let (clock, mock) = Clock::mock();
        mock.increment(Duration::from_secs(100));
        let timer = Timer::new(clock);

        let config = BudgetingConfig::new(
            Duration::from_secs(10),
            Duration::from_secs(5),
            Duration::from_secs(1),
            200.,
        )
        .with_timer(timer.clone());

        let mut stats = ProjectStats::new(Arc::new(config));

        // we spend `10` every `100ms`
        for _ in 0..(5 * 10) {
            stats.record_spending(10.);
            mock.increment(Duration::from_millis(100));
        }

        for _ in 0..(5 * 10) {
            let now = timer.now();
            let truncated_now = stats.config.truncated_now(now);
            let spent_budget = stats.spent_budget(now, truncated_now);
            // FIXME:
            // Ideally this should round to 100 all the time,
            // but there still seems to be an edgecase in here somewhere:
            // This rounds to 100 for 9/10 cases, but ~125 in 1/10.
            dbg!(spent_budget);

            stats.record_spending(10.);
            mock.increment(Duration::from_millis(100));
        }
    }
}

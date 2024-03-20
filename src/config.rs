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
    pub backoff_duration: Duration,

    /// Length of the sliding budgeting window.
    pub budgeting_window: Duration,

    /// The size of the buckets that spent budget is sorted into.
    pub bucket_size: Duration,

    /// The budget assigned to each project.
    pub budget: f64,

    /// The number of time buckets to keep track of.
    ///
    /// This should be at least ⌈budgeting_window/buckt_size⌉.
    pub(crate) num_buckets: usize,

    /// The [`Timer`] used to select the proper bucket.
    timer: Timer,
}

impl BudgetingConfig {
    /// Creates a new [`BudgetingConfig`] with the provided configuration.
    pub fn new(
        backoff_duration: Duration,
        budgeting_window: Duration,
        bucket_size: Duration,
        budget: f64,
    ) -> Self {
        // Note: this is only correct if bucket_size divides budgeting_window
        let num_buckets = (budgeting_window.as_micros() / bucket_size.as_micros()) as usize;
        let timer = Timer::new(Clock::new());

        Self {
            backoff_duration,
            budgeting_window,
            bucket_size,
            num_buckets,
            budget,
            timer,
        }
    }

    /// Overrides the [`Timer`] that is being used by this configuration.
    pub(crate) fn with_timer(mut self, timer: Timer) -> Self {
        self.timer = timer;
        self
    }

    /// Returns a [`Instant::recent()`] which can be further truncated.
    pub(crate) fn now(&self) -> Instant {
        self.timer.now()
    }

    /// Returns the recent [`Instant`], as well as one truncated to `bucket_size`.
    pub(crate) fn truncated_now(&self, now: Instant) -> Instant {
        self.timer.truncated(now, self.bucket_size)
    }
}

/// A [`Timer`] that is mockable and allows us to get a truncated [`Instant`].
#[derive(Clone, Debug)]
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

    /// Returns a [`Instant::recent()`] which can be further truncated.
    pub fn now(&self) -> Instant {
        self.clock.recent()
    }

    /// Returns the `now` truncated to a multiple of the given [`Duration`].
    pub fn truncated(&self, now: Instant, duration: Duration) -> Instant {
        let elapsed = now - self.start_time;
        let duration_secs = duration.as_micros() as u64;
        let truncated_offset =
            Duration::from_micros((elapsed.as_micros() as u64 / duration_secs) * duration_secs);

        self.start_time + truncated_offset
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
        let now = timer.truncated(timer.now(), duration);

        mock.increment(Duration::from_millis(750));

        let still_now = timer.truncated(timer.now(), duration);
        assert_eq!(now, still_now);

        mock.increment(Duration::from_millis(750));

        let advanced_now = timer.truncated(timer.now(), duration);
        assert!(advanced_now > now);
        assert_eq!(advanced_now.duration_since(now), duration);
    }
}

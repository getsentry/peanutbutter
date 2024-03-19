mod config;
mod stats;

use std::sync::Arc;
use std::thread::JoinHandle;
use std::time::Duration;

pub use config::BudgetingConfig;
use config::Timer;
use dashmap::mapref::entry::Entry;
use dashmap::mapref::one::RefMut;
use dashmap::DashMap;
use indexmap::IndexMap;
use quanta::Clock;
pub use stats::ProjectStats;

type ProjectBudgets = Arc<DashMap<(usize, u64), ProjectStats>>;
type ProjectRef<'a> = RefMut<'a, (usize, u64), ProjectStats>;

#[derive(Debug)]
pub struct Service {
    /// The global [`Timer`] used within all the [`BudgetingConfig`]s.
    ///
    /// The timers clock will be updated regularly (for proper [`Clock::recent`] access).
    timer: Timer,

    /// A map of known configurations.
    ///
    /// This is a [`IndexMap`] as an optimization, so we do not need to constantly
    /// [`Arc::clone`] the [`BudgetingConfig`] to index into the main budget map.
    configs: IndexMap<String, Arc<BudgetingConfig>>,

    /// A concurrent [`DashMap`] containing all the project stats/budgets.
    project_budgets: ProjectBudgets,

    /// The background thread that updates the [`Timer`] and cleans up stale stats.
    // TODO: actually implement graceful shutdown
    #[allow(unused)]
    maintenance_thread: JoinHandle<()>,
}

impl Service {
    /// Creates a new (empty) Service
    pub fn new() -> Self {
        let clock = Clock::new();
        quanta::set_recent(clock.now());
        let timer = Timer::new(clock.clone());
        let project_budgets = ProjectBudgets::default();

        let maintenance_thread = std::thread::spawn({
            let project_budgets = project_budgets.clone();
            move || service_maintenance(clock, project_budgets)
        });

        Self {
            timer,
            configs: Default::default(),
            project_budgets,
            maintenance_thread,
        }
    }

    /// Add/register a new [`BudgetingConfig`] with a specific name.
    ///
    /// This function will `panic` when a duplicated config is provided.
    /// The intention is to only add configuration once on startup,
    /// and `panic`-ing in that situation is considered acceptable.
    pub fn add_config(&mut self, name: &str, config: BudgetingConfig) {
        let config = Arc::new(config.with_timer(self.timer.clone()));
        let previous = self.configs.insert(name.into(), config);
        assert!(previous.is_none());
    }

    /// Checks whether this project exceeds its budgets.
    ///
    /// A project that is not (yet) known will always return `false`,
    /// meaning it does not exceed the budget.
    pub fn exceeds_budget(&self, config: &str, project_id: u64) -> bool {
        if let Some(mut stats) = self.get_project_stats(config, project_id, false) {
            stats.exceeds_budget()
        } else {
            false
        }
    }

    /// Records spent budget.
    pub fn record_spending(&self, config: &str, project_id: u64, spent: f64) -> bool {
        if let Some(mut stats) = self.get_project_stats(config, project_id, true) {
            stats.record_spending(spent)
        } else {
            false
        }
    }

    /// Gets a mutable [`ProjectStats`] reference from the concurrent [`DashMap`].
    fn get_project_stats(
        &self,
        config: &str,
        project_id: u64,
        or_insert: bool,
    ) -> Option<ProjectRef> {
        let (config_idx, _name, config) = self.configs.get_full(config)?;
        let key = (config_idx, project_id);

        match self.project_budgets.entry(key) {
            Entry::Occupied(e) => Some(e.into_ref()),
            Entry::Vacant(e) if or_insert => Some(e.insert(ProjectStats::new(config.clone()))),
            _ => None,
        }
    }
}

impl Default for Service {
    fn default() -> Self {
        Self::new()
    }
}

/// A background maintenance task that periodically updates the [`Clock`],
/// and cleans up state [`ProjectStats`].
fn service_maintenance(timer: Clock, project_budgets: ProjectBudgets) {
    // We scan the map, and clean up stale entries in two phases.
    // The [`DashMap`] docs specifically mention that certain operations can deadlock,
    // such as iterating and calling `remove_if` at the same time.
    let mut keys_needing_cleanup = vec![];

    loop {
        std::thread::sleep(Duration::from_millis(500));
        let now = timer.now();
        quanta::set_recent(now);

        for entry in project_budgets.iter() {
            if entry.value().is_stale(now) {
                keys_needing_cleanup.push(*entry.key());
            }
        }

        for key in keys_needing_cleanup.drain(..) {
            project_budgets.remove_if(&key, |_k, stats| stats.is_stale(now));
        }
    }
}

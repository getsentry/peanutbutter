use std::sync::atomic::AtomicU64;
use std::time::Duration;

use divan::{counter, Bencher};
use rand::{Rng, SeedableRng};

use peanutbutter::*;

fn main() {
    divan::main();
}

#[divan::bench(min_time = 0.5, threads = [0, 1, 4], args = [1 << 10, 1 << 15, 1 << 20])]
fn fibonacci(bencher: Bencher, projects: u64) {
    let allowed_budget = 1_000.;
    let mut service = Service::new();
    service.add_config(
        "test",
        BudgetingConfig::new(
            Duration::from_millis(10),
            Duration::from_millis(5),
            Duration::from_micros(500),
            allowed_budget,
        ),
    );

    let seed = AtomicU64::new(0);
    let num_ops: u32 = 10_000;

    bencher
        .counter(counter::ItemsCount::new(num_ops))
        .bench(move || {
            let mut rng = rand::rngs::SmallRng::seed_from_u64(
                seed.fetch_add(1, std::sync::atomic::Ordering::Relaxed),
            );
            for _ in 0..num_ops {
                if rng.gen() {
                    service.record_budget_spend(
                        "test",
                        rng.gen_range(0..projects),
                        rng.gen_range(0.0..allowed_budget),
                    );
                } else {
                    service.exceeds_budget("test", rng.gen_range(0..projects));
                }
            }
        });
}

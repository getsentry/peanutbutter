use std::time::Duration;

use divan::Bencher;
use rand::{Rng, SeedableRng};

use peanutbutter::*;

fn main() {
    divan::main();
}

#[divan::bench(threads, args = [1 << 10, 1 << 15, 1 << 20])]
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

    let rng = rand::rngs::SmallRng::seed_from_u64(0);

    bencher.bench(move || {
        let mut rng = rng.clone();
        service.record_budget_spend(
            "test",
            rng.gen_range(0..projects),
            rng.gen_range(0.0..allowed_budget),
        );
        service.exceeds_budget("test", rng.gen_range(0..projects))
    });
}

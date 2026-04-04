use crate::financial_model::config::Config;
use crate::financial_model::runner::{run_headless, ExecutionMode};
use crate::financial_model::StrategyRunSummary;
use std::cmp::Ordering;

const REDUCED_CONFIG_PATH: &str = "tests/fixtures/config_reduced_seeded.json";

fn score(run: &StrategyRunSummary) -> f32 {
    let bankruptcy_penalty = run.bankruptcy_rate * 100_000.0;
    let high_risk_penalty = (run.p90_net_worth - run.p10_net_worth).max(0.0) * 0.1;
    run.median_net_worth - bankruptcy_penalty - high_risk_penalty
}

fn best_run(runs: &[StrategyRunSummary]) -> StrategyRunSummary {
    runs.iter()
        .max_by(|a, b| score(a).partial_cmp(&score(b)).unwrap_or(Ordering::Equal))
        .expect("No strategy runs")
        .clone()
}

fn assert_close(label: &str, expected: f32, actual: f32, rel_tol: f32, abs_tol: f32) {
    let diff = (expected - actual).abs();
    let bound = abs_tol + rel_tol * expected.abs().max(actual.abs());
    assert!(
        diff <= bound,
        "{} mismatch: expected {:.6}, got {:.6}, diff {:.6}, bound {:.6}",
        label,
        expected,
        actual,
        diff,
        bound
    );
}

#[test]
fn financial_headless_serial_multithreaded_parity() {
    let mut config = Config::read_from(REDUCED_CONFIG_PATH);
    config.simulation.thread_count = Some(4);

    let serial_summaries = run_headless(&config, ExecutionMode::Serial);
    let mt_summaries = run_headless(&config, ExecutionMode::Multithreaded);

    assert_eq!(
        serial_summaries.len(),
        mt_summaries.len(),
        "Strategy count mismatch between serial and multithreaded"
    );

    let serial_runs: Vec<StrategyRunSummary> = serial_summaries
        .iter()
        .map(StrategyRunSummary::from_financial_summary)
        .collect();
    let mt_runs: Vec<StrategyRunSummary> = mt_summaries
        .iter()
        .map(StrategyRunSummary::from_financial_summary)
        .collect();

    let serial_best = best_run(&serial_runs);
    let mt_best = best_run(&mt_runs);

    assert_eq!(
        serial_best.strategy_desc, mt_best.strategy_desc,
        "Best strategy mismatch between serial and multithreaded"
    );

    assert_close(
        "median_net_worth",
        serial_best.median_net_worth,
        mt_best.median_net_worth,
        1e-4,
        1e-2,
    );
    assert_close(
        "p10_net_worth",
        serial_best.p10_net_worth,
        mt_best.p10_net_worth,
        1e-4,
        1e-2,
    );
    assert_close(
        "p90_net_worth",
        serial_best.p90_net_worth,
        mt_best.p90_net_worth,
        1e-4,
        1e-2,
    );

    let serial_bankrupt =
        (serial_best.bankruptcy_rate * config.simulation.num_agents as f32).round() as u32;
    let mt_bankrupt =
        (mt_best.bankruptcy_rate * config.simulation.num_agents as f32).round() as u32;
    assert_eq!(
        serial_bankrupt, mt_bankrupt,
        "Bankruptcy count mismatch between serial and multithreaded"
    );
}

use crate::financial_model::config::Config;
use crate::financial_model::runner::{run_headless, ExecutionMode};
use std::path::Path;

const FIXTURE_PATH: &str = "tests/fixtures/serial_baseline_summary.json";
const FIXED_SEED: u64 = 42;

#[derive(serde::Deserialize, serde::Serialize)]
struct BaselineFixture {
    strategy_desc: String,
    median_net_worth: f32,
    p10_net_worth: f32,
    p90_net_worth: f32,
    bankruptcy_count: u32,
    successful_retirement_count: u32,
    seed: u64,
}

fn write_fixture(path: &str, best: &crate::financial_model::FinancialSummary, seed: u64) {
    let generated = BaselineFixture {
        strategy_desc: best.strategy_desc.clone(),
        median_net_worth: best.median_net_worth,
        p10_net_worth: best.p10_net_worth,
        p90_net_worth: best.p90_net_worth,
        bankruptcy_count: best.bankruptcy_count,
        successful_retirement_count: best.successful_retirement_count,
        seed,
    };
    let payload = serde_json::to_string_pretty(&generated)
        .expect("Failed to serialize generated baseline fixture");
    std::fs::write(path, payload).expect("Failed to write generated baseline fixture");
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
fn serial_baseline_matches_financial_fixture() {
    let mut config = Config::read_from("tests/fixtures/config_reduced_seeded.json");
    config.simulation.seed = Some(FIXED_SEED);

    let summaries = run_headless(&config, ExecutionMode::Serial);
    let best = summaries
        .iter()
        .max_by(|a, b| {
            a.median_net_worth
                .partial_cmp(&b.median_net_worth)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .expect("No strategy summaries produced");

    if !Path::new(FIXTURE_PATH).exists() {
        write_fixture(FIXTURE_PATH, best, FIXED_SEED);
        panic!(
            "Baseline fixture was missing and has been created at {}. Re-run tests.",
            FIXTURE_PATH
        );
    }

    let fixture_raw =
        std::fs::read_to_string(FIXTURE_PATH).expect("Failed to read baseline fixture JSON");
    let fixture: BaselineFixture =
        serde_json::from_str(&fixture_raw).expect("Invalid baseline fixture JSON");

    assert_eq!(fixture.seed, FIXED_SEED, "Fixture seed metadata drifted");

    if std::env::var("REGENERATE_BASELINE_FIXTURE").ok().as_deref() == Some("1") {
        write_fixture(FIXTURE_PATH, best, FIXED_SEED);
        panic!(
            "Baseline fixture regenerated at {}. Re-run tests.",
            FIXTURE_PATH
        );
    }

    assert_eq!(
        best.strategy_desc, fixture.strategy_desc,
        "Best strategy drifted"
    );
    assert_eq!(
        best.bankruptcy_count, fixture.bankruptcy_count,
        "bankruptcy_count mismatch"
    );
    assert_eq!(
        best.successful_retirement_count, fixture.successful_retirement_count,
        "successful_retirement_count mismatch"
    );

    assert_close(
        "median_net_worth",
        fixture.median_net_worth,
        best.median_net_worth,
        1e-4,
        1e-2,
    );
    assert_close(
        "p10_net_worth",
        fixture.p10_net_worth,
        best.p10_net_worth,
        1e-4,
        1e-2,
    );
    assert_close(
        "p90_net_worth",
        fixture.p90_net_worth,
        best.p90_net_worth,
        1e-4,
        1e-2,
    );
}

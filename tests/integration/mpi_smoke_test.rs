use crate::financial_model::config::Config;
use crate::financial_model::runner::{run_headless, ExecutionMode};
use crate::financial_model::StrategyRunSummary;
use std::cmp::Ordering;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

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

fn extract_summary_path(stdout: &str) -> Option<String> {
    stdout
        .lines()
        .find_map(|line| line.trim().strip_prefix("- summary: "))
        .map(|s| s.trim().to_string())
}

fn map_container_to_host_path(container_path: &str) -> PathBuf {
    let container_prefix = "/home/mpiuser/workdir/";
    if let Some(rel) = container_path.strip_prefix(container_prefix) {
        Path::new(rel).to_path_buf()
    } else {
        PathBuf::from(container_path)
    }
}

#[test]
fn mpi_smoke_via_docker_script() {
    if std::env::var("RUN_MPI_DOCKER_TESTS").ok().as_deref() != Some("1") {
        return;
    }

    let serial_config = Config::read_from(REDUCED_CONFIG_PATH);
    let serial_summaries = run_headless(&serial_config, ExecutionMode::Serial);
    let serial_runs: Vec<StrategyRunSummary> = serial_summaries
        .iter()
        .map(StrategyRunSummary::from_financial_summary)
        .collect();
    let serial_best = best_run(&serial_runs);

    let output = Command::new("bash")
        .arg("run_mpi_docker.sh")
        .env("KRAB_CONFIG_PATH", REDUCED_CONFIG_PATH)
        .env(
            "KRAB_OUTPUT_DIR",
            "/home/mpiuser/workdir/output/mpi_parity_test",
        )
        .env("KRAB_MPI_NP", "2")
        .output()
        .expect("Failed to execute run_mpi_docker.sh");

    assert!(
        output.status.success(),
        "MPI Docker smoke run failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("MPI headless run artifacts:"),
        "Expected MPI artifacts output in stdout"
    );
    assert!(
        stdout.contains("Best strategy:"),
        "Expected best strategy output in stdout"
    );

    let summary_path =
        extract_summary_path(&stdout).expect("Could not locate summary path in MPI output");
    let summary_host_path = map_container_to_host_path(&summary_path);
    let summary_json = fs::read_to_string(&summary_host_path).unwrap_or_else(|_| {
        panic!(
            "Unable to read MPI summary at {}",
            summary_host_path.display()
        )
    });

    let mpi_runs: Vec<StrategyRunSummary> =
        serde_json::from_str(&summary_json).expect("Invalid MPI summary JSON format");
    let mpi_best = best_run(&mpi_runs);

    assert_eq!(
        serial_best.strategy_desc, mpi_best.strategy_desc,
        "Best strategy mismatch between serial and MPI"
    );
    assert_close(
        "median_net_worth",
        serial_best.median_net_worth,
        mpi_best.median_net_worth,
        1e-4,
        1e-2,
    );
    assert_close(
        "p10_net_worth",
        serial_best.p10_net_worth,
        mpi_best.p10_net_worth,
        1e-4,
        1e-2,
    );
    assert_close(
        "p90_net_worth",
        serial_best.p90_net_worth,
        mpi_best.p90_net_worth,
        1e-4,
        1e-2,
    );

    let serial_bankrupt =
        (serial_best.bankruptcy_rate * serial_config.simulation.num_agents as f32).round() as u32;
    let mpi_bankrupt =
        (mpi_best.bankruptcy_rate * serial_config.simulation.num_agents as f32).round() as u32;
    assert_eq!(
        serial_bankrupt, mpi_bankrupt,
        "Bankruptcy count mismatch between serial and MPI"
    );
}

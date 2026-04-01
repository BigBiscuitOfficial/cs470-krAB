mod financial_model;

#[cfg(feature = "parallel")]
use financial_model::config::Config;
#[cfg(feature = "parallel")]
use financial_model::report::write_sweep_artifacts;
#[cfg(feature = "parallel")]
use financial_model::runner::{run_headless, ExecutionMode};
#[cfg(feature = "parallel")]
use financial_model::StrategyRunSummary;

#[cfg(feature = "parallel")]
fn main() {
    let config = Config::read_from("examples/config_comprehensive.json");
    let summaries = run_headless(&config, ExecutionMode::Multithreaded);

    // Convert to StrategyRunSummary for reporting
    let runs: Vec<StrategyRunSummary> = summaries
        .iter()
        .map(StrategyRunSummary::from_financial_summary)
        .collect();

    // Find best strategy (highest median net worth)
    let best = runs
        .iter()
        .max_by(|a, b| {
            a.median_net_worth
                .partial_cmp(&b.median_net_worth)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .expect("No strategies ran");

    let artifacts = write_sweep_artifacts(&config, "multithreaded", &runs, best);

    println!("\nHeadless run artifacts:");
    println!("- run dir: {}", artifacts.run_dir);
    println!("- report: {}", artifacts.report_html);
    println!("- summary: {}", artifacts.summary_json);
    println!(
        "- sweep results: {}",
        artifacts
            .sweep_results_csv
            .unwrap_or_else(|| "N/A".to_string())
    );
    println!("\nBest strategy: {}", best.strategy_desc);
    println!("  Median net worth: ${:.0}", best.median_net_worth);
    println!(
        "  P10-P90 range: ${:.0} - ${:.0}",
        best.p10_net_worth, best.p90_net_worth
    );
}

#[cfg(not(feature = "parallel"))]
fn main() {
    println!("Please enable the 'parallel' feature to run this example.");
}

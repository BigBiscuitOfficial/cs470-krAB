use chrono::Utc;
use serde::Serialize;
use serde_json;
use std::fs::{self, File};
use std::io::Write;

use super::config::Config;
use super::report_html::generate_interactive_html;
use super::FinancialSummary;
use super::StrategyRunSummary;

#[derive(Debug, Clone, Serialize)]
pub struct ArtifactPaths {
    pub run_dir: String,
    pub summary_json: String,
    pub timeseries_csv: String,
    pub advice_txt: String,
    pub report_html: String,
    pub sweep_results_csv: Option<String>,
}

fn create_run_dir(base_dir: &str, mode: &str) -> String {
    let timestamp = Utc::now().format("%Y%m%d_%H%M%S_%3f").to_string();
    let path = format!("{}/financial_{}_{}", base_dir, mode, timestamp);
    fs::create_dir_all(&path).expect("Failed to create output directory");
    path
}

fn score_run(run: &StrategyRunSummary) -> f32 {
    // Higher is better: prioritize high median, low bankruptcy, successful retirement
    let base_score = run.median_net_worth;
    let bankruptcy_penalty = run.bankruptcy_rate * 500_000.0;
    let retirement_bonus = run.successful_retirement_rate * 200_000.0;
    base_score - bankruptcy_penalty + retirement_bonus
}

pub fn build_advice(best: &StrategyRunSummary) -> String {
    format!(
        "Best Path Forward - Comprehensive Life Strategy\n\
         ================================================\n\n\
         Recommended Strategy:\n\
         {}\n\n\
         Expected Financial Outcomes:\n\
         - Median Net Worth: ${:.0}\n\
         - 10th Percentile (Worst Case): ${:.0}\n\
         - 90th Percentile (Best Case): ${:.0}\n\
         - Bankruptcy Rate: {:.1}%\n\
         - Successful Retirement Rate: {:.1}%\n\n\
         Final Account Composition (Average):\n\
         - Liquid Cash: ${:.0}\n\
         - 401(k): ${:.0}\n\
         - Home Equity: ${:.0}\n\
         - Total Debt: ${:.0}\n\n\
         Simulation Runtime: {:.3}s\n",
        best.strategy_desc,
        best.median_net_worth,
        best.p10_net_worth,
        best.p90_net_worth,
        best.bankruptcy_rate * 100.0,
        best.successful_retirement_rate * 100.0,
        best.avg_liquid_cash,
        best.avg_401k,
        best.avg_home_equity,
        best.avg_total_debt,
        best.run_duration,
    )
}

fn write_sweep_csv(path: &str, runs: &[StrategyRunSummary]) {
    let mut f = File::create(path).expect("Unable to create sweep csv");
    writeln!(
        f,
        "strategy,median_net_worth,p10_net_worth,p90_net_worth,bankruptcy_rate,retirement_success_rate,avg_cash,avg_401k,avg_home_equity,avg_debt,score"
    )
    .expect("write failed");
    for run in runs {
        writeln!(
            f,
            "\"{}\",{:.2},{:.2},{:.2},{:.4},{:.4},{:.2},{:.2},{:.2},{:.2},{:.2}",
            run.strategy_desc,
            run.median_net_worth,
            run.p10_net_worth,
            run.p90_net_worth,
            run.bankruptcy_rate,
            run.successful_retirement_rate,
            run.avg_liquid_cash,
            run.avg_401k,
            run.avg_home_equity,
            run.avg_total_debt,
            score_run(run),
        )
        .expect("write failed");
    }
}

pub fn write_sweep_artifacts(
    config: &Config,
    mode: &str,
    runs: &[StrategyRunSummary],
    best: &StrategyRunSummary,
) -> ArtifactPaths {
    let output_base = config.output_base_dir();
    let run_dir = create_run_dir(&output_base, mode);
    let summary_json = format!("{}/summary.json", run_dir);
    let timeseries_csv = format!("{}/timeseries.csv", run_dir);
    let advice_txt = format!("{}/advice.txt", run_dir);
    let report_html = format!("{}/report.html", run_dir);
    let sweep_results_csv = format!("{}/sweep_results.csv", run_dir);

    let advice = build_advice(best);
    let json = serde_json::to_string_pretty(runs).expect("json serialization failed");
    fs::write(&summary_json, json).expect("Unable to write summary.json");
    fs::write(
        &timeseries_csv,
        "Timeseries data is not available for sweep runs. Run a single strategy to see time-series data.\n",
    )
    .expect("Unable to write timeseries placeholder");
    fs::write(&advice_txt, advice.clone()).expect("Unable to write advice.txt");
    write_sweep_csv(&sweep_results_csv, runs);

    let html = generate_interactive_html(&advice, None, runs);
    fs::write(&report_html, html).expect("Unable to write report.html");

    ArtifactPaths {
        run_dir,
        summary_json,
        timeseries_csv,
        advice_txt,
        report_html,
        sweep_results_csv: Some(sweep_results_csv),
    }
}

pub fn write_and_print_headless_sweep(
    config: &Config,
    mode: &str,
    summaries: &[FinancialSummary],
) -> ArtifactPaths {
    let runs: Vec<StrategyRunSummary> = summaries
        .iter()
        .map(StrategyRunSummary::from_financial_summary)
        .collect();

    let best = runs
        .iter()
        .max_by(|a, b| {
            a.median_net_worth
                .partial_cmp(&b.median_net_worth)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .expect("No strategies ran");

    let artifacts = write_sweep_artifacts(config, mode, &runs, best);

    println!("\nHeadless run artifacts:");
    println!("- run dir: {}", artifacts.run_dir);
    println!("- report: {}", artifacts.report_html);
    println!("- summary: {}", artifacts.summary_json);
    println!(
        "- sweep results: {}",
        artifacts
            .sweep_results_csv
            .clone()
            .unwrap_or_else(|| "N/A".to_string())
    );
    println!("\nBest strategy: {}", best.strategy_desc);
    println!("  Median net worth: ${:.0}", best.median_net_worth);
    println!(
        "  P10-P90 range: ${:.0} - ${:.0}",
        best.p10_net_worth, best.p90_net_worth
    );

    artifacts
}

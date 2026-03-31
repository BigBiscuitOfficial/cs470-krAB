use chrono::Utc;
use serde::Serialize;
use std::cmp::Ordering;
use std::fs::{self, File};
use std::io::Write;

use super::config::Config;
use super::{FinancialSummary, StrategyRunSummary};

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
    let timestamp = Utc::now().format("%Y%m%d_%H%M%S").to_string();
    let path = format!("{}/financial_{}_{}", base_dir, mode, timestamp);
    fs::create_dir_all(&path).expect("Failed to create output directory");
    path
}

fn score_run(run: &StrategyRunSummary) -> f32 {
    let bankruptcy_penalty = run.bankruptcy_count as f32 * 50_000.0;
    let gini_penalty = run.gini_coefficient * 100_000.0;
    run.median_wealth - bankruptcy_penalty - gini_penalty
}

pub fn build_advice(best: &StrategyRunSummary, total_agents: u32) -> String {
    format!(
        "Best Path Forward\n\
         ----------------\n\
         Recommended strategy:\n\
         - Save {:.0}% of income each period\n\
         - Use risk profile {:.2}\n\
         - Keep an emergency fund of ${:.2}\n\n\
         Expected outcomes:\n\
         - Median wealth: ${:.2}\n\
         - Average wealth: ${:.2}\n\
         - Bankruptcy count: {} of {}\n\
         - Gini coefficient: {:.3}\n\n\
         Timing breakdown:\n\
         - Init: {:.4}s\n\
         - Step compute: {:.4}s\n\
         - Metrics: {:.4}s\n\
         - Communication/overhead: {:.4}s\n",
        best.savings_rate * 100.0,
        best.risk_profile,
        best.emergency_fund,
        best.median_wealth,
        best.average_wealth,
        best.bankruptcy_count,
        total_agents,
        best.gini_coefficient,
        best.init_time,
        best.step_compute_time,
        best.metrics_calc_time,
        best.communication_overhead.max(0.0),
    )
}

fn write_timeseries_csv(path: &str, summary: &FinancialSummary) {
    let mut f = File::create(path).expect("Unable to create timeseries csv");
    writeln!(f, "step,average_wealth,median_wealth,bankruptcy_count").expect("write failed");
    for point in &summary.timeseries {
        writeln!(
            f,
            "{},{:.6},{:.6},{}",
            point.step, point.average_wealth, point.median_wealth, point.bankruptcy_count
        )
        .expect("write failed");
    }
}

fn write_sweep_csv(path: &str, runs: &[StrategyRunSummary]) {
    let mut f = File::create(path).expect("Unable to create sweep csv");
    writeln!(f, "savings_rate,risk_profile,emergency_fund,average_wealth,median_wealth,max_wealth,min_wealth,gini,bankruptcy_count,run_duration,init_time,step_compute_time,metrics_calc_time,communication_overhead,score").expect("write failed");
    for run in runs {
        writeln!(
            f,
            "{:.6},{:.6},{:.6},{:.6},{:.6},{:.6},{:.6},{:.6},{},{:.6},{:.6},{:.6},{:.6},{:.6},{:.6}",
            run.savings_rate,
            run.risk_profile,
            run.emergency_fund,
            run.average_wealth,
            run.median_wealth,
            run.max_wealth,
            run.min_wealth,
            run.gini_coefficient,
            run.bankruptcy_count,
            run.run_duration,
            run.init_time,
            run.step_compute_time,
            run.metrics_calc_time,
            run.communication_overhead,
            score_run(run),
        )
        .expect("write failed");
    }
}

fn write_html(path: &str, advice: &str, runs: &[StrategyRunSummary]) {
    let mut sorted = runs.to_vec();
    sorted.sort_by(|a, b| {
        score_run(b)
            .partial_cmp(&score_run(a))
            .unwrap_or(Ordering::Equal)
    });
    let top_rows = sorted
        .iter()
        .take(10)
        .map(|r| {
            format!(
                "<tr><td>{:.0}%</td><td>{:.2}</td><td>${:.0}</td><td>${:.2}</td><td>{}</td><td>{:.3}</td><td>{:.2}</td></tr>",
                r.savings_rate * 100.0,
                r.risk_profile,
                r.emergency_fund,
                r.median_wealth,
                r.bankruptcy_count,
                r.gini_coefficient,
                score_run(r)
            )
        })
        .collect::<Vec<_>>()
        .join("");

    let html = format!(
        "<!doctype html><html><head><meta charset=\"utf-8\"><title>Financial Simulation Report</title><style>body{{font-family:Georgia,serif;background:#f5f7f2;color:#182019;margin:0;padding:24px}}.card{{background:#fff;border:1px solid #d9e1d4;border-radius:10px;padding:16px;margin-bottom:16px}}table{{border-collapse:collapse;width:100%}}th,td{{border:1px solid #d9e1d4;padding:8px;text-align:left}}th{{background:#eef4ea}}</style></head><body><div class=\"card\"><h1>Financial Simulation Report</h1><pre>{}</pre></div><div class=\"card\"><h2>Top Strategy Candidates</h2><table><thead><tr><th>Savings</th><th>Risk</th><th>Emergency Fund</th><th>Median Wealth</th><th>Bankruptcies</th><th>Gini</th><th>Score</th></tr></thead><tbody>{}</tbody></table></div></body></html>",
        advice.replace('&', "&amp;").replace('<', "&lt;"),
        top_rows
    );
    let mut f = File::create(path).expect("Unable to create report html");
    f.write_all(html.as_bytes()).expect("write failed");
}

pub fn write_single_run_artifacts(
    config: &Config,
    mode: &str,
    summary: &FinancialSummary,
) -> ArtifactPaths {
    let run_dir = create_run_dir(config.output_base_dir(), mode);
    let summary_json = format!("{}/summary.json", run_dir);
    let timeseries_csv = format!("{}/timeseries.csv", run_dir);
    let advice_txt = format!("{}/advice.txt", run_dir);
    let report_html = format!("{}/report.html", run_dir);

    let run = StrategyRunSummary::from_financial_summary(summary);
    let advice = build_advice(&run, summary.num_agents);

    let json = serde_json::to_string_pretty(summary).expect("json serialization failed");
    fs::write(&summary_json, json).expect("Unable to write summary.json");
    write_timeseries_csv(&timeseries_csv, summary);
    fs::write(&advice_txt, advice.clone()).expect("Unable to write advice.txt");
    write_html(&report_html, &advice, &[run]);

    ArtifactPaths {
        run_dir,
        summary_json,
        timeseries_csv,
        advice_txt,
        report_html,
        sweep_results_csv: None,
    }
}

pub fn write_sweep_artifacts(
    config: &Config,
    mode: &str,
    runs: &[StrategyRunSummary],
    best: &StrategyRunSummary,
    total_agents: u32,
) -> ArtifactPaths {
    let run_dir = create_run_dir(config.output_base_dir(), mode);
    let summary_json = format!("{}/summary.json", run_dir);
    let timeseries_csv = format!("{}/timeseries.csv", run_dir);
    let advice_txt = format!("{}/advice.txt", run_dir);
    let report_html = format!("{}/report.html", run_dir);
    let sweep_results_csv = format!("{}/sweep_results.csv", run_dir);

    let advice = build_advice(best, total_agents);
    let json = serde_json::to_string_pretty(runs).expect("json serialization failed");
    fs::write(&summary_json, json).expect("Unable to write summary.json");
    fs::write(
        &timeseries_csv,
        "step,average_wealth,median_wealth,bankruptcy_count\n",
    )
    .expect("Unable to write timeseries placeholder");
    fs::write(&advice_txt, advice.clone()).expect("Unable to write advice.txt");
    write_sweep_csv(&sweep_results_csv, runs);
    write_html(&report_html, &advice, runs);

    ArtifactPaths {
        run_dir,
        summary_json,
        timeseries_csv,
        advice_txt,
        report_html,
        sweep_results_csv: Some(sweep_results_csv),
    }
}

#![allow(unused)]
mod financial_model;

use financial_model::config::Config;
use financial_model::report::write_sweep_artifacts;
#[cfg(all(feature = "distributed_mpi", feature = "parallel"))]
use financial_model::runner::{
    generate_strategy_space, run_single_strategy_with_index, ExecutionMode,
};
use financial_model::StrategyRunSummary;

#[cfg(all(feature = "distributed_mpi", feature = "parallel"))]
use krabmaga::explore::local_sweep::SweepJob;
#[cfg(all(feature = "distributed_mpi", feature = "parallel"))]
use krabmaga::explore::mpi::sweep::sweep_mpi;

#[cfg(all(feature = "distributed_mpi", feature = "parallel"))]
use mpi::traits::*;
use std::cmp::Ordering;

#[cfg(all(feature = "distributed_mpi", feature = "parallel"))]
fn score(run: &StrategyRunSummary) -> f32 {
    let bankruptcy_penalty = run.bankruptcy_rate * 100_000.0;
    let high_risk_penalty = (run.p90_net_worth - run.p10_net_worth).max(0.0) * 0.1;
    run.median_net_worth - bankruptcy_penalty - high_risk_penalty
}

#[cfg(all(feature = "distributed_mpi", feature = "parallel"))]
fn main() {
    let universe = mpi::initialize().expect("MPI init failed");
    let world = universe.world();
    let is_root = world.rank() == 0;

    let config_path = std::env::var("KRAB_CONFIG_PATH")
        .unwrap_or_else(|_| "examples/config_comprehensive.json".to_string());
    let config = Config::read_from(&config_path);
    let strategies = generate_strategy_space(&config);

    if strategies.is_empty() {
        if is_root {
            panic!("No strategies generated from config. Check strategy_sweeps configuration.");
        }
        return;
    }

    let jobs: Vec<SweepJob<usize>> = (0..strategies.len())
        .map(|idx| SweepJob {
            conf_id: idx as u64,
            rep_id: 0,
            config: idx,
        })
        .collect();

    let gathered = sweep_mpi(&world, &jobs, 0, |job| {
        let idx = job.config;
        let strategy = &strategies[idx];
        let summary =
            run_single_strategy_with_index(&config, strategy, idx, ExecutionMode::Multithreaded);
        let run = StrategyRunSummary::from_financial_summary(&summary);
        (
            run,
            krabmaga::engine::run::RunStats {
                run_duration: summary.run_duration,
                executed_steps: summary.steps as u64,
            },
        )
    })
    .expect("MPI sweep failed");

    if !is_root {
        return;
    }

    let records = gathered.expect("Root rank must receive gathered strategy summaries");
    let all_runs: Vec<StrategyRunSummary> = records.into_iter().map(|r| r.result).collect();
    if all_runs.len() != strategies.len() {
        panic!(
            "Gathered {} strategy runs, expected {}",
            all_runs.len(),
            strategies.len()
        );
    }

    let best = all_runs
        .iter()
        .max_by(|a, b| score(a).partial_cmp(&score(b)).unwrap_or(Ordering::Equal))
        .expect("No strategies ran");

    let global_best_score = score(best);

    let artifacts = write_sweep_artifacts(&config, "mpi_parallel", &all_runs, best);

    println!("\nMPI headless run artifacts:");
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
    println!(
        "  Global best score (all-reduced): {:.2}",
        global_best_score
    );
}

#[cfg(not(all(feature = "distributed_mpi", feature = "parallel")))]
fn main() {
    println!("Please enable both 'distributed_mpi' and 'parallel' features to run this example.");
}

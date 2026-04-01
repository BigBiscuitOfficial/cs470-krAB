#![allow(unused)]
mod financial_model;

use financial_model::config::Config;
#[cfg(feature = "distributed_mpi")]
use financial_model::mpi_utils::{allreduce_best_score, gather_strategy_summaries_root};
#[cfg(feature = "distributed_mpi")]
use financial_model::partitioning::{contiguous_rank_workload, strategy_space_size};
use financial_model::report::write_sweep_artifacts;
#[cfg(feature = "distributed_mpi")]
use financial_model::runner::{
    describe_strategy, generate_strategy_space, run_single_strategy, ExecutionMode,
};
use financial_model::{FinancialState, StrategyRunSummary};

#[cfg(feature = "distributed_mpi")]
use mpi::traits::*;
use std::cmp::Ordering;

#[cfg(feature = "distributed_mpi")]
fn score(run: &StrategyRunSummary) -> f32 {
    let bankruptcy_penalty = run.bankruptcy_rate * 100_000.0;
    let high_risk_penalty = (run.p90_net_worth - run.p10_net_worth).max(0.0) * 0.1;
    run.median_net_worth - bankruptcy_penalty - high_risk_penalty
}

#[cfg(feature = "distributed_mpi")]
fn main() {
    let universe = mpi::initialize().expect("MPI init failed");
    let world = universe.world();
    let rank = world.rank();
    let world_size = world.size();
    let is_root = world.rank() == 0;

    let config = Config::read_from("examples/config_comprehensive.json");
    let strategies = generate_strategy_space(&config);
    let total_from_config = strategy_space_size(&config);

    if total_from_config != strategies.len() {
        panic!(
            "Strategy count mismatch: partitioning reports {}, runner generated {}",
            total_from_config,
            strategies.len()
        );
    }

    if strategies.is_empty() {
        if is_root {
            panic!("No strategies generated from config. Check strategy_sweeps configuration.");
        }
        return;
    }

    let workload = contiguous_rank_workload(strategies.len(), rank, world_size);
    println!(
        "[rank {}/{}] assigned strategy indices [{}..{}) ({} total)",
        rank,
        world_size,
        workload.start,
        workload.end,
        workload.len()
    );

    let mut local_runs: Vec<StrategyRunSummary> = Vec::with_capacity(workload.len());
    for idx in workload.start..workload.end {
        let strategy = &strategies[idx];
        let summary = run_single_strategy(&config, strategy, ExecutionMode::Serial);
        let mut run = StrategyRunSummary::from_financial_summary(&summary);
        run.strategy_desc = describe_strategy(strategy);
        local_runs.push(run);
    }

    let local_best_score = local_runs
        .iter()
        .map(score)
        .fold(f32::NEG_INFINITY, f32::max);
    let global_best_score = allreduce_best_score(&world, local_best_score)
        .expect("MPI allreduce for best score failed");

    let gathered = gather_strategy_summaries_root(&world, 0, &local_runs)
        .expect("MPI gather for strategy summaries failed");

    if !is_root {
        return;
    }

    let mut all_runs = gathered.expect("Root rank must receive gathered strategy summaries");
    if all_runs.len() != strategies.len() {
        panic!(
            "Gathered {} strategy runs, expected {}",
            all_runs.len(),
            strategies.len()
        );
    }

    all_runs.sort_by(|a, b| a.strategy_desc.cmp(&b.strategy_desc));

    let best = all_runs
        .iter()
        .max_by(|a, b| score(a).partial_cmp(&score(b)).unwrap_or(Ordering::Equal))
        .expect("No strategies ran");

    let artifacts = write_sweep_artifacts(&config, "mpi", &all_runs, best);

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

#[cfg(not(feature = "distributed_mpi"))]
fn main() {
    println!("Please enable the 'distributed_mpi' feature to run this example.");
}

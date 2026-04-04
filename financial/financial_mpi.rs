#![allow(unused)]
mod financial_model;

use financial_model::config::Config;
use financial_model::report::write_sweep_artifacts;
#[cfg(feature = "distributed_mpi")]
use financial_model::runner::{
    generate_strategy_space, run_single_strategy_with_index, ExecutionMode,
};
use financial_model::StrategyRunSummary;

#[cfg(feature = "distributed_mpi")]
use krabmaga::explore::local_sweep::SweepJob;
#[cfg(feature = "distributed_mpi")]
use krabmaga::explore::mpi::sweep::{sweep_mpi_with_timings, SweepMpiSubroutineTimings};

#[cfg(feature = "distributed_mpi")]
use mpi::traits::*;
use std::cmp::Ordering;
#[cfg(feature = "distributed_mpi")]
use std::fs;
#[cfg(feature = "distributed_mpi")]
use std::path::Path;
#[cfg(feature = "distributed_mpi")]
use std::time::{SystemTime, UNIX_EPOCH};

#[cfg(feature = "distributed_mpi")]
fn score(run: &StrategyRunSummary) -> f32 {
    let bankruptcy_penalty = run.bankruptcy_rate * 100_000.0;
    let high_risk_penalty = (run.p90_net_worth - run.p10_net_worth).max(0.0) * 0.1;
    run.median_net_worth - bankruptcy_penalty - high_risk_penalty
}

#[cfg(feature = "distributed_mpi")]
fn next_numeric_csv_path(base_dir: &str) -> String {
    let dir = Path::new(base_dir);
    fs::create_dir_all(dir).expect("Failed to create MPI profiling output directory");

    let mut max_id: u32 = 0;
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            if let Some(name) = entry.file_name().to_str() {
                if let Some(stem) = name.strip_suffix(".csv") {
                    if stem.chars().all(|c| c.is_ascii_digit()) {
                        if let Ok(id) = stem.parse::<u32>() {
                            if id > max_id {
                                max_id = id;
                            }
                        }
                    }
                }
            }
        }
    }

    format!("{}/{:06}.csv", base_dir, max_id + 1)
}

#[cfg(feature = "distributed_mpi")]
fn write_mpi_subroutine_timings_csv(
    csv_path: &str,
    config_path: &str,
    ranks: i32,
    jobs_total: usize,
    jobs_local_root: usize,
    config: &Config,
    timings: SweepMpiSubroutineTimings,
) {
    let timestamp_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);

    let csv = format!(
        "timestamp_ms,mode,ranks,jobs_total,jobs_local_root,num_agents,num_steps,num_reps,local_execute_s,serialize_local_s,collective_transfer_s,deserialize_remote_s,sort_records_s,sweep_total_s,config_path\n{timestamp_ms},mpi,{ranks},{jobs_total},{jobs_local_root},{num_agents},{num_steps},{num_reps},{local_execute_s:.6},{serialize_local_s:.6},{collective_transfer_s:.6},{deserialize_remote_s:.6},{sort_records_s:.6},{sweep_total_s:.6},\"{config_path}\"\n",
        num_agents = config.simulation.num_agents,
        num_steps = config.simulation.steps,
        num_reps = config.simulation.reps,
        local_execute_s = timings.local_execute_s,
        serialize_local_s = timings.serialize_local_s,
        collective_transfer_s = timings.collective_transfer_s,
        deserialize_remote_s = timings.deserialize_remote_s,
        sort_records_s = timings.sort_records_s,
        sweep_total_s = timings.sweep_total_s,
    );

    fs::write(csv_path, csv).expect("Failed to write MPI subroutine timings CSV");
}

#[cfg(feature = "distributed_mpi")]
fn main() {
    let universe = mpi::initialize().expect("MPI init failed");
    let world = universe.world();
    let rank = world.rank();
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

    let (gathered, timings) = sweep_mpi_with_timings(&world, &jobs, 0, |job| {
        let idx = job.config;
        let strategy = &strategies[idx];
        let summary = run_single_strategy_with_index(&config, strategy, idx, ExecutionMode::Serial);
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
    let mut all_runs: Vec<StrategyRunSummary> = records.into_iter().map(|r| r.result).collect();
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

    let artifacts = write_sweep_artifacts(&config, "mpi", &all_runs, best);

    let profiling_dir = format!("{}/scaling_results/mpi", config.output_base_dir());
    let timing_csv_path = next_numeric_csv_path(&profiling_dir);
    let (root_start, root_end) = {
        let world_size = world.size() as usize;
        let total = jobs.len();
        let base = total / world_size;
        let rem = total % world_size;
        let rank = 0usize;
        let start = rank * base + rank.min(rem);
        let size = base + usize::from(rank < rem);
        (start, start + size)
    };
    let jobs_local_root = root_end.saturating_sub(root_start);
    write_mpi_subroutine_timings_csv(
        &timing_csv_path,
        &config_path,
        world.size(),
        jobs.len(),
        jobs_local_root,
        &config,
        timings,
    );

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
    println!("- mpi subroutine timings: {}", timing_csv_path);
}

#[cfg(not(feature = "distributed_mpi"))]
fn main() {
    println!("Please enable the 'distributed_mpi' feature to run this example.");
}

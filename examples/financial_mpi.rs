#![allow(unused)]
mod financial_model;

use financial_model::config::Config;
use financial_model::report::write_sweep_artifacts;
use financial_model::{FinancialState, StrategyRunSummary};

#[cfg(feature = "distributed_mpi")]
use krabmaga::explore::model_exploration::ExploreMode;
#[cfg(feature = "distributed_mpi")]
use krabmaga::{
    build_configurations, build_dataframe, count_tts, explore_distributed_mpi, extend_dataframe,
    simulate_explore,
};
#[cfg(feature = "distributed_mpi")]
use mpi::traits::*;
#[cfg(feature = "distributed_mpi")]
use std::fs;
#[cfg(feature = "distributed_mpi")]
use std::path::PathBuf;

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
    let is_root = world.rank() == 0;

    if is_root {
        println!("MPI distributed sweep not yet implemented for comprehensive strategy model.");
        println!("TODO: Refactor to distribute strategy combinations across MPI ranks.");
        println!("For now, use serial or multithreaded modes.");
    }

    // TODO: Implement distributed strategy sweep
    // The challenge is that LifeStrategy contains enums (HousingStrategy, DebtStrategy, RetirementGoal)
    // which don't work directly with explore_distributed_mpi! macro that expects numeric types.
    // Possible approaches:
    // 1. Encode strategies as integers and decode on each rank
    // 2. Use a custom MPI distribution logic instead of the macro
    // 3. Generate all strategy combinations on root and distribute work units
}

#[cfg(not(feature = "distributed_mpi"))]
fn main() {
    println!("Please enable the 'distributed_mpi' feature to run this example.");
}

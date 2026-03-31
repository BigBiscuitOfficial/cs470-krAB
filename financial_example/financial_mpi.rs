#![allow(unused)]
mod financial_model;
use financial_model::FinancialState;

#[cfg(feature = "distributed_mpi")]
use krabmaga::explore::model_exploration::ExploreMode;
#[cfg(feature = "distributed_mpi")]
use krabmaga::{
    build_configurations, build_dataframe, count_tts, explore_distributed_mpi, extend_dataframe,
    simulate_explore,
};
#[cfg(feature = "distributed_mpi")]
use mpi::{environment::Universe, traits::*};

#[cfg(feature = "distributed_mpi")]
fn main() {
    let _universe = mpi::initialize().unwrap();
    let step = 100;
    let reps = 2;

    let inflation_rate = vec![0.01, 0.02, 0.03];
    let market_return = vec![0.02, 0.05, 0.08];
    let job_loss_prob = vec![0.01, 0.05, 0.10];

    // MPI macro for parameter sweeping
    let _results = explore_distributed_mpi!(
        step,
        reps,
        FinancialState,
        input {
            inflation_rate: f32,
            market_return: f32,
            job_loss_prob: f32
        },
        output [
            average_wealth: f32,
            median_wealth: f32,
            max_wealth: f32,
            min_wealth: f32,
            gini_coefficient: f32,
            bankruptcy_count: u32
        ],
        ExploreMode::Exaustive,
    );
}

#[cfg(not(feature = "distributed_mpi"))]
fn main() {
    println!("Please enable the 'distributed_mpi' feature to run this example.");
}

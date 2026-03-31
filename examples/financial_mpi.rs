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
fn score(run: &StrategyRunSummary) -> f32 {
    let bankruptcy_penalty = run.bankruptcy_count as f32 * 50_000.0;
    let gini_penalty = run.gini_coefficient * 100_000.0;
    run.median_wealth - bankruptcy_penalty - gini_penalty
}

#[cfg(feature = "distributed_mpi")]
fn main() {
    let universe = mpi::initialize().expect("MPI init failed");
    let world = universe.world();
    let is_root = world.rank() == 0;

    let config = Config::read_from("examples/config.json");

    let step = config.simulation.steps;
    let reps = config.simulation.reps;

    let total_steps = vec![config.simulation.steps];
    let inflation_rate = vec![config.macro_economics.inflation_rate];
    let market_return = vec![config.macro_economics.market_return];
    let job_loss_prob = vec![config.macro_economics.job_loss_prob];
    let savings_rate = config.strategy_sweeps.savings_rates.clone();
    let risk_profile = config.strategy_sweeps.risk_profiles.clone();
    let emergency_fund = config.strategy_sweeps.emergency_funds.clone();

    if is_root {
        println!(
            "Starting distributed headless sweep on {} ranks",
            world.size()
        );
    }

    let rows = explore_distributed_mpi!(
        step,
        reps,
        FinancialState,
        input {
            total_steps: u32,
            inflation_rate: f32,
            market_return: f32,
            job_loss_prob: f32,
            savings_rate: f32,
            risk_profile: f32,
            emergency_fund: f32
        },
        output [
            average_wealth: f32,
            median_wealth: f32,
            max_wealth: f32,
            min_wealth: f32,
            gini_coefficient: f32,
            bankruptcy_count: u32,
            init_time: f32,
            step_compute_time: f32,
            metrics_calc_time: f32
        ],
        ExploreMode::Exaustive,
    );

    if is_root {
        let mut runs = Vec::with_capacity(rows.len());
        for r in &rows {
            let pure = r.init_time + r.step_compute_time + r.metrics_calc_time;
            let overhead = (r.run_duration - pure).max(0.0);
            runs.push(StrategyRunSummary {
                savings_rate: r.savings_rate,
                risk_profile: r.risk_profile,
                emergency_fund: r.emergency_fund,
                average_wealth: r.average_wealth,
                median_wealth: r.median_wealth,
                max_wealth: r.max_wealth,
                min_wealth: r.min_wealth,
                gini_coefficient: r.gini_coefficient,
                bankruptcy_count: r.bankruptcy_count,
                init_time: r.init_time,
                step_compute_time: r.step_compute_time,
                metrics_calc_time: r.metrics_calc_time,
                run_duration: r.run_duration,
                communication_overhead: overhead,
            });
        }

        runs.sort_by(|a, b| {
            score(b)
                .partial_cmp(&score(a))
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        if let Some(best) = runs.first() {
            let artifacts =
                write_sweep_artifacts(&config, "mpi", &runs, best, config.simulation.num_agents);
            println!("Headless MPI artifacts:");
            println!("- run dir: {}", artifacts.run_dir);
            println!("- report: {}", artifacts.report_html);
            println!(
                "- sweep csv: {}",
                artifacts.sweep_results_csv.unwrap_or_default()
            );
        } else {
            println!("No results returned from distributed sweep.");
        }
    }
}

#[cfg(not(feature = "distributed_mpi"))]
fn main() {
    println!("Please enable the 'distributed_mpi' feature to run this example.");
}

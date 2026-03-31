mod financial_model;

use financial_model::config::Config;
use financial_model::report::write_single_run_artifacts;
use financial_model::FinancialState;
use krabmaga::engine::schedule::Schedule;
use krabmaga::engine::state::State;
use std::time::Instant;

fn main() {
    let config = Config::read_from("examples/config.json");

    let mut state = FinancialState::new(
        config.simulation.steps,
        config.macro_economics.inflation_rate,
        config.macro_economics.market_return,
        config.macro_economics.job_loss_prob,
        config.strategy_sweeps.savings_rates[0],
        config.strategy_sweeps.risk_profiles[0],
        config.strategy_sweeps.emergency_funds[0],
    )
    .with_run_context(
        config.simulation.num_agents,
        config.simulation.reps,
        "serial",
    );

    let timer = Instant::now();
    for _ in 0..config.simulation.reps {
        let mut schedule = Schedule::new();
        state.init(&mut schedule);
        for _ in 0..config.simulation.steps {
            schedule.step(state.as_state_mut());
            if state.end_condition(&mut schedule) {
                break;
            }
        }
    }
    let run_duration = timer.elapsed().as_secs_f32();

    state.finalize_timing(run_duration);
    let summary = state.to_summary();
    let artifacts = write_single_run_artifacts(&config, "serial", &summary);

    println!("Headless run artifacts:");
    println!("- run dir: {}", artifacts.run_dir);
    println!("- report: {}", artifacts.report_html);
    println!("- summary: {}", artifacts.summary_json);
}

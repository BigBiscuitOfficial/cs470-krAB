mod financial_model;
use financial_model::FinancialState;
use krabmaga::simulate;

fn main() {
    let state = FinancialState::new(0.02, 0.05, 0.05);
    let step = 100;
    let reps = 1;
    let _ = simulate!(state, step, reps);
}

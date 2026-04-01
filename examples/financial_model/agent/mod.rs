mod expenses;
mod income;
mod lifecycle;
mod person;
mod portfolio;

pub use person::Person;

use crate::financial_model::state::FinancialState;
use crate::financial_model::utils::sample_normal;
use krabmaga::engine::agent::Agent;
use krabmaga::engine::state::State;

impl Agent for Person {
    fn step(&mut self, state: &mut dyn State) {
        let state = state
            .as_any()
            .downcast_ref::<FinancialState>()
            .expect("state downcast failed");
        let mut rng = state.rng.lock().expect("rng lock failed");

        self.age += 1;

        // Sample market conditions for this year
        let actual_inflation = sample_normal(
            &mut rng,
            state.config.macro_economics.inflation_mean,
            state.config.macro_economics.inflation_std_dev,
        );
        let actual_stock_return = sample_normal(
            &mut rng,
            state.config.macro_economics.stock_return_mean,
            state.config.macro_economics.stock_return_std_dev,
        );
        let actual_bond_return = sample_normal(
            &mut rng,
            state.config.macro_economics.bond_return_mean,
            state.config.macro_economics.bond_return_std_dev,
        );

        // Track unemployment duration
        let mut unemployed_months_this_year = 0_u32;
        if self.unemployed_months > 0 {
            unemployed_months_this_year = self.unemployed_months.min(12);
            self.unemployed_months -= unemployed_months_this_year;
        }

        // Process life events
        lifecycle::process_life_events(self, state, &mut rng);

        // Calculate income and taxes
        let income_result = income::calculate_income_and_taxes(
            self,
            state,
            actual_inflation,
            unemployed_months_this_year,
        );

        // Calculate expenses (includes debt minimums)
        let annual_expenses = expenses::calculate_expenses(
            self,
            state,
            income_result.total_gross_income,
            actual_inflation,
            &mut rng,
        );

        // Get debt minimums for portfolio management
        let total_minimum_debt_service = expenses::calculate_debt_minimums(self, state);

        // Remaining cash after income - expenses
        let mut available_cash = income_result.net_income - annual_expenses;

        // Manage debt
        portfolio::manage_debt(self, state, &mut available_cash, total_minimum_debt_service);

        // Handle shortfalls or invest surplus
        if available_cash < 0.0 {
            portfolio::handle_cash_shortfall(
                self,
                state,
                &mut available_cash,
                income_result.taxable_income,
                income_result.federal_tax,
                actual_stock_return,
            );
        } else {
            portfolio::invest_surplus(self, &mut available_cash);
        }

        // Apply market returns and rebalance
        portfolio::apply_market_returns(self, actual_stock_return, actual_bond_return);

        // Clamp accounts to prevent blow-up
        portfolio::clamp_accounts(self, state);

        // Store final state at end of simulation
        if state.step == (state.total_steps - 1) as u64 {
            let mut persons = state.final_persons.lock().expect("lock failed");
            persons.push(self.clone());
        }
    }
}

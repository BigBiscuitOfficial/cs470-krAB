use super::person::Person;
use crate::financial_model::config::RetirementGoal;
use crate::financial_model::state::FinancialState;
use crate::financial_model::utils::utils::allocate_by_target;
use rand::Rng;

/// Process all life events for the person this year
pub fn process_life_events(person: &mut Person, state: &FinancialState, rng: &mut impl Rng) {
    process_marriage(person, state, rng);
    process_divorce(person, state, rng);
    process_children(person, state, rng);
    process_disability(person, state, rng);
    process_job_loss(person, state, rng);
    process_inheritance(person, state, rng);
    process_promotion(person, state, rng);
    process_job_switch(person, state, rng);
    process_location_move(person, state, rng);
    process_retirement(person);
}

/// Handle marriage events
fn process_marriage(person: &mut Person, state: &FinancialState, rng: &mut impl Rng) {
    if !person.is_married
        && rng.random_range(0.0..1.0) < state.config.life_events.marriage_rate.clamp(0.0, 1.0)
    {
        person.is_married = true;
    }
}

/// Handle divorce events
fn process_divorce(person: &mut Person, state: &FinancialState, rng: &mut impl Rng) {
    if person.is_married
        && rng.random_range(0.0..1.0) < state.config.life_events.divorce_rate.clamp(0.0, 1.0)
    {
        person.is_married = false;

        // Split assets and debts 50/50
        person.home_equity *= 0.50;
        person.home_value *= 0.50;
        person.mortgage_balance *= 0.50;
        person.student_loan_debt *= 0.50;
        person.auto_loan_debt *= 0.50;
        person.credit_card_debt *= 0.50;
        person.k401_stocks *= 0.50;
        person.k401_bonds *= 0.50;
        person.brokerage_stocks *= 0.50;
        person.brokerage_bonds *= 0.50;
        person.liquid_cash *= 0.50;
    }
}

/// Handle birth of children
fn process_children(person: &mut Person, state: &FinancialState, rng: &mut impl Rng) {
    if rng.random_range(0.0..1.0) < state.config.life_events.birth_rate.clamp(0.0, 1.0) {
        person.num_children = person.num_children.saturating_add(1);
        person.annual_base_expenses *= 1.15;
    }
}

/// Handle permanent disability
fn process_disability(person: &mut Person, state: &FinancialState, rng: &mut impl Rng) {
    if !person.is_disabled
        && rng.random_range(0.0..1.0) < state.config.life_events.disability_rate.clamp(0.0, 1.0)
    {
        person.is_disabled = true;
        person.base_income *= state
            .config
            .healthcare
            .disability_income_replacement
            .clamp(0.0, 1.0);
    }
}

/// Handle job loss events
fn process_job_loss(person: &mut Person, state: &FinancialState, rng: &mut impl Rng) {
    if !person.retired
        && person.unemployed_months == 0
        && rng.random_range(0.0..1.0) < state.config.life_events.job_loss_rate.clamp(0.0, 1.0)
    {
        person.unemployed_months = 6;
        person.years_at_current_job = 0;
    }
}

/// Handle inheritance (ages 35-74, ~0.8% annual probability, $50K-$300K)
fn process_inheritance(person: &mut Person, state: &FinancialState, rng: &mut impl Rng) {
    if person.age >= 35 && person.age < 75 {
        if rng.random_range(0.0..1.0) < state.config.macro_economics.inheritance_prob_per_year {
            let inheritance = rng.random_range(
                state.config.macro_economics.inheritance_amount_range[0]
                    ..state.config.macro_economics.inheritance_amount_range[1],
            );
            // 70% goes to liquid cash, 30% directly invested
            person.liquid_cash += inheritance * 0.70;
            let target_stock_fraction = person.strategy.asset_allocation.stocks.clamp(0.0, 1.0);
            let (to_stocks, to_bonds) =
                allocate_by_target(inheritance * 0.30, target_stock_fraction);
            person.brokerage_stocks += to_stocks;
            person.brokerage_bonds += to_bonds;
        }
    }
}

/// Handle job promotion (requires 1+ years at job, ~15% annual chance declining with age, 8-25% raise)
fn process_promotion(person: &mut Person, state: &FinancialState, rng: &mut impl Rng) {
    if !person.retired && person.unemployed_months == 0 && person.years_at_current_job >= 1 {
        let promotion_chance = state.config.macro_economics.promotion_prob_per_year
            * (1.0 - (person.age as f32 - 25.0) / 80.0).max(0.3);

        if rng.random_range(0.0..1.0) < promotion_chance {
            let raise = rng.random_range(
                state.config.macro_economics.promotion_raise_range[0]
                    ..state.config.macro_economics.promotion_raise_range[1],
            );
            person.base_income *= 1.0 + raise;
            person.years_at_current_job = 0;
        }
    }
}

/// Handle job switch (requires 2+ years at job, ~12% annual chance, -5% to +35% income change)
fn process_job_switch(person: &mut Person, state: &FinancialState, rng: &mut impl Rng) {
    if !person.retired && person.unemployed_months == 0 && person.years_at_current_job >= 2 {
        if rng.random_range(0.0..1.0) < state.config.macro_economics.job_switch_prob_per_year {
            let change = rng.random_range(
                state.config.macro_economics.job_switch_raise_range[0]
                    ..state.config.macro_economics.job_switch_raise_range[1],
            );
            person.base_income *= 1.0 + change;
            person.years_at_current_job = 0;
            person.career_growth_rate = rng.random_range(0.005..0.035);
        }
    }
}

/// Handle location move (ages 25-64, ~8% annual chance, 0.7x-1.4x cost of living adjustment)
fn process_location_move(person: &mut Person, state: &FinancialState, rng: &mut impl Rng) {
    if person.age >= 25 && person.age < 65 {
        if rng.random_range(0.0..1.0) < state.config.macro_economics.location_move_prob_per_year {
            let old_multiplier = person.location_expense_multiplier;
            person.location_expense_multiplier = rng.random_range(
                state
                    .config
                    .macro_economics
                    .location_expense_multiplier_range[0]
                    ..state
                        .config
                        .macro_economics
                        .location_expense_multiplier_range[1],
            );

            // Adjust base expenses for new location cost of living
            let expense_ratio = person.location_expense_multiplier / old_multiplier;
            person.annual_base_expenses *= expense_ratio;
            person.monthly_rent *= expense_ratio;

            // Moving costs
            let moving_cost = rng.random_range(2_000.0..10_000.0);
            person.liquid_cash -= moving_cost;
        }
    }
}

/// Check and process retirement eligibility
fn process_retirement(person: &mut Person) {
    match &person.strategy.retirement_goal {
        RetirementGoal::Age { target } => {
            if person.age >= *target {
                person.retired = true;
            }
        }
        RetirementGoal::FIRE { expenses_multiple } => {
            let annual_exp = person.annual_base_expenses * (1.0 + person.num_children as f32 * 0.3);
            if person.portfolio_value() >= annual_exp * expenses_multiple {
                person.retired = true;
            }
        }
    }
}

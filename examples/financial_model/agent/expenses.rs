use super::person::Person;
use crate::financial_model::demographics::{HealthStatus, InsuranceType};
use crate::financial_model::state::FinancialState;
use crate::financial_model::strategies::HousingStrategy;
use crate::financial_model::utils::utils::annual_min_payment;
use rand::Rng;

/// Calculate all annual expenses including housing, healthcare, and debt minimums
pub fn calculate_expenses(
    person: &mut Person,
    state: &FinancialState,
    total_gross_income: f32,
    actual_inflation: f32,
    rng: &mut impl Rng,
) -> f32 {
    // Base expenses adjusted for inflation
    person.annual_base_expenses *= 1.0 + actual_inflation;
    let child_cost = person.num_children as f32 * state.config.macro_economics.child_annual_cost;

    // Spending discipline affects discretionary expenses
    let essential_expenses = person.annual_base_expenses * 0.70;
    let discretionary_expenses = person.annual_base_expenses * 0.30;
    let discretionary_multiplier = (2.0 - person.spending_discipline).clamp(0.70, 1.35);

    let mut annual_expenses =
        (essential_expenses + discretionary_expenses * discretionary_multiplier + child_cost)
            * person.location_expense_multiplier;

    let marriage_expense_multiplier = if person.is_married { 1.3 } else { 1.0 };
    annual_expenses *= marriage_expense_multiplier;

    // Healthcare expenses
    annual_expenses += calculate_healthcare_expenses(person, state, rng);

    // Housing expenses
    annual_expenses +=
        calculate_housing_expenses(person, state, total_gross_income, actual_inflation);

    // Debt minimum payments
    annual_expenses += calculate_debt_minimums(person, state);

    annual_expenses
}

/// Calculate healthcare premiums and out-of-pocket costs
fn calculate_healthcare_expenses(
    person: &Person,
    state: &FinancialState,
    rng: &mut impl Rng,
) -> f32 {
    let premium_multiplier = match person.health_insurance_type {
        InsuranceType::EmployerSponsored => 1.0,
        InsuranceType::ACAMarketplace => 1.15,
        InsuranceType::Medicaid => 0.35,
        InsuranceType::Medicare => 0.75,
        InsuranceType::Uninsured => 0.0,
    };
    let annual_premium_cost = person.monthly_health_premium * 12.0 * premium_multiplier;

    let (oop_prob, oop_min, oop_max) = match person.health_status {
        HealthStatus::Excellent => (0.15, 100.0, 900.0),
        HealthStatus::Good => (0.35, 250.0, 1_800.0),
        HealthStatus::Fair => (0.60, 1_000.0, 5_500.0),
        HealthStatus::Poor => (0.85, 2_500.0, 12_000.0),
    };
    let oop_coverage_multiplier = match person.health_insurance_type {
        InsuranceType::EmployerSponsored => 0.85,
        InsuranceType::ACAMarketplace => 1.10,
        InsuranceType::Medicaid => 0.65,
        InsuranceType::Medicare => 0.80,
        InsuranceType::Uninsured => 1.80,
    };
    let chronic_cost_multiplier = if person.has_chronic_condition {
        1.25
    } else {
        1.0
    };

    let random_oop_cost = if rng.random_range(0.0..1.0) < oop_prob {
        rng.random_range(oop_min..oop_max) * oop_coverage_multiplier * chronic_cost_multiplier
    } else {
        0.0
    };

    // Emergency medical events
    let health_emergency_prob_multiplier = match person.health_status {
        HealthStatus::Excellent => 0.7,
        HealthStatus::Good => 1.0,
        HealthStatus::Fair => 1.4,
        HealthStatus::Poor => 2.0,
    };
    let emergency_cost = if rng.random_range(0.0..1.0)
        < state.config.macro_economics.medical_emergency_prob * health_emergency_prob_multiplier
    {
        rng.random_range(
            state.config.macro_economics.emergency_cost_range[0]
                ..state.config.macro_economics.emergency_cost_range[1],
        ) * oop_coverage_multiplier
    } else {
        0.0
    };

    person.annual_healthcare_costs
        + annual_premium_cost
        + random_oop_cost
        + emergency_cost
        + person.life_insurance_premium
}

/// Calculate housing costs and handle home purchases
fn calculate_housing_expenses(
    person: &mut Person,
    state: &FinancialState,
    total_gross_income: f32,
    actual_inflation: f32,
) -> f32 {
    // Check if should buy a home
    if !person.owns_home && person.strategy.housing == HousingStrategy::Buy {
        attempt_home_purchase(person, state);
    }

    if person.owns_home {
        calculate_homeowner_expenses(person, state)
    } else {
        calculate_renter_expenses(person, state, total_gross_income, actual_inflation)
    }
}

/// Attempt to purchase a home if conditions are met
fn attempt_home_purchase(person: &mut Person, state: &FinancialState) {
    let target_home_price = 3.0 * person.base_income.max(0.0);
    let down_payment_percent = state.config.housing.down_payment_percent.clamp(0.0, 1.0);
    let down_payment = target_home_price * down_payment_percent;

    if person.liquid_cash >= down_payment && target_home_price > 0.0 {
        person.liquid_cash -= down_payment;
        person.owns_home = true;
        person.home_value = target_home_price;
        person.mortgage_balance = (target_home_price - down_payment).max(0.0);
        person.home_equity = (person.home_value - person.mortgage_balance).max(0.0);

        // Fixed-rate mortgage payment approximation
        let monthly_rate = state.config.housing.mortgage_interest_rate / 12.0;
        let num_payments = (state.config.housing.mortgage_years * 12) as f32;
        person.monthly_mortgage_payment = if person.mortgage_balance <= 0.0 {
            0.0
        } else if monthly_rate > 0.0 {
            let growth = (1.0 + monthly_rate).powf(num_payments);
            person.mortgage_balance * monthly_rate * growth / (growth - 1.0)
        } else {
            person.mortgage_balance / num_payments.max(1.0)
        };
    }
}

/// Calculate expenses for homeowners
fn calculate_homeowner_expenses(person: &mut Person, state: &FinancialState) -> f32 {
    person.home_value *= 1.0 + state.config.housing.appreciation_rate;

    let property_tax = person.home_value * state.config.housing.property_tax_rate;
    let maintenance = person.home_value * state.config.housing.maintenance_rate;
    let mut expenses = property_tax + maintenance;

    if person.mortgage_balance > 0.0 {
        let annual_mortgage_payment = person.monthly_mortgage_payment * 12.0;
        let interest_portion =
            person.mortgage_balance * state.config.housing.mortgage_interest_rate;
        let principal_portion = (annual_mortgage_payment - interest_portion)
            .max(0.0)
            .min(person.mortgage_balance);

        person.mortgage_balance -= principal_portion;
        expenses += annual_mortgage_payment;
        person.home_equity = (person.home_value - person.mortgage_balance).max(0.0);
    } else {
        person.home_equity = person.home_value.max(0.0);
    }

    expenses
}

/// Calculate expenses for renters
fn calculate_renter_expenses(
    person: &mut Person,
    state: &FinancialState,
    total_gross_income: f32,
    actual_inflation: f32,
) -> f32 {
    person.monthly_rent *= 1.0 + actual_inflation;
    let income_scaled_rent =
        (total_gross_income * state.config.housing.rent_as_pct_income).max(0.0) / 12.0;
    person.monthly_rent = person.monthly_rent.max(income_scaled_rent);
    person.monthly_rent * 12.0
}

/// Calculate minimum debt payments
pub fn calculate_debt_minimums(person: &Person, state: &FinancialState) -> f32 {
    let student_loan_minimum = annual_min_payment(
        person.student_loan_debt,
        state.config.debt.student_loan_min_payment_rate,
        state.config.debt.student_loan_min_payment_floor,
    );
    let auto_loan_minimum = annual_min_payment(
        person.auto_loan_debt,
        state.config.debt.auto_loan_min_payment_rate,
        state.config.debt.auto_loan_min_payment_floor,
    );
    let cc_minimum = annual_min_payment(
        person.credit_card_debt,
        state.config.debt.credit_card_min_payment_rate,
        state.config.debt.credit_card_min_payment_floor,
    );

    student_loan_minimum + auto_loan_minimum + cc_minimum
}

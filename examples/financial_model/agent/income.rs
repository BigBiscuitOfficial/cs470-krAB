use super::person::Person;
use crate::financial_model::demographics::EducationLevel;
use crate::financial_model::state::FinancialState;
use crate::financial_model::utils::allocate_by_target;

/// Result of income and tax calculations
pub struct IncomeResult {
    pub total_gross_income: f32,
    pub taxable_income: f32,
    pub federal_tax: f32,
    pub net_income: f32,
    pub employee_401k_contribution: f32,
    pub employer_match: f32,
}

/// Calculate all income, taxes, and 401k contributions
pub fn calculate_income_and_taxes(
    person: &mut Person,
    state: &FinancialState,
    actual_inflation: f32,
    unemployed_months_this_year: u32,
) -> IncomeResult {
    // Education income multipliers
    let education_income_multiplier = match person.education_level {
        EducationLevel::HighSchool | EducationLevel::Associates => 1.0,
        EducationLevel::Bachelors => 1.5,
        EducationLevel::Masters => 2.0,
        EducationLevel::PhD => 2.2,
    };

    // Career growth (normal annual raises)
    if !person.retired {
        person.base_income *= 1.0 + actual_inflation * 0.6 + person.career_growth_rate;
        person.years_at_current_job += 1;
    }

    // Calculate earned income
    let education_adjusted_income = person.base_income * education_income_multiplier;
    let disability_income = if person.is_disabled {
        education_adjusted_income * state.config.healthcare.disability_income_replacement
    } else {
        education_adjusted_income
    };
    let employed_fraction = ((12 - unemployed_months_this_year) as f32 / 12.0).clamp(0.0, 1.0);
    let earned_income = if person.retired {
        0.0
    } else {
        disability_income * employed_fraction
    };

    // Social security for retirees
    let social_security_income = if person.retired {
        (education_adjusted_income * 0.35).max(0.0)
    } else {
        0.0
    };

    // Marriage income boost
    let marriage_income_multiplier = if person.is_married { 1.5 } else { 1.0 };
    let primary_income = if person.retired {
        social_security_income
    } else {
        earned_income * marriage_income_multiplier
    };

    let total_gross_income = primary_income;

    // 401k contributions
    let employer_match = if person.retired {
        0.0
    } else {
        (total_gross_income * state.config.tax_system.k401_match_rate)
            .min(state.config.tax_system.k401_contribution_limit * 0.5)
    };
    let employee_401k_contribution = if !person.retired {
        (total_gross_income * 0.06)
            .min(state.config.tax_system.k401_contribution_limit - employer_match)
    } else {
        0.0
    };

    // Tax calculation
    let taxable_income = total_gross_income - employee_401k_contribution;
    let federal_tax = state.calculate_income_tax(taxable_income);
    let net_income = taxable_income - federal_tax;

    // Allocate 401k contributions to stocks/bonds
    let target_stock_fraction = person.strategy.asset_allocation.stocks.clamp(0.0, 1.0);
    let (to_401k_stocks, to_401k_bonds) = allocate_by_target(
        employee_401k_contribution + employer_match,
        target_stock_fraction,
    );
    person.k401_stocks += to_401k_stocks;
    person.k401_bonds += to_401k_bonds;

    IncomeResult {
        total_gross_income,
        taxable_income,
        federal_tax,
        net_income,
        employee_401k_contribution,
        employer_match,
    }
}

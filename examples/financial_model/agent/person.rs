use crate::financial_model::demographics::*;
use crate::financial_model::strategies::*;

/// An agent representing a person with financial lifecycle
#[derive(Clone)]
pub struct Person {
    pub id: u32,
    pub age: u32,
    pub starting_age: u32,

    // Life status
    pub is_married: bool,
    pub num_children: u32,
    pub is_disabled: bool,
    pub unemployed_months: u32,
    pub retired: bool,

    // Income & career
    pub base_income: f32,
    pub career_growth_rate: f32,
    pub years_at_current_job: u8,
    pub education_level: EducationLevel,
    pub gender: Gender,
    pub race_ethnicity: RaceEthnicity,
    pub geographic_region: Region,
    pub health_status: HealthStatus,
    pub health_insurance_type: InsuranceType,
    pub monthly_health_premium: f32,
    pub annual_healthcare_costs: f32,
    pub has_chronic_condition: bool,
    pub financial_literacy_score: f32,
    pub spending_discipline: f32,
    pub risk_tolerance: f32,
    pub career_ambition: CareerAmbition,
    pub social_network_strength: f32,
    pub family_financial_support: f32,
    pub life_insurance_coverage: f32,
    pub life_insurance_premium: f32,

    // Accounts
    pub liquid_cash: f32,
    pub brokerage_stocks: f32,
    pub brokerage_bonds: f32,
    pub k401_stocks: f32,
    pub k401_bonds: f32,
    pub home_equity: f32,

    // Debts
    pub student_loan_debt: f32,
    pub auto_loan_debt: f32,
    pub mortgage_balance: f32,
    pub credit_card_debt: f32,

    // Housing
    pub owns_home: bool,
    pub home_value: f32,
    pub monthly_rent: f32,
    pub monthly_mortgage_payment: f32,

    // Strategy
    pub strategy: LifeStrategy,

    // Lifestyle
    pub annual_base_expenses: f32,
    pub location_expense_multiplier: f32,
}

impl Person {
    /// Calculate total net worth (assets - liabilities)
    pub fn net_worth(&self) -> f32 {
        self.liquid_cash
            + self.brokerage_stocks
            + self.brokerage_bonds
            + self.k401_stocks
            + self.k401_bonds
            + self.home_equity
            - self.student_loan_debt
            - self.auto_loan_debt
            - self.mortgage_balance
            - self.credit_card_debt
    }

    /// Calculate total outstanding debt
    pub fn total_debt(&self) -> f32 {
        self.student_loan_debt + self.auto_loan_debt + self.mortgage_balance + self.credit_card_debt
    }

    /// Calculate liquid wealth (cash + taxable brokerage)
    pub fn liquid_wealth(&self) -> f32 {
        self.liquid_cash + self.brokerage_stocks + self.brokerage_bonds
    }

    /// Calculate total portfolio value (taxable + retirement accounts)
    pub fn portfolio_value(&self) -> f32 {
        self.brokerage_stocks + self.brokerage_bonds + self.k401_stocks + self.k401_bonds
    }

    /// Calculate taxable brokerage account total
    pub fn taxable_brokerage_total(&self) -> f32 {
        self.brokerage_stocks + self.brokerage_bonds
    }

    /// Calculate 401k account total
    pub fn k401_total(&self) -> f32 {
        self.k401_stocks + self.k401_bonds
    }
}

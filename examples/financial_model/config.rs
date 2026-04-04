use serde::Deserialize;
use std::collections::HashMap;
use std::env;
use std::fs;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub macro_economics: MacroEconomics,
    pub tax_system: TaxSystem,
    pub housing: Housing,
    pub debt: Debt,
    #[serde(default)]
    pub demographics: Demographics,
    #[serde(default)]
    pub healthcare: Healthcare,
    #[serde(default)]
    pub personality_traits: PersonalityTraits,
    #[serde(default)]
    pub life_insurance: LifeInsurance,
    #[serde(default)]
    pub life_events: LifeEvents,
    pub strategy_sweeps: StrategySweeps,
    pub simulation: Simulation,
    pub output: Option<OutputConfig>,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct LifeEvents {
    pub marriage_rate: f32,
    pub divorce_rate: f32,
    pub birth_rate: f32,
    pub disability_rate: f32,
    pub job_loss_rate: f32,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct Demographics {
    pub education_income_multipliers: HashMap<String, f32>,
    pub education_unemployment_multipliers: HashMap<String, f32>,
    pub gender_pay_gap_female: f32,
    pub gender_pay_gap_nonbinary: f32,
    pub race_wealth_multipliers: HashMap<String, f32>,
    pub regional_col_multipliers: HashMap<String, f32>,
    pub regional_income_multipliers: HashMap<String, f32>,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct Healthcare {
    pub monthly_premium_ranges: HashMap<String, [f32; 2]>,
    pub annual_healthcare_base_cost: f32,
    pub health_status_cost_multipliers: HashMap<String, f32>,
    pub chronic_condition_annual_cost: f32,
    pub chronic_condition_probability: f32,
    pub disability_onset_probability: f32,
    pub disability_duration_years: f32,
    pub disability_income_replacement: f32,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct PersonalityTraits {
    pub financial_literacy_distribution: [f32; 3],
    pub spending_discipline_range: [f32; 2],
    pub risk_tolerance_by_education: HashMap<String, f32>,
    pub career_ambition_probabilities: [f32; 3],
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct LifeInsurance {
    pub coverage_by_income_multiple: f32,
    pub annual_premium_per_100k: f32,
    pub purchase_probability_by_education: HashMap<String, f32>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MacroEconomics {
    #[serde(default = "default_inflation_mean", alias = "inflation_rate")]
    pub inflation_mean: f32,
    #[serde(default)]
    pub inflation_std_dev: f32,
    #[serde(default = "default_stock_return_mean")]
    pub stock_return_mean: f32,
    #[serde(default, alias = "stock_volatility")]
    pub stock_return_std_dev: f32,
    #[serde(default = "default_bond_return_mean")]
    pub bond_return_mean: f32,
    #[serde(default, alias = "bond_volatility")]
    pub bond_return_std_dev: f32,
    #[serde(default = "default_safe_withdrawal_rate")]
    pub safe_withdrawal_rate: f32,
    pub real_estate_appreciation: f32,
    pub rent_growth_rate: f32,
    pub job_loss_prob: f32,
    pub unemployment_duration_years: f32,
    pub marriage_prob_per_year: f32,
    pub child_prob_per_year: f32,
    pub medical_emergency_prob: f32,
    pub child_annual_cost: f32,
    pub emergency_cost_range: [f32; 2],

    // New life events
    pub divorce_prob_per_year: f32,
    pub inheritance_prob_per_year: f32,
    pub inheritance_amount_range: [f32; 2],
    pub promotion_prob_per_year: f32,
    pub promotion_raise_range: [f32; 2],
    pub job_switch_prob_per_year: f32,
    pub job_switch_raise_range: [f32; 2],
    pub location_move_prob_per_year: f32,
    pub location_expense_multiplier_range: [f32; 2],
}

fn default_inflation_mean() -> f32 {
    0.02
}

fn default_stock_return_mean() -> f32 {
    0.07
}

fn default_bond_return_mean() -> f32 {
    0.03
}

fn default_safe_withdrawal_rate() -> f32 {
    0.04
}

#[derive(Debug, Clone, Deserialize)]
pub struct TaxBracket {
    #[serde(alias = "limit")]
    pub threshold: f32,
    pub rate: f32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TaxSystem {
    #[serde(alias = "brackets")]
    pub income_tax_brackets: Vec<TaxBracket>,
    #[serde(default = "default_long_term_capital_gains_rate")]
    pub long_term_capital_gains_rate: f32,
    pub standard_deduction: f32,
    pub k401_contribution_limit: f32,
    pub k401_match_rate: f32,
    pub early_withdrawal_penalty: f32,
}

fn default_long_term_capital_gains_rate() -> f32 {
    0.15
}

#[derive(Debug, Clone, Deserialize)]
pub struct Housing {
    pub rent_as_pct_income: f32,
    pub median_home_price: f32,
    #[serde(default = "default_down_payment_percent", alias = "down_payment_pct")]
    pub down_payment_percent: f32,
    pub mortgage_interest_rate: f32,
    pub mortgage_years: u32,
    #[serde(default = "default_property_tax_rate")]
    pub property_tax_rate: f32,
    #[serde(default = "default_maintenance_rate", alias = "maintenance_pct")]
    pub maintenance_rate: f32,
    #[serde(default = "default_appreciation_rate")]
    pub appreciation_rate: f32,
}

fn default_property_tax_rate() -> f32 {
    0.012
}

fn default_maintenance_rate() -> f32 {
    0.01
}

fn default_appreciation_rate() -> f32 {
    0.038
}

fn default_down_payment_percent() -> f32 {
    0.20
}

#[derive(Debug, Clone, Deserialize)]
pub struct Debt {
    pub student_loan_balance_range: [f32; 2],
    pub student_loan_rate: f32,
    #[serde(default = "default_auto_loan_balance_range")]
    pub auto_loan_balance_range: [f32; 2],
    #[serde(default = "default_auto_loan_rate")]
    pub auto_loan_rate: f32,
    pub credit_card_rate: f32,
    pub credit_card_limit: f32,
    #[serde(default = "default_student_loan_min_payment_rate")]
    pub student_loan_min_payment_rate: f32,
    #[serde(default = "default_student_loan_min_payment_floor")]
    pub student_loan_min_payment_floor: f32,
    #[serde(default = "default_auto_loan_min_payment_rate")]
    pub auto_loan_min_payment_rate: f32,
    #[serde(default = "default_auto_loan_min_payment_floor")]
    pub auto_loan_min_payment_floor: f32,
    #[serde(default = "default_credit_card_min_payment_rate")]
    pub credit_card_min_payment_rate: f32,
    #[serde(default = "default_credit_card_min_payment_floor")]
    pub credit_card_min_payment_floor: f32,
}

fn default_auto_loan_balance_range() -> [f32; 2] {
    [5_000.0, 35_000.0]
}

fn default_auto_loan_rate() -> f32 {
    0.072
}

fn default_student_loan_min_payment_rate() -> f32 {
    0.06
}

fn default_student_loan_min_payment_floor() -> f32 {
    1_200.0
}

fn default_auto_loan_min_payment_rate() -> f32 {
    0.08
}

fn default_auto_loan_min_payment_floor() -> f32 {
    1_800.0
}

fn default_credit_card_min_payment_rate() -> f32 {
    0.03
}

fn default_credit_card_min_payment_floor() -> f32 {
    300.0
}

#[derive(Debug, Clone, Deserialize)]
pub struct AssetAllocation {
    pub stocks: f32,
    pub bonds: f32,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type")]
pub enum RetirementGoal {
    Age { target: u32 },
    FIRE { expenses_multiple: f32 },
}

#[derive(Debug, Clone, Deserialize)]
pub struct StrategySweeps {
    pub housing_strategies: Vec<String>,
    pub debt_strategies: Vec<String>,
    pub asset_allocations: Vec<AssetAllocation>,
    pub retirement_goals: Vec<RetirementGoal>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Simulation {
    pub steps: u32,
    pub num_agents: u32,
    pub reps: u32,
    pub thread_count: Option<usize>,
    pub seed: Option<u64>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OutputConfig {
    pub base_dir: Option<String>,
    pub per_rank_debug: Option<bool>,
}

impl Config {
    pub fn read_from(path: &str) -> Self {
        let raw = fs::read_to_string(path).expect("Unable to read config file");
        serde_json::from_str(&raw).expect("Invalid config format")
    }

    pub fn output_base_dir(&self) -> String {
        if let Ok(path) = env::var("KRAB_OUTPUT_DIR") {
            let trimmed = path.trim();
            if !trimmed.is_empty() {
                return trimmed.to_string();
            }
        }

        self.output
            .as_ref()
            .and_then(|o| o.base_dir.as_deref())
            .unwrap_or("output")
            .to_string()
    }
}

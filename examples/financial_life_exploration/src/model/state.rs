use crate::model::household::Household;
use crate::scale_config;
use core::fmt;
use krabmaga::engine::schedule::Schedule;
use krabmaga::engine::state::State;
use krabmaga::rand::{rngs::StdRng, Rng, SeedableRng};
use std::any::Any;

pub const TARGET_SPEND_RATIO: f32 = 0.58;
pub const TARGET_RETIREMENT_COVERAGE: f32 = 1.10;

#[derive(Clone, Copy, Debug)]
pub struct FinancePolicy {
    pub frugality: f32,
    pub savings_discipline: f32,
    pub career_drive: f32,
    pub risk_tolerance: f32,
    pub resilience: f32,
    pub family_pressure: f32,
    pub education_investment: f32,
}

impl FinancePolicy {
    pub const PARAM_COUNT: usize = 7;

    pub fn from_parameters(parameters: &str) -> Self {
        let parts: Vec<&str> = parameters.split(';').collect();
        assert_eq!(
            parts.len(),
            Self::PARAM_COUNT,
            "Expected {} parameters, got {}",
            Self::PARAM_COUNT,
            parts.len()
        );

        FinancePolicy {
            frugality: clamp01(
                parts[0]
                    .parse::<f32>()
                    .expect("Invalid frugality parameter"),
            ),
            savings_discipline: clamp01(
                parts[1]
                    .parse::<f32>()
                    .expect("Invalid savings_discipline parameter"),
            ),
            career_drive: clamp01(
                parts[2]
                    .parse::<f32>()
                    .expect("Invalid career_drive parameter"),
            ),
            risk_tolerance: clamp01(
                parts[3]
                    .parse::<f32>()
                    .expect("Invalid risk_tolerance parameter"),
            ),
            resilience: clamp01(
                parts[4]
                    .parse::<f32>()
                    .expect("Invalid resilience parameter"),
            ),
            family_pressure: clamp01(
                parts[5]
                    .parse::<f32>()
                    .expect("Invalid family_pressure parameter"),
            ),
            education_investment: clamp01(
                parts[6]
                    .parse::<f32>()
                    .expect("Invalid education_investment parameter"),
            ),
        }
    }
}

impl fmt::Display for FinancePolicy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "frugality {:.2} savings {:.2} career {:.2} risk {:.2} resilience {:.2} family {:.2} education {:.2}",
            self.frugality,
            self.savings_discipline,
            self.career_drive,
            self.risk_tolerance,
            self.resilience,
            self.family_pressure,
            self.education_investment,
        )
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LifeStage {
    EarlyCareer,
    CareerBuilding,
    PeakCareer,
    Retirement,
}

pub fn clamp01(value: f32) -> f32 {
    value.clamp(0.0, 1.0)
}

pub fn life_stage(age: u32, retirement_age: u32) -> LifeStage {
    if age >= retirement_age {
        LifeStage::Retirement
    } else if age < 30 {
        LifeStage::EarlyCareer
    } else if age < 45 {
        LifeStage::CareerBuilding
    } else {
        LifeStage::PeakCareer
    }
}

pub fn benchmark_wealth_curve(year: usize) -> f32 {
    let t = year as f32;
    let early = 18_000.0 + 2_500.0 * t;
    let career = 190_000.0 / (1.0 + (-0.18 * (t - 18.0)).exp());
    let retirement = 240_000.0 / (1.0 + (-0.12 * (t - 40.0)).exp());
    early + career + retirement
}

pub struct FinanceLifeState {
    pub step: u64,
    pub run_id: u64,
    pub horizon: u64,
    pub households: u32,
    pub policy: FinancePolicy,
    pub rng: StdRng,
    pub yearly_avg_net_worth: Vec<f32>,
    pub yearly_avg_income: Vec<f32>,
    pub yearly_avg_debt: Vec<f32>,
    pub yearly_avg_spend_ratio: Vec<f32>,
    pub yearly_bankruptcy_rate: Vec<f32>,
    pub yearly_avg_retirement_coverage: Vec<f32>,
    pub yearly_retirement_samples: Vec<u32>,
    pub year_net_worth_sum: f32,
    pub year_income_sum: f32,
    pub year_debt_sum: f32,
    pub year_spend_ratio_sum: f32,
    pub year_retirement_coverage_sum: f32,
    pub year_retirement_samples: u32,
    pub year_bankruptcy_count: u32,
    pub year_has_activity: bool,
    pub total_bankruptcies: u32,
    pub total_promotions: u32,
    pub total_shocks: u32,
    pub total_family_events: u32,
    pub total_home_events: u32,
    pub total_inheritances: u32,
    pub total_training_events: u32,
}

impl FinanceLifeState {
    #[allow(dead_code)]
    pub fn new(run_id: usize, parameters: &str) -> FinanceLifeState {
        Self::new_with_parameters(run_id, parameters)
    }

    #[allow(dead_code)]
    pub fn new_with_parameters(run_id: usize, parameters: &str) -> FinanceLifeState {
        let config = scale_config();
        let seed = 0x5EED_F17Eu64.wrapping_add(run_id as u64 * 1_000_003);
        FinanceLifeState {
            step: 0,
            run_id: run_id as u64,
            horizon: config.horizon,
            households: config.households,
            policy: FinancePolicy::from_parameters(parameters),
            rng: StdRng::seed_from_u64(seed),
            yearly_avg_net_worth: Vec::new(),
            yearly_avg_income: Vec::new(),
            yearly_avg_debt: Vec::new(),
            yearly_avg_spend_ratio: Vec::new(),
            yearly_bankruptcy_rate: Vec::new(),
            yearly_avg_retirement_coverage: Vec::new(),
            yearly_retirement_samples: Vec::new(),
            year_net_worth_sum: 0.0,
            year_income_sum: 0.0,
            year_debt_sum: 0.0,
            year_spend_ratio_sum: 0.0,
            year_retirement_coverage_sum: 0.0,
            year_retirement_samples: 0,
            year_bankruptcy_count: 0,
            year_has_activity: false,
            total_bankruptcies: 0,
            total_promotions: 0,
            total_shocks: 0,
            total_family_events: 0,
            total_home_events: 0,
            total_inheritances: 0,
            total_training_events: 0,
        }
    }

    fn reset_history(&mut self) {
        self.yearly_avg_net_worth.clear();
        self.yearly_avg_income.clear();
        self.yearly_avg_debt.clear();
        self.yearly_avg_spend_ratio.clear();
        self.yearly_bankruptcy_rate.clear();
        self.yearly_avg_retirement_coverage.clear();
        self.yearly_retirement_samples.clear();
    }

    fn reset_year_accumulators(&mut self) {
        self.year_net_worth_sum = 0.0;
        self.year_income_sum = 0.0;
        self.year_debt_sum = 0.0;
        self.year_spend_ratio_sum = 0.0;
        self.year_retirement_coverage_sum = 0.0;
        self.year_retirement_samples = 0;
        self.year_bankruptcy_count = 0;
        self.year_has_activity = false;
    }

    fn record_year_metrics(&mut self) {
        let households = self.households.max(1) as f32;
        self.yearly_avg_net_worth
            .push(self.year_net_worth_sum / households);
        self.yearly_avg_income
            .push(self.year_income_sum / households);
        self.yearly_avg_debt.push(self.year_debt_sum / households);
        self.yearly_avg_spend_ratio
            .push(self.year_spend_ratio_sum / households);
        self.yearly_bankruptcy_rate
            .push(self.year_bankruptcy_count as f32 / households);
        if self.year_retirement_samples > 0 {
            self.yearly_avg_retirement_coverage
                .push(self.year_retirement_coverage_sum / self.year_retirement_samples as f32);
        } else {
            self.yearly_avg_retirement_coverage.push(0.0);
        }
        self.yearly_retirement_samples
            .push(self.year_retirement_samples);
    }

    pub fn fitness_penalty(&self) -> f32 {
        let years = self.yearly_avg_net_worth.len();
        if years == 0 {
            return f32::MAX / 4.0;
        }

        let mut wealth_error = 0.0;
        let mut debt_ratio_error = 0.0;
        let mut spending_error = 0.0;
        let mut bankruptcy_rate = 0.0;
        let mut retirement_error = 0.0;
        let mut retirement_years = 0usize;

        for year in 0..years {
            let target = benchmark_wealth_curve(year).max(1.0);
            wealth_error += (self.yearly_avg_net_worth[year] - target).abs() / target;
            debt_ratio_error +=
                (self.yearly_avg_debt[year] / (self.yearly_avg_income[year] + 1.0)).min(4.0);
            spending_error += (self.yearly_avg_spend_ratio[year] - TARGET_SPEND_RATIO).abs();
            bankruptcy_rate += self.yearly_bankruptcy_rate[year];

            if self.yearly_retirement_samples[year] > 0 {
                retirement_error +=
                    (self.yearly_avg_retirement_coverage[year] - TARGET_RETIREMENT_COVERAGE).abs();
                retirement_years += 1;
            }
        }

        let years_f = years as f32;
        let wealth_error = wealth_error / years_f;
        let debt_ratio_error = debt_ratio_error / years_f;
        let spending_error = spending_error / years_f;
        let bankruptcy_rate = bankruptcy_rate / years_f;
        let retirement_error = if retirement_years > 0 {
            retirement_error / retirement_years as f32
        } else {
            1.0
        };

        wealth_error
            + 3.5 * bankruptcy_rate
            + 1.1 * debt_ratio_error
            + 0.9 * spending_error
            + 1.4 * retirement_error
    }
}

impl fmt::Display for FinanceLifeState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let wealth = self.yearly_avg_net_worth.last().copied().unwrap_or(0.0);
        let debt = self.yearly_avg_debt.last().copied().unwrap_or(0.0);
        let spend = self.yearly_avg_spend_ratio.last().copied().unwrap_or(0.0);
        let bankruptcy = self.yearly_bankruptcy_rate.last().copied().unwrap_or(0.0) * 100.0;

        write!(
            f,
            "run {} year {} wealth {:.0} debt {:.0} spend {:.2} bankrupt {:.1}% policy [{}]",
            self.run_id, self.step, wealth, debt, spend, bankruptcy, self.policy
        )
    }
}

impl State for FinanceLifeState {
    fn reset(&mut self) {
        self.step = 0;
        self.rng = StdRng::seed_from_u64(0x5EED_F17Eu64.wrapping_add(self.run_id * 1_000_003));
        self.reset_history();
        self.reset_year_accumulators();
        self.total_bankruptcies = 0;
        self.total_promotions = 0;
        self.total_shocks = 0;
        self.total_family_events = 0;
        self.total_home_events = 0;
        self.total_inheritances = 0;
        self.total_training_events = 0;
    }

    fn init(&mut self, schedule: &mut Schedule) {
        self.step = 0;
        self.reset_history();
        self.reset_year_accumulators();

        for id in 0..self.households {
            let profile = self.rng.random_range(0..3);
            let age = match profile {
                0 => self.rng.random_range(20..=34),
                1 => self.rng.random_range(28..=46),
                _ => self.rng.random_range(40..=58),
            };
            let retirement_age = self.rng.random_range(60..=68);

            let (income, assets, debt, dependents, skill, homeowner, housing_cost, career_level) =
                match profile {
                    0 => (
                        self.rng.random_range(18_000.0..=34_000.0),
                        self.rng.random_range(0.0..=12_000.0),
                        self.rng.random_range(15_000.0..=80_000.0),
                        self.rng.random_range(0..=1),
                        self.rng.random_range(0.40..=0.85),
                        false,
                        self.rng.random_range(3_500.0..=7_500.0),
                        self.rng.random_range(0..=2),
                    ),
                    1 => (
                        self.rng.random_range(35_000.0..=75_000.0),
                        self.rng.random_range(5_000.0..=55_000.0),
                        self.rng.random_range(0.0..=40_000.0),
                        self.rng.random_range(0..=2),
                        self.rng.random_range(0.70..=1.10),
                        self.rng.random_bool(0.55),
                        self.rng.random_range(4_500.0..=10_000.0),
                        self.rng.random_range(1..=4),
                    ),
                    _ => (
                        self.rng.random_range(70_000.0..=160_000.0),
                        self.rng.random_range(30_000.0..=180_000.0),
                        self.rng.random_range(0.0..=30_000.0),
                        self.rng.random_range(1..=3),
                        self.rng.random_range(0.90..=1.40),
                        self.rng.random_bool(0.82),
                        self.rng.random_range(8_000.0..=18_000.0),
                        self.rng.random_range(2..=5),
                    ),
                };

            let household = Household::new(
                id,
                age,
                retirement_age,
                income,
                assets,
                debt,
                dependents,
                skill,
                homeowner,
                housing_cost,
                career_level,
            );

            schedule.schedule_repeating(Box::new(household), 0.0, id as i32);
        }
    }

    fn update(&mut self, step: u64) {
        self.step = step;
        if self.year_has_activity {
            self.record_year_metrics();
        }
    }

    fn before_step(&mut self, _schedule: &mut Schedule) {
        self.reset_year_accumulators();
    }

    fn after_step(&mut self, _schedule: &mut Schedule) {}

    fn end_condition(&mut self, _schedule: &mut Schedule) -> bool {
        self.yearly_avg_net_worth.len() as u64 >= self.horizon
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn as_state_mut(&mut self) -> &mut dyn State {
        self
    }

    fn as_state(&self) -> &dyn State {
        self
    }
}

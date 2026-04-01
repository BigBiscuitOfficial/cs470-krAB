use super::agent::Person;
use super::config::{Config, RetirementGoal};
use super::demographics::*;
use super::strategies::*;
use super::types::*;
use super::utils::*;
use krabmaga::engine::schedule::Schedule;
use krabmaga::engine::state::State;
use rand::Rng;
use std::any::Any;
use std::sync::Mutex;
use std::time::Instant;

/// Main simulation state containing all agents and global metrics
pub struct FinancialState {
    pub step: u64,
    pub total_steps: u32,
    pub num_agents: u32,
    pub reps: u32,
    pub mode: String,
    pub config: Config,
    pub current_strategy: LifeStrategy,
    pub final_persons: Mutex<Vec<Person>>,

    // Computed metrics
    pub average_net_worth: f32,
    pub median_net_worth: f32,
    pub p10_net_worth: f32,
    pub p90_net_worth: f32,
    pub max_net_worth: f32,
    pub min_net_worth: f32,
    pub gini_coefficient: f32,
    pub bankruptcy_count: u32,
    pub successful_retirement_count: u32,

    pub avg_liquid_cash: f32,
    pub avg_taxable: f32,
    pub avg_401k: f32,
    pub avg_home_equity: f32,
    pub avg_total_debt: f32,

    pub init_time: f32,
    pub step_compute_time: f32,
    pub metrics_calc_time: f32,
    pub run_duration: f32,
    pub communication_overhead: f32,
    pub step_start_time: Option<Instant>,
    pub timeseries: Vec<TimePoint>,
}

impl FinancialState {
    pub fn new(config: Config, strategy: LifeStrategy) -> Self {
        let total_steps = config.simulation.steps;
        let num_agents = config.simulation.num_agents;

        Self {
            step: 0,
            total_steps,
            num_agents,
            reps: 1,
            mode: "run".to_string(),
            config,
            current_strategy: strategy,
            final_persons: Mutex::new(Vec::new()),
            average_net_worth: 0.0,
            median_net_worth: 0.0,
            p10_net_worth: 0.0,
            p90_net_worth: 0.0,
            max_net_worth: 0.0,
            min_net_worth: 0.0,
            gini_coefficient: 0.0,
            bankruptcy_count: 0,
            successful_retirement_count: 0,
            avg_liquid_cash: 0.0,
            avg_taxable: 0.0,
            avg_401k: 0.0,
            avg_home_equity: 0.0,
            avg_total_debt: 0.0,
            init_time: 0.0,
            step_compute_time: 0.0,
            metrics_calc_time: 0.0,
            run_duration: 0.0,
            communication_overhead: 0.0,
            step_start_time: None,
            timeseries: Vec::new(),
        }
    }

    pub fn with_run_context(mut self, reps: u32, mode: &str) -> Self {
        self.reps = reps;
        self.mode = mode.to_string();
        self
    }

    pub fn calculate_income_tax(&self, gross_income: f32) -> f32 {
        let taxable = (gross_income - self.config.tax_system.standard_deduction).max(0.0);
        let brackets = &self.config.tax_system.income_tax_brackets;
        if taxable <= 0.0 || brackets.is_empty() {
            return 0.0;
        }

        let mut tax = 0.0_f32;

        for (idx, bracket) in brackets.iter().enumerate() {
            let upper_bound = brackets
                .get(idx + 1)
                .map(|next| next.threshold)
                .unwrap_or(f32::INFINITY);
            let bracket_income = (taxable.min(upper_bound) - bracket.threshold).max(0.0);
            tax += bracket_income * bracket.rate;
            if taxable <= upper_bound {
                break;
            }
        }
        tax
    }

    pub fn compute_metrics(&mut self) {
        let start = Instant::now();
        let persons = self.final_persons.lock().expect("lock failed").clone();
        if persons.is_empty() {
            return;
        }

        let mut net_worths: Vec<f32> = persons.iter().map(|p| p.net_worth()).collect();
        net_worths.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

        let sum: f32 = net_worths.iter().sum();
        let count = net_worths.len() as f32;
        self.average_net_worth = sum / count;

        let median_idx = net_worths.len() / 2;
        self.median_net_worth = if net_worths.len() % 2 == 0 {
            (net_worths[median_idx - 1] + net_worths[median_idx]) * 0.5
        } else {
            net_worths[median_idx]
        };

        let p10_idx = (net_worths.len() as f32 * 0.10) as usize;
        let p90_idx = (net_worths.len() as f32 * 0.90) as usize;
        self.p10_net_worth = net_worths[p10_idx.min(net_worths.len() - 1)];
        self.p90_net_worth = net_worths[p90_idx.min(net_worths.len() - 1)];

        self.max_net_worth = *net_worths.last().unwrap_or(&0.0);
        self.min_net_worth = *net_worths.first().unwrap_or(&0.0);
        self.bankruptcy_count = net_worths.iter().filter(|&&w| w <= 0.0).count() as u32;

        // Gini calculation with negative wealth handling
        let min_nw = *net_worths.first().unwrap_or(&0.0);
        let shift = if min_nw < 0.0 { -min_nw + 1.0 } else { 0.0 };
        let shifted_sum: f32 = net_worths.iter().map(|w| *w + shift).sum();
        let mut diff_sum = 0.0_f32;
        for (i, nw) in net_worths.iter().enumerate() {
            diff_sum += (i as f32 + 1.0) * (*nw + shift);
        }
        if shifted_sum > 0.0 {
            self.gini_coefficient =
                ((2.0 * diff_sum) / (count * shifted_sum) - (count + 1.0) / count).clamp(0.0, 1.0);
        } else {
            self.gini_coefficient = 0.0;
        }

        // Retirement success
        self.successful_retirement_count = persons
            .iter()
            .filter(|p| {
                if p.retired {
                    let annual_expenses =
                        p.annual_base_expenses * (1.0 + p.num_children as f32 * 0.3);
                    let portfolio = p.portfolio_value();
                    match &p.strategy.retirement_goal {
                        RetirementGoal::Age { target } => {
                            p.age >= *target && portfolio >= annual_expenses * 20.0
                        }
                        RetirementGoal::FIRE { expenses_multiple } => {
                            portfolio >= annual_expenses * expenses_multiple
                        }
                    }
                } else {
                    false
                }
            })
            .count() as u32;

        // Account composition
        self.avg_liquid_cash = persons.iter().map(|p| p.liquid_cash).sum::<f32>() / count;
        self.avg_taxable = persons
            .iter()
            .map(Person::taxable_brokerage_total)
            .sum::<f32>()
            / count;
        self.avg_401k = persons.iter().map(Person::k401_total).sum::<f32>() / count;
        self.avg_home_equity = persons.iter().map(|p| p.home_equity).sum::<f32>() / count;
        self.avg_total_debt = persons.iter().map(|p| p.total_debt()).sum::<f32>() / count;

        self.metrics_calc_time = start.elapsed().as_secs_f32();
    }

    fn snapshot_step_metrics(&mut self, schedule: &Schedule) {
        let events = schedule.get_all_events();
        if events.is_empty() {
            return;
        }
        let mut net_worths = Vec::with_capacity(events.len());
        let mut cash_vals = Vec::new();
        let mut k401_vals = Vec::new();
        let mut equity_vals = Vec::new();
        let mut debt_vals = Vec::new();

        for event in events {
            if let Some(person) = event.downcast_ref::<Person>() {
                net_worths.push(person.net_worth());
                cash_vals.push(person.liquid_cash);
                k401_vals.push(person.k401_total());
                equity_vals.push(person.home_equity);
                debt_vals.push(person.total_debt());
            }
        }
        if net_worths.is_empty() {
            return;
        }
        net_worths.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

        let sum: f32 = net_worths.iter().sum();
        let count = net_worths.len() as f32;
        let avg = sum / count;
        let median_idx = net_worths.len() / 2;
        let med = if net_worths.len() % 2 == 0 {
            (net_worths[median_idx - 1] + net_worths[median_idx]) * 0.5
        } else {
            net_worths[median_idx]
        };

        let p10_idx = (net_worths.len() as f32 * 0.10) as usize;
        let p90_idx = (net_worths.len() as f32 * 0.90) as usize;
        let p10 = net_worths[p10_idx.min(net_worths.len() - 1)];
        let p90 = net_worths[p90_idx.min(net_worths.len() - 1)];

        let bankrupt = net_worths.iter().filter(|&&w| w <= 0.0).count() as u32;

        self.timeseries.push(TimePoint {
            step: self.step,
            average_net_worth: avg,
            median_net_worth: med,
            p10_net_worth: p10,
            p90_net_worth: p90,
            bankruptcy_count: bankrupt,
            average_liquid_cash: cash_vals.iter().sum::<f32>() / count,
            average_401k: k401_vals.iter().sum::<f32>() / count,
            average_home_equity: equity_vals.iter().sum::<f32>() / count,
            average_debt: debt_vals.iter().sum::<f32>() / count,
        });
    }

    pub fn finalize_timing(&mut self, run_duration: f32) {
        self.run_duration = run_duration;
        let pure = self.init_time + self.step_compute_time + self.metrics_calc_time;
        self.communication_overhead = (self.run_duration - pure).max(0.0);
    }

    pub fn to_summary(&self) -> FinancialSummary {
        let strategy_desc = format!(
            "Housing: {:?}, Debt: {:?}, Stocks: {:.0}%, Retirement: {:?}",
            self.current_strategy.housing,
            self.current_strategy.debt_paydown,
            self.current_strategy.asset_allocation.stocks * 100.0,
            self.current_strategy.retirement_goal
        );

        FinancialSummary {
            mode: self.mode.clone(),
            steps: self.total_steps,
            num_agents: self.num_agents,
            reps: self.reps,
            strategy_desc,
            average_net_worth: self.average_net_worth,
            median_net_worth: self.median_net_worth,
            p10_net_worth: self.p10_net_worth,
            p90_net_worth: self.p90_net_worth,
            max_net_worth: self.max_net_worth,
            min_net_worth: self.min_net_worth,
            gini_coefficient: self.gini_coefficient,
            bankruptcy_count: self.bankruptcy_count,
            successful_retirement_count: self.successful_retirement_count,
            avg_liquid_cash: self.avg_liquid_cash,
            avg_taxable: self.avg_taxable,
            avg_401k: self.avg_401k,
            avg_home_equity: self.avg_home_equity,
            avg_total_debt: self.avg_total_debt,
            init_time: self.init_time,
            step_compute_time: self.step_compute_time,
            metrics_calc_time: self.metrics_calc_time,
            run_duration: self.run_duration,
            communication_overhead: self.communication_overhead,
            timeseries: self.timeseries.clone(),
        }
    }
}

impl State for FinancialState {
    fn init(&mut self, schedule: &mut Schedule) {
        let start = Instant::now();
        let mut rng = rand::rng();
        self.final_persons.lock().expect("lock failed").clear();
        self.timeseries.clear();
        self.step = 0;

        for id in 0..self.num_agents {
            let starting_age = rng.random_range(22..35);

            let education_level = {
                let roll = rng.random_range(0.0..1.0);
                if roll < 0.28 {
                    EducationLevel::HighSchool
                } else if roll < 0.45 {
                    EducationLevel::Associates
                } else if roll < 0.77 {
                    EducationLevel::Bachelors
                } else if roll < 0.94 {
                    EducationLevel::Masters
                } else {
                    EducationLevel::PhD
                }
            };

            let gender = {
                let roll = rng.random_range(0.0..1.0);
                if roll < 0.49 {
                    Gender::Male
                } else if roll < 0.98 {
                    Gender::Female
                } else {
                    Gender::NonBinary
                }
            };

            let race_ethnicity = {
                let roll = rng.random_range(0.0..1.0);
                if roll < 0.60 {
                    RaceEthnicity::White
                } else if roll < 0.75 {
                    RaceEthnicity::Black
                } else if roll < 0.93 {
                    RaceEthnicity::Hispanic
                } else if roll < 0.98 {
                    RaceEthnicity::Asian
                } else if roll < 0.99 {
                    RaceEthnicity::NativeAmerican
                } else {
                    RaceEthnicity::MultiRacial
                }
            };

            let geographic_region = {
                let roll = rng.random_range(0.0..1.0);
                if roll < 0.18 {
                    Region::Northeast
                } else if roll < 0.40 {
                    Region::West
                } else if roll < 0.62 {
                    Region::Midwest
                } else if roll < 0.90 {
                    Region::South
                } else {
                    Region::Mountain
                }
            };

            let health_status = {
                let roll = rng.random_range(0.0..1.0);
                if roll < 0.32 {
                    HealthStatus::Excellent
                } else if roll < 0.72 {
                    HealthStatus::Good
                } else if roll < 0.92 {
                    HealthStatus::Fair
                } else {
                    HealthStatus::Poor
                }
            };

            let health_insurance_type = {
                let roll = rng.random_range(0.0..1.0);
                if roll < 0.57 {
                    InsuranceType::EmployerSponsored
                } else if roll < 0.69 {
                    InsuranceType::ACAMarketplace
                } else if roll < 0.82 {
                    InsuranceType::Medicaid
                } else if roll < 0.94 {
                    InsuranceType::Medicare
                } else {
                    InsuranceType::Uninsured
                }
            };

            let career_ambition = {
                let probs = self.config.personality_traits.career_ambition_probabilities;
                let stable_p = probs[0].clamp(0.0, 1.0);
                let moderate_p = probs[1].clamp(0.0, 1.0);
                let cut_stable = stable_p;
                let cut_moderate = (stable_p + moderate_p).clamp(0.0, 1.0);
                let roll = rng.random_range(0.0..1.0);
                if roll < cut_stable {
                    CareerAmbition::Stable
                } else if roll < cut_moderate {
                    CareerAmbition::Moderate
                } else {
                    CareerAmbition::Aggressive
                }
            };

            let chronic_prob = self
                .config
                .healthcare
                .chronic_condition_probability
                .clamp(0.0, 1.0);
            let has_chronic_condition = rng.random_range(0.0..1.0) < chronic_prob;

            let is_disabled = false;

            let education_income_multiplier = map_multiplier_or(
                &self.config.demographics.education_income_multipliers,
                education_level.as_key(),
                1.0,
            );
            let gender_income_multiplier = match gender {
                Gender::Male => 1.0,
                Gender::Female => self.config.demographics.gender_pay_gap_female,
                Gender::NonBinary => self.config.demographics.gender_pay_gap_nonbinary,
            };
            let race_income_multiplier = map_multiplier_or(
                &self.config.demographics.race_wealth_multipliers,
                race_ethnicity.as_key(),
                1.0,
            );
            let region_income_multiplier = map_multiplier_or(
                &self.config.demographics.regional_income_multipliers,
                geographic_region.as_key(),
                1.0,
            );
            let region_col_multiplier = map_multiplier_or(
                &self.config.demographics.regional_col_multipliers,
                geographic_region.as_key(),
                1.0,
            );

            let demographic_income_multiplier = education_income_multiplier
                * gender_income_multiplier
                * race_income_multiplier
                * region_income_multiplier;

            let base_income =
                rng.random_range(35_000.0..95_000.0) * demographic_income_multiplier.max(0.4);
            let is_married = rng.random_range(0.0..1.0) < 0.35;

            let starting_wealth = rng.random_range(5_000.0..75_000.0);
            let student_loan_min = self.config.debt.student_loan_balance_range[0];
            let student_loan_max = self.config.debt.student_loan_balance_range[1];

            // Demographic debt seeding: education strongly drives student debt incidence and size.
            let (student_debt_prob, student_low_mult, student_high_mult) = match education_level {
                EducationLevel::HighSchool => (0.10_f32, 0.0_f32, 0.30_f32),
                EducationLevel::Associates => (0.35_f32, 0.20_f32, 0.65_f32),
                EducationLevel::Bachelors => (0.70_f32, 0.55_f32, 1.00_f32),
                EducationLevel::Masters => (0.82_f32, 0.85_f32, 1.35_f32),
                EducationLevel::PhD => (0.88_f32, 1.00_f32, 1.60_f32),
            };
            let student_loan_debt = if rng.random_range(0.0..1.0) < student_debt_prob {
                let seeded_min = student_loan_min * student_low_mult;
                let seeded_max = student_loan_max * student_high_mult;
                if seeded_max <= seeded_min {
                    seeded_min.max(0.0)
                } else {
                    rng.random_range(seeded_min..seeded_max).max(0.0)
                }
            } else {
                0.0
            };

            // Auto loan seeding: younger workers with stronger income are likelier to carry auto debt.
            let income_factor = (base_income / 120_000.0).clamp(0.0, 1.0);
            let age_factor = ((40.0 - starting_age as f32) / 18.0).clamp(0.0, 1.0);
            let auto_debt_prob =
                (0.18 + income_factor * 0.35 + age_factor * 0.22).clamp(0.08, 0.72);
            let auto_loan_debt = if rng.random_range(0.0..1.0) < auto_debt_prob {
                let auto_min = self.config.debt.auto_loan_balance_range[0];
                let auto_max = self.config.debt.auto_loan_balance_range[1];
                let size_adjustment = (0.70 + income_factor * 0.50).clamp(0.6, 1.3);
                if auto_max <= auto_min {
                    (auto_min * size_adjustment).max(0.0)
                } else {
                    (rng.random_range(auto_min..auto_max) * size_adjustment).max(0.0)
                }
            } else {
                0.0
            };

            let annual_base_expenses =
                base_income * rng.random_range(0.45..0.75) * region_col_multiplier;
            let monthly_rent = (base_income / 12.0)
                * self.config.housing.rent_as_pct_income
                * region_col_multiplier;

            let monthly_health_premium = self
                .config
                .healthcare
                .monthly_premium_ranges
                .get(health_insurance_type.as_key())
                .map(|range| sample_from_range_or(&mut rng, *range, [0.0, 0.0]))
                .unwrap_or_else(|| match health_insurance_type {
                    InsuranceType::EmployerSponsored => 260.0,
                    InsuranceType::ACAMarketplace => 420.0,
                    InsuranceType::Medicaid => 35.0,
                    InsuranceType::Medicare => 190.0,
                    InsuranceType::Uninsured => 0.0,
                });

            let health_cost_multiplier = map_multiplier_or(
                &self.config.healthcare.health_status_cost_multipliers,
                health_status.as_key(),
                match health_status {
                    HealthStatus::Excellent => 0.6,
                    HealthStatus::Good => 1.0,
                    HealthStatus::Fair => 1.8,
                    HealthStatus::Poor => 3.0,
                },
            );
            let annual_healthcare_costs = (self.config.healthcare.annual_healthcare_base_cost
                * health_cost_multiplier)
                + if has_chronic_condition {
                    self.config.healthcare.chronic_condition_annual_cost
                } else {
                    0.0
                };

            let [financial_literacy_mean, financial_literacy_spread, _] = self
                .config
                .personality_traits
                .financial_literacy_distribution;
            let literacy_center = (financial_literacy_mean / 100.0).clamp(0.0, 1.0);
            let literacy_half_width = (financial_literacy_spread / 100.0).abs().clamp(0.0, 1.0);
            let financial_literacy_min = (literacy_center - literacy_half_width).clamp(0.0, 1.0);
            let financial_literacy_max = (literacy_center + literacy_half_width).clamp(0.0, 1.0);
            let financial_literacy_score = if financial_literacy_min >= financial_literacy_max {
                financial_literacy_min
            } else {
                rng.random_range(financial_literacy_min..financial_literacy_max)
            }
            .clamp(0.0, 1.0);
            let spending_discipline = sample_from_range_or(
                &mut rng,
                self.config.personality_traits.spending_discipline_range,
                [0.85, 1.20],
            )
            .clamp(0.6, 1.5);
            let risk_tolerance = map_multiplier_or(
                &self.config.personality_traits.risk_tolerance_by_education,
                education_level.as_key(),
                0.5,
            )
            .clamp(0.0, 1.0);
            let social_network_strength = rng.random_range(0.1_f32..1.0_f32).clamp(0.0, 1.0);
            let family_financial_support = rng.random_range(0.0_f32..10_000.0_f32).max(0.0);

            let ambition_bonus: f32 = match career_ambition {
                CareerAmbition::Stable => -0.002,
                CareerAmbition::Moderate => 0.003,
                CareerAmbition::Aggressive => 0.009,
            };

            let life_insurance_purchase_prob = map_multiplier_or(
                &self.config.life_insurance.purchase_probability_by_education,
                education_level.as_key(),
                0.0,
            )
            .clamp(0.0, 1.0);
            let buys_life_insurance = rng.random_range(0.0..1.0) < life_insurance_purchase_prob;
            let life_insurance_coverage = if buys_life_insurance {
                base_income
                    * if is_married { 1.5 } else { 1.0 }
                    * self
                        .config
                        .life_insurance
                        .coverage_by_income_multiple
                        .max(0.0)
            } else {
                0.0
            };
            let life_insurance_premium = if life_insurance_coverage > 0.0 {
                self.config.life_insurance.annual_premium_per_100k.max(0.0)
                    * (life_insurance_coverage / 100_000.0)
            } else {
                0.0
            };

            // Small stochastic revolving balance at simulation start.
            let discipline_norm = ((spending_discipline - 0.6) / 0.9).clamp(0.0, 1.0);
            let cc_start_prob =
                (0.16 + (1.0 - discipline_norm) * 0.18 + (1.0 - financial_literacy_score) * 0.14)
                    .clamp(0.05, 0.45);
            let credit_card_debt = if rng.random_range(0.0..1.0) < cc_start_prob {
                let raw: f32 = rng.random_range(300.0..3_500.0);
                raw.min(self.config.debt.credit_card_limit * 0.20)
            } else {
                0.0
            };

            let person = Person {
                id,
                age: starting_age,
                starting_age,
                is_married,
                num_children: 0,
                is_disabled,
                unemployed_months: 0,
                retired: false,
                base_income,
                career_growth_rate: (rng.random_range(0.005..0.035) + ambition_bonus).max(0.0),
                years_at_current_job: 0,
                education_level,
                gender,
                race_ethnicity,
                geographic_region,
                health_status,
                health_insurance_type,
                monthly_health_premium,
                annual_healthcare_costs,
                has_chronic_condition,
                financial_literacy_score,
                spending_discipline,
                risk_tolerance,
                career_ambition,
                social_network_strength,
                family_financial_support,
                life_insurance_coverage,
                life_insurance_premium,
                liquid_cash: starting_wealth,
                brokerage_stocks: 0.0,
                brokerage_bonds: 0.0,
                k401_stocks: 0.0,
                k401_bonds: 0.0,
                home_equity: 0.0,
                student_loan_debt,
                auto_loan_debt,
                mortgage_balance: 0.0,
                credit_card_debt,
                owns_home: false,
                home_value: 0.0,
                monthly_rent,
                monthly_mortgage_payment: 0.0,
                strategy: self.current_strategy.clone(),
                annual_base_expenses,
                location_expense_multiplier: 1.0,
            };
            schedule.schedule_repeating(Box::new(person), 0.0, 0);
        }
        self.init_time = start.elapsed().as_secs_f32();
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

    fn reset(&mut self) {
        self.step = 0;
    }

    fn before_step(&mut self, _schedule: &mut Schedule) {
        self.step_start_time = Some(Instant::now());
    }

    fn after_step(&mut self, schedule: &mut Schedule) {
        if let Some(start) = self.step_start_time {
            self.step_compute_time += start.elapsed().as_secs_f32();
        }
        self.snapshot_step_metrics(schedule);
    }

    fn update(&mut self, _step: u64) {
        self.step += 1;
        if self.step == self.total_steps as u64 {
            self.compute_metrics();
        }
    }
}

use crate::model::state::{clamp01, life_stage, FinanceLifeState, LifeStage};
use core::fmt;
use krabmaga::engine::agent::Agent;
use krabmaga::engine::state::State;
use krabmaga::rand::Rng;
use std::hash::{Hash, Hasher};

#[derive(Clone, Copy)]
pub struct Household {
    pub id: u32,
    pub age: u32,
    pub retirement_age: u32,
    pub income: f32,
    pub assets: f32,
    pub debt: f32,
    pub dependents: u32,
    pub skill: f32,
    pub homeowner: bool,
    pub housing_cost: f32,
    pub career_level: u32,
}

impl Household {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: u32,
        age: u32,
        retirement_age: u32,
        income: f32,
        assets: f32,
        debt: f32,
        dependents: u32,
        skill: f32,
        homeowner: bool,
        housing_cost: f32,
        career_level: u32,
    ) -> Self {
        Household {
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
        }
    }
}

impl Agent for Household {
    fn step(&mut self, state: &mut dyn State) {
        let state = state
            .as_any_mut()
            .downcast_mut::<FinanceLifeState>()
            .unwrap();
        state.year_has_activity = true;

        let policy = state.policy;
        let previous_income = self.income;
        self.age = self.age.saturating_add(1);
        let retired = self.age >= self.retirement_age;
        let stage = life_stage(self.age, self.retirement_age);

        if retired {
            self.income = (previous_income * (0.42 + 0.18 * policy.resilience)).max(12_000.0);
        } else {
            let stage_growth = match stage {
                LifeStage::EarlyCareer => 0.05,
                LifeStage::CareerBuilding => 0.04,
                LifeStage::PeakCareer => 0.03,
                LifeStage::Retirement => 0.0,
            };
            let career_growth = policy.career_drive * (0.02 + 0.004 * self.career_level as f32);
            let education_boost = if self.age < 30 {
                policy.education_investment * 0.03
            } else {
                0.0
            };
            let noise = state.rng.random_range(-0.025..=0.05);
            let growth =
                (stage_growth + career_growth + education_boost + self.skill * 0.015 + noise)
                    .clamp(-0.10, 0.20);
            self.income = (previous_income * (1.0 + growth)).max(10_000.0);

            if self.age < 35
                && state
                    .rng
                    .random_bool(clamp01(0.03 + policy.education_investment * 0.12) as f64)
            {
                let training_cost = state.rng.random_range(1_500.0..=6_500.0);
                let effective_cost = training_cost * (0.7 + 0.3 * policy.education_investment);
                if effective_cost <= self.assets {
                    self.assets -= effective_cost;
                } else {
                    self.debt += effective_cost - self.assets;
                    self.assets = 0.0;
                }
                self.skill = (self.skill + 0.03 + policy.education_investment * 0.05).min(2.0);
                self.income *= 1.0 + 0.01 * policy.education_investment;
                state.total_training_events += 1;
            }

            if state
                .rng
                .random_bool(clamp01(0.03 + policy.career_drive * 0.16 + self.skill * 0.01) as f64)
            {
                self.career_level = self.career_level.saturating_add(1);
                self.income *= 1.0 + 0.015 * self.career_level as f32;
                state.total_promotions += 1;
            }

            if state
                .rng
                .random_bool(clamp01(0.02 + (1.0 - policy.career_drive) * 0.04) as f64)
            {
                let setback = state.rng.random_range(0.02..=0.08);
                self.income *= 1.0 - setback;
                state.total_shocks += 1;
            }
        }

        let mut event_cost = 0.0;

        if !retired
            && self.age >= 24
            && state
                .rng
                .random_bool(clamp01(0.02 + policy.family_pressure * 0.08) as f64)
        {
            self.dependents += 1;
            event_cost += 2_000.0 + 700.0 * self.dependents as f32;
            state.total_family_events += 1;
        }

        if !self.homeowner
            && !retired
            && self.age >= 28
            && state
                .rng
                .random_bool(clamp01(0.02 + policy.family_pressure * 0.05) as f64)
        {
            self.homeowner = true;
            self.housing_cost = (self.income * (0.12 + 0.08 * policy.family_pressure)).max(4_000.0);
            self.debt += 130_000.0 * (1.0 - 0.2 * policy.frugality);
            event_cost += 14_000.0;
            state.total_home_events += 1;
        }

        if state
            .rng
            .random_bool(clamp01(0.01 + self.age as f32 / 280.0) as f64)
        {
            let shock = state.rng.random_range(0.0..=1.0) * (6_000.0 + self.age as f32 * 160.0);
            event_cost += shock * (1.0 - 0.5 * policy.resilience);
            state.total_shocks += 1;
        }

        if state
            .rng
            .random_bool(clamp01(0.004 + policy.family_pressure * 0.01) as f64)
        {
            let gift = state.rng.random_range(2_000.0..=20_000.0);
            self.assets += gift;
            state.total_inheritances += 1;
        }

        let debt_interest = self.debt * (0.03 + 0.015 * (1.0 - policy.frugality));
        self.debt += debt_interest;

        let housing = if self.homeowner {
            self.housing_cost
        } else {
            (self.income * 0.18).max(3_500.0)
        };
        let childcare = self.dependents as f32 * (1_500.0 + 750.0 * policy.family_pressure);
        let care_cost = if retired {
            1_500.0 + (self.age - self.retirement_age) as f32 * 180.0
        } else {
            0.0
        };
        let discretionary_ratio = (0.18
            + 0.20 * (1.0 - policy.frugality)
            + if self.age < 30 {
                0.05 * policy.education_investment
            } else {
                0.0
            })
        .clamp(0.10, 0.55);
        let discretionary = self.income * discretionary_ratio;
        let mandatory = housing + childcare + care_cost + event_cost;
        let gross_spend = mandatory + discretionary;
        let spending_ratio = if self.income > 0.0 {
            gross_spend / self.income
        } else {
            1.0
        };

        let leftover = self.income - gross_spend;
        if leftover >= 0.0 {
            let save_rate = (0.10 + 0.50 * policy.savings_discipline + 0.10 * policy.resilience)
                .clamp(0.0, 0.80);
            let reserve = leftover * save_rate;
            let debt_pay = ((leftover - reserve) * (0.25 + 0.35 * policy.frugality)).min(self.debt);
            self.assets += reserve + (leftover - reserve - debt_pay).max(0.0);
            self.debt -= debt_pay;
        } else {
            let shortfall = -leftover;
            if self.assets >= shortfall {
                self.assets -= shortfall;
            } else {
                let unpaid = shortfall - self.assets;
                self.assets = 0.0;
                self.debt += unpaid;
            }
        }

        let market_return = 0.02 + policy.risk_tolerance * 0.06;
        let volatility = 0.015 + policy.risk_tolerance * 0.05;
        let return_rate =
            (market_return + state.rng.random_range(-volatility..=volatility)).clamp(-0.25, 0.25);
        self.assets = (self.assets * (1.0 + return_rate)).max(0.0);

        let net_worth = self.assets - self.debt;
        let retirement_target = (self.income * 9.0).max(80_000.0);
        if retired {
            state.year_retirement_coverage_sum += (self.assets / retirement_target).min(3.0);
            state.year_retirement_samples += 1;
        }

        state.year_net_worth_sum += net_worth;
        state.year_income_sum += self.income;
        state.year_debt_sum += self.debt;
        state.year_spend_ratio_sum += spending_ratio;

        if net_worth < -50_000.0 || self.debt > self.income * 8.0 {
            state.year_bankruptcy_count += 1;
            state.total_bankruptcies += 1;
            self.assets = 0.0;
            self.debt = self.debt.min(self.income * 4.0);
            self.homeowner = false;
            self.housing_cost = (self.income * 0.16).max(3_500.0);
            self.skill = (self.skill * 0.97).max(0.3);
            self.career_level = self.career_level.saturating_sub(1);
        }
    }
}

impl Hash for Household {
    fn hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        self.id.hash(state);
    }
}

impl Eq for Household {}

impl PartialEq for Household {
    fn eq(&self, other: &Household) -> bool {
        self.id == other.id
    }
}

impl fmt::Display for Household {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} age {} wealth {:.0} debt {:.0} homeowner {}",
            self.id,
            self.age,
            self.assets - self.debt,
            self.debt,
            self.homeowner
        )
    }
}

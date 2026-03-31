pub mod config;
pub mod report;

use krabmaga::engine::agent::Agent;
use krabmaga::engine::schedule::Schedule;
use krabmaga::engine::state::State;
use rand::Rng;
use serde::Serialize;
use std::any::Any;
use std::sync::Mutex;
use std::time::Instant;

#[derive(Clone)]
pub struct Person {
    pub age: u32,
    pub wealth: f32,
    pub income: f32,
    pub expenses: f32,
}

#[derive(Debug, Clone, Serialize)]
pub struct TimePoint {
    pub step: u64,
    pub average_wealth: f32,
    pub median_wealth: f32,
    pub bankruptcy_count: u32,
}

#[derive(Debug, Clone, Serialize)]
pub struct FinancialSummary {
    pub mode: String,
    pub steps: u32,
    pub num_agents: u32,
    pub reps: u32,
    pub savings_rate: f32,
    pub risk_profile: f32,
    pub emergency_fund: f32,
    pub average_wealth: f32,
    pub median_wealth: f32,
    pub max_wealth: f32,
    pub min_wealth: f32,
    pub gini_coefficient: f32,
    pub bankruptcy_count: u32,
    pub init_time: f32,
    pub step_compute_time: f32,
    pub metrics_calc_time: f32,
    pub run_duration: f32,
    pub communication_overhead: f32,
    pub timeseries: Vec<TimePoint>,
}

#[derive(Debug, Clone, Serialize)]
pub struct StrategyRunSummary {
    pub savings_rate: f32,
    pub risk_profile: f32,
    pub emergency_fund: f32,
    pub average_wealth: f32,
    pub median_wealth: f32,
    pub max_wealth: f32,
    pub min_wealth: f32,
    pub gini_coefficient: f32,
    pub bankruptcy_count: u32,
    pub init_time: f32,
    pub step_compute_time: f32,
    pub metrics_calc_time: f32,
    pub run_duration: f32,
    pub communication_overhead: f32,
}

impl StrategyRunSummary {
    pub fn from_financial_summary(summary: &FinancialSummary) -> Self {
        Self {
            savings_rate: summary.savings_rate,
            risk_profile: summary.risk_profile,
            emergency_fund: summary.emergency_fund,
            average_wealth: summary.average_wealth,
            median_wealth: summary.median_wealth,
            max_wealth: summary.max_wealth,
            min_wealth: summary.min_wealth,
            gini_coefficient: summary.gini_coefficient,
            bankruptcy_count: summary.bankruptcy_count,
            init_time: summary.init_time,
            step_compute_time: summary.step_compute_time,
            metrics_calc_time: summary.metrics_calc_time,
            run_duration: summary.run_duration,
            communication_overhead: summary.communication_overhead,
        }
    }
}

pub struct FinancialState {
    pub step: u64,
    pub total_steps: u32,
    pub num_agents: u32,
    pub reps: u32,
    pub mode: String,
    pub inflation_rate: f32,
    pub market_return: f32,
    pub job_loss_prob: f32,
    pub savings_rate: f32,
    pub risk_profile: f32,
    pub emergency_fund: f32,
    pub final_wealths: Mutex<Vec<f32>>,
    pub average_wealth: f32,
    pub median_wealth: f32,
    pub max_wealth: f32,
    pub min_wealth: f32,
    pub gini_coefficient: f32,
    pub bankruptcy_count: u32,
    pub init_time: f32,
    pub step_compute_time: f32,
    pub metrics_calc_time: f32,
    pub run_duration: f32,
    pub communication_overhead: f32,
    pub step_start_time: Option<Instant>,
    pub timeseries: Vec<TimePoint>,
}

impl FinancialState {
    pub fn new(
        total_steps: u32,
        inflation_rate: f32,
        market_return: f32,
        job_loss_prob: f32,
        savings_rate: f32,
        risk_profile: f32,
        emergency_fund: f32,
    ) -> Self {
        Self {
            step: 0,
            total_steps,
            num_agents: 1000,
            reps: 1,
            mode: "run".to_string(),
            inflation_rate,
            market_return,
            job_loss_prob,
            savings_rate,
            risk_profile,
            emergency_fund,
            final_wealths: Mutex::new(Vec::new()),
            average_wealth: 0.0,
            median_wealth: 0.0,
            max_wealth: 0.0,
            min_wealth: 0.0,
            gini_coefficient: 0.0,
            bankruptcy_count: 0,
            init_time: 0.0,
            step_compute_time: 0.0,
            metrics_calc_time: 0.0,
            run_duration: 0.0,
            communication_overhead: 0.0,
            step_start_time: None,
            timeseries: Vec::new(),
        }
    }

    pub fn with_run_context(mut self, num_agents: u32, reps: u32, mode: &str) -> Self {
        self.num_agents = num_agents;
        self.reps = reps;
        self.mode = mode.to_string();
        self
    }

    pub fn compute_metrics(&mut self) {
        let start = Instant::now();
        let mut wealths = self.final_wealths.lock().expect("lock failed").clone();
        if wealths.is_empty() {
            return;
        }
        wealths.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let sum: f32 = wealths.iter().sum();
        let count = wealths.len() as f32;
        self.average_wealth = sum / count;
        self.median_wealth = wealths[wealths.len() / 2];
        self.max_wealth = *wealths.last().unwrap_or(&0.0);
        self.min_wealth = *wealths.first().unwrap_or(&0.0);
        self.bankruptcy_count = wealths.iter().filter(|&&w| w <= 0.0).count() as u32;

        let mut diff_sum = 0.0;
        for (i, &yi) in wealths.iter().enumerate() {
            diff_sum += (i as f32 + 1.0) * yi;
        }
        if sum > 0.0 {
            self.gini_coefficient = (2.0 * diff_sum) / (count * sum) - (count + 1.0) / count;
        }
        self.metrics_calc_time = start.elapsed().as_secs_f32();
    }

    fn snapshot_step_metrics(&mut self, schedule: &Schedule) {
        let events = schedule.get_all_events();
        if events.is_empty() {
            return;
        }
        let mut wealths = Vec::with_capacity(events.len());
        for event in events {
            if let Some(person) = event.downcast_ref::<Person>() {
                wealths.push(person.wealth);
            }
        }
        if wealths.is_empty() {
            return;
        }
        wealths.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let sum: f32 = wealths.iter().sum();
        let avg = sum / wealths.len() as f32;
        let med = wealths[wealths.len() / 2];
        let bankrupt = wealths.iter().filter(|&&w| w <= 0.0).count() as u32;
        self.timeseries.push(TimePoint {
            step: self.step,
            average_wealth: avg,
            median_wealth: med,
            bankruptcy_count: bankrupt,
        });
    }

    pub fn finalize_timing(&mut self, run_duration: f32) {
        self.run_duration = run_duration;
        let pure = self.init_time + self.step_compute_time + self.metrics_calc_time;
        self.communication_overhead = (self.run_duration - pure).max(0.0);
    }

    pub fn to_summary(&self) -> FinancialSummary {
        FinancialSummary {
            mode: self.mode.clone(),
            steps: self.total_steps,
            num_agents: self.num_agents,
            reps: self.reps,
            savings_rate: self.savings_rate,
            risk_profile: self.risk_profile,
            emergency_fund: self.emergency_fund,
            average_wealth: self.average_wealth,
            median_wealth: self.median_wealth,
            max_wealth: self.max_wealth,
            min_wealth: self.min_wealth,
            gini_coefficient: self.gini_coefficient,
            bankruptcy_count: self.bankruptcy_count,
            init_time: self.init_time,
            step_compute_time: self.step_compute_time,
            metrics_calc_time: self.metrics_calc_time,
            run_duration: self.run_duration,
            communication_overhead: self.communication_overhead,
            timeseries: self.timeseries.clone(),
        }
    }
}

impl Agent for Person {
    fn step(&mut self, state: &mut dyn State) {
        let state = state
            .as_any()
            .downcast_ref::<FinancialState>()
            .expect("state downcast failed");
        let mut rng = rand::rng();
        self.age += 1;
        self.expenses *= 1.0 + state.inflation_rate;
        self.income *= 1.0 + (state.inflation_rate * 0.8);

        if rng.random_range(0.0..1.0) < state.job_loss_prob {
            self.income = 0.0;
        } else if rng.random_range(0.0..1.0) < 0.05 {
            self.income *= 1.2;
        }
        if rng.random_range(0.0..1.0) < 0.02 {
            self.wealth -= 10_000.0;
        }
        if rng.random_range(0.0..1.0) < 0.01 {
            self.wealth += 50_000.0;
        }

        let target_savings = self.income * state.savings_rate;
        let actual_expenses = self.income - target_savings;
        self.wealth += self.income - actual_expenses;

        if self.wealth > state.emergency_fund {
            let investable = self.wealth - state.emergency_fund;
            let invested = investable * state.risk_profile;
            self.wealth += invested * state.market_return;
        }

        if state.step == (state.total_steps - 1) as u64 {
            let mut wealths = state.final_wealths.lock().expect("lock failed");
            wealths.push(self.wealth);
        }
    }
}

impl State for FinancialState {
    fn init(&mut self, schedule: &mut Schedule) {
        let start = Instant::now();
        let mut rng = rand::rng();
        self.final_wealths.lock().expect("lock failed").clear();
        self.timeseries.clear();
        self.step = 0;
        self.average_wealth = 0.0;
        self.median_wealth = 0.0;
        self.max_wealth = 0.0;
        self.min_wealth = 0.0;
        self.gini_coefficient = 0.0;
        self.bankruptcy_count = 0;
        self.init_time = 0.0;
        self.step_compute_time = 0.0;
        self.metrics_calc_time = 0.0;
        self.run_duration = 0.0;
        self.communication_overhead = 0.0;

        for _id in 0..self.num_agents {
            let person = Person {
                age: rng.random_range(20..60),
                wealth: rng.random_range(1_000.0..50_000.0),
                income: rng.random_range(30_000.0..120_000.0),
                expenses: rng.random_range(20_000.0..80_000.0),
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

use krabmaga::engine::agent::Agent;
use krabmaga::engine::schedule::Schedule;
use krabmaga::engine::state::State;
use rand::Rng;
use std::any::Any;
use std::sync::Mutex;

#[derive(Clone)]
pub struct Person {
    pub id: u32,
    pub age: u32,
    pub wealth: f32,
    pub income: f32,
    pub expenses: f32,
    pub risk_profile: f32,
}

impl Agent for Person {
    fn step(&mut self, state: &mut dyn State) {
        let state = state.as_any().downcast_ref::<FinancialState>().unwrap();
        let mut rng = rand::rng();

        self.age += 1;
        self.expenses *= 1.0 + state.inflation_rate;
        self.income *= 1.0 + (state.inflation_rate * 0.8);

        // Life events
        if rng.random_range(0.0..1.0) < state.job_loss_prob {
            self.income = 0.0;
        } else if rng.random_range(0.0..1.0) < 0.05 {
            self.income *= 1.2;
        }

        if rng.random_range(0.0..1.0) < 0.02 {
            self.wealth -= 10000.0;
        }
        if rng.random_range(0.0..1.0) < 0.01 {
            self.wealth += 50000.0;
        }

        let net = self.income - self.expenses;
        self.wealth += net;

        let invested = self.wealth * self.risk_profile;
        let return_amount = invested * state.market_return;
        self.wealth += return_amount;

        // Collect stats on the final step
        if state.step == 99 {
            let mut wealths = state.final_wealths.lock().unwrap();
            wealths.push(self.wealth);
        }
    }
}

pub struct FinancialState {
    pub step: u64,
    pub num_agents: u32,
    pub inflation_rate: f32,
    pub market_return: f32,
    pub job_loss_prob: f32,
    pub deterministic: u32,

    pub final_wealths: Mutex<Vec<f32>>,

    pub average_wealth: f32,
    pub median_wealth: f32,
    pub max_wealth: f32,
    pub min_wealth: f32,
    pub gini_coefficient: f32,
    pub bankruptcy_count: u32,
}

impl FinancialState {
    pub fn new(inflation_rate: f32, market_return: f32, job_loss_prob: f32) -> Self {
        FinancialState {
            step: 0,
            num_agents: 1000,
            inflation_rate,
            market_return,
            job_loss_prob,
            deterministic: 0,
            final_wealths: Mutex::new(Vec::new()),
            average_wealth: 0.0,
            median_wealth: 0.0,
            max_wealth: 0.0,
            min_wealth: 0.0,
            gini_coefficient: 0.0,
            bankruptcy_count: 0,
        }
    }

    pub fn compute_metrics(&mut self) {
        let mut wealths = self.final_wealths.lock().unwrap().clone();
        if wealths.is_empty() {
            return;
        }

        wealths.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let sum: f32 = wealths.iter().sum();
        let count = wealths.len() as f32;

        self.average_wealth = sum / count;
        self.median_wealth = wealths[wealths.len() / 2];
        self.max_wealth = *wealths.last().unwrap();
        self.min_wealth = *wealths.first().unwrap();
        self.bankruptcy_count = wealths.iter().filter(|&&w| w <= 0.0).count() as u32;

        // Gini coefficient
        let mut diff_sum = 0.0;
        for (i, &yi) in wealths.iter().enumerate() {
            diff_sum += (i as f32 + 1.0) * yi;
        }
        if sum > 0.0 {
            self.gini_coefficient = (2.0 * diff_sum) / (count * sum) - (count + 1.0) / count;
        }
    }
}

impl State for FinancialState {
    fn init(&mut self, schedule: &mut Schedule) {
        let mut rng = rand::rng();
        for id in 0..self.num_agents {
            let person = Person {
                id,
                age: rng.random_range(20..60),
                wealth: rng.random_range(1000.0..50000.0),
                income: rng.random_range(30000.0..120000.0),
                expenses: rng.random_range(20000.0..80000.0),
                risk_profile: rng.random_range(0.1..0.9),
            };
            schedule.schedule_repeating(Box::new(person), 0.0, 0);
        }
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

    fn update(&mut self, _step: u64) {
        self.step += 1;
        if self.step == 100 {
            self.compute_metrics();
        }
    }
}

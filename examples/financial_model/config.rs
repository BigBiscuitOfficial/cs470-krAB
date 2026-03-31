use serde::Deserialize;
use std::fs;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub macro_economics: MacroEconomics,
    pub strategy_sweeps: StrategySweeps,
    pub simulation: Simulation,
    pub output: Option<OutputConfig>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MacroEconomics {
    pub inflation_rate: f32,
    pub market_return: f32,
    pub job_loss_prob: f32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct StrategySweeps {
    pub savings_rates: Vec<f32>,
    pub risk_profiles: Vec<f32>,
    pub emergency_funds: Vec<f32>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Simulation {
    pub steps: u32,
    pub num_agents: u32,
    pub reps: u32,
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

    pub fn output_base_dir(&self) -> &str {
        self.output
            .as_ref()
            .and_then(|o| o.base_dir.as_deref())
            .unwrap_or("output")
    }
}

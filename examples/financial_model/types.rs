use serde::Serialize;

/// Time-series snapshot of aggregate metrics at a specific simulation step
#[derive(Debug, Clone, Serialize)]
pub struct TimePoint {
    pub step: u64,
    pub average_net_worth: f32,
    pub median_net_worth: f32,
    pub p10_net_worth: f32,
    pub p90_net_worth: f32,
    pub bankruptcy_count: u32,
    pub average_liquid_cash: f32,
    pub average_401k: f32,
    pub average_home_equity: f32,
    pub average_debt: f32,
}

/// Complete financial simulation results and metrics
#[derive(Debug, Clone, Serialize)]
pub struct FinancialSummary {
    pub mode: String,
    pub steps: u32,
    pub num_agents: u32,
    pub reps: u32,
    pub strategy_desc: String,

    // Final outcomes
    pub average_net_worth: f32,
    pub median_net_worth: f32,
    pub p10_net_worth: f32,
    pub p90_net_worth: f32,
    pub max_net_worth: f32,
    pub min_net_worth: f32,
    pub gini_coefficient: f32,
    pub bankruptcy_count: u32,
    pub successful_retirement_count: u32,

    // Account composition at end
    pub avg_liquid_cash: f32,
    pub avg_taxable: f32,
    pub avg_401k: f32,
    pub avg_home_equity: f32,
    pub avg_total_debt: f32,

    // Timing
    pub init_time: f32,
    pub step_compute_time: f32,
    pub metrics_calc_time: f32,
    pub run_duration: f32,
    pub communication_overhead: f32,

    pub timeseries: Vec<TimePoint>,
}

/// Condensed summary of a strategy run for comparison
#[derive(Debug, Clone, Serialize)]
pub struct StrategyRunSummary {
    pub strategy_desc: String,
    pub median_net_worth: f32,
    pub p10_net_worth: f32,
    pub p90_net_worth: f32,
    pub bankruptcy_rate: f32,
    pub successful_retirement_rate: f32,
    pub avg_liquid_cash: f32,
    pub avg_401k: f32,
    pub avg_home_equity: f32,
    pub avg_total_debt: f32,
    pub run_duration: f32,
}

impl StrategyRunSummary {
    pub fn from_financial_summary(summary: &FinancialSummary) -> Self {
        Self {
            strategy_desc: summary.strategy_desc.clone(),
            median_net_worth: summary.median_net_worth,
            p10_net_worth: summary.p10_net_worth,
            p90_net_worth: summary.p90_net_worth,
            bankruptcy_rate: summary.bankruptcy_count as f32 / summary.num_agents as f32,
            successful_retirement_rate: summary.successful_retirement_count as f32
                / summary.num_agents as f32,
            avg_liquid_cash: summary.avg_liquid_cash,
            avg_401k: summary.avg_401k,
            avg_home_equity: summary.avg_home_equity,
            avg_total_debt: summary.avg_total_debt,
            run_duration: summary.run_duration,
        }
    }
}

/// Breakdown of withdrawals from different account types
#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct WithdrawalBreakdown {
    pub from_cash: f32,
    pub from_brokerage_stocks: f32,
    pub from_brokerage_bonds: f32,
    pub from_k401_stocks: f32,
    pub from_k401_bonds: f32,
}

impl WithdrawalBreakdown {
    pub fn total(self) -> f32 {
        self.from_cash
            + self.from_brokerage_stocks
            + self.from_brokerage_bonds
            + self.from_k401_stocks
            + self.from_k401_bonds
    }

    pub fn brokerage_withdrawals(self) -> f32 {
        self.from_brokerage_stocks + self.from_brokerage_bonds
    }

    pub fn k401_withdrawals(self) -> f32 {
        self.from_k401_stocks + self.from_k401_bonds
    }
}

use super::config::{AssetAllocation, RetirementGoal};

/// Housing strategy options for agents
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HousingStrategy {
    Rent,
    Buy,
}

/// Debt paydown strategy options for agents
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DebtStrategy {
    Minimum,
    Aggressive,
}

/// Complete life strategy for an agent
#[derive(Debug, Clone)]
pub struct LifeStrategy {
    pub housing: HousingStrategy,
    pub debt_paydown: DebtStrategy,
    pub asset_allocation: AssetAllocation,
    pub retirement_goal: RetirementGoal,
}

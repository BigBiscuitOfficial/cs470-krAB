use super::config::{Config, RetirementGoal};
use super::{DebtStrategy, HousingStrategy, LifeStrategy};

fn parse_housing_strategy(s: &str) -> HousingStrategy {
    match s.to_lowercase().as_str() {
        "rent" => HousingStrategy::Rent,
        "buy" => HousingStrategy::Buy,
        _ => panic!("Unknown housing strategy: {}", s),
    }
}

fn parse_debt_strategy(s: &str) -> DebtStrategy {
    match s.to_lowercase().as_str() {
        "minimum" => DebtStrategy::Minimum,
        "aggressive" => DebtStrategy::Aggressive,
        _ => panic!("Unknown debt strategy: {}", s),
    }
}

pub(crate) fn generate_strategies(config: &Config) -> Vec<LifeStrategy> {
    let mut strategies = Vec::new();

    for housing_str in &config.strategy_sweeps.housing_strategies {
        for debt_str in &config.strategy_sweeps.debt_strategies {
            for asset_alloc in &config.strategy_sweeps.asset_allocations {
                for retirement_goal in &config.strategy_sweeps.retirement_goals {
                    strategies.push(LifeStrategy {
                        housing: parse_housing_strategy(housing_str),
                        debt_paydown: parse_debt_strategy(debt_str),
                        asset_allocation: asset_alloc.clone(),
                        retirement_goal: retirement_goal.clone(),
                    });
                }
            }
        }
    }

    strategies
}

pub(crate) fn strategy_description(strategy: &LifeStrategy) -> String {
    let housing = match strategy.housing {
        HousingStrategy::Rent => "Rent",
        HousingStrategy::Buy => "Buy",
    };
    let debt = match strategy.debt_paydown {
        DebtStrategy::Minimum => "MinDebt",
        DebtStrategy::Aggressive => "AggDebt",
    };
    let alloc = format!(
        "{}%stocks/{}%bonds",
        (strategy.asset_allocation.stocks * 100.0) as u32,
        (strategy.asset_allocation.bonds * 100.0) as u32
    );
    let retire = match &strategy.retirement_goal {
        RetirementGoal::Age { target } => format!("Age{}", target),
        RetirementGoal::FIRE { expenses_multiple } => {
            format!("FIRE{}x", (*expenses_multiple) as u32)
        }
    };

    format!("{} | {} | {} | {}", housing, debt, alloc, retire)
}

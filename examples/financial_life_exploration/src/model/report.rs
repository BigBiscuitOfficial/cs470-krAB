use crate::model::state::{
    FinanceRunSummary, TARGET_RETIREMENT_COVERAGE, TARGET_SPEND_RATIO,
};

pub struct PolicyNarrative {
    pub profile: String,
}

pub struct MetricNarrative {
    pub core_reason: String,
}

pub struct EventNarrative {
    pub career_story: String,
    pub risk_story: String,
    pub household_story: String,
}

impl PolicyNarrative {
    pub fn from_summary(summary: &FinanceRunSummary) -> Self {
        let policy = summary.policy;
        let mut descriptors = Vec::new();
        descriptors.push(label(policy.frugality, "very frugal", "balanced spender", "free spender"));
        descriptors.push(label(
            policy.savings_discipline,
            "aggressive saver",
            "steady saver",
            "light saver",
        ));
        descriptors.push(label(
            policy.career_drive,
            "career-focused",
            "career-balanced",
            "career-cautious",
        ));
        descriptors.push(label(
            policy.risk_tolerance,
            "risk-seeking",
            "moderate risk",
            "risk-averse",
        ));
        descriptors.push(label(
            policy.resilience,
            "shock-ready",
            "moderately resilient",
            "shock-sensitive",
        ));

        Self {
            profile: descriptors.join(", "),
        }
    }
}

impl MetricNarrative {
    pub fn from_summary(summary: &FinanceRunSummary) -> Self {
        let wealth_clause = if summary.final_avg_net_worth > 250_000.0 {
            "it built strong long-term wealth"
        } else if summary.final_avg_net_worth > 100_000.0 {
            "it kept wealth growth positive"
        } else {
            "it limited wealth collapse better than the alternatives"
        };

        let debt_clause = if summary.final_avg_debt <= summary.final_avg_income * 0.8 {
            "while keeping debt under control"
        } else if summary.final_avg_debt <= summary.final_avg_income * 1.5 {
            "while keeping debt manageable"
        } else {
            "despite carrying meaningful debt pressure"
        };

        let spending_gap = (summary.final_spend_ratio - TARGET_SPEND_RATIO).abs();
        let spending_clause = if spending_gap <= 0.08 {
            "spending stayed close to the target"
        } else if summary.final_spend_ratio < TARGET_SPEND_RATIO {
            "spending stayed below the target"
        } else {
            "the policy accepted higher spending to support other gains"
        };

        let bankruptcy_clause = if summary.final_bankruptcy_rate <= 0.05 {
            "bankruptcy stayed low"
        } else if summary.final_bankruptcy_rate <= 0.12 {
            "bankruptcy stayed contained"
        } else {
            "bankruptcy was the main weakness"
        };

        let retirement_clause = if summary.final_retirement_coverage >= TARGET_RETIREMENT_COVERAGE {
            "retirement coverage met the benchmark"
        } else if summary.final_retirement_coverage >= 0.8 {
            "retirement coverage stayed reasonably close to target"
        } else {
            "retirement coverage lagged the target"
        };

        Self {
            core_reason: format!(
                "{}, {}, {}, and {}. {}.",
                wealth_clause, debt_clause, spending_clause, bankruptcy_clause, retirement_clause
            ),
        }
    }
}

impl EventNarrative {
    pub fn from_summary(summary: &FinanceRunSummary) -> Self {
        let career_story = if summary.total_promotions >= summary.total_training_events.saturating_mul(2).max(4) {
            format!(
                "promotions were frequent ({} total), so income growth came mostly from career advancement rather than extra training.",
                summary.total_promotions
            )
        } else if summary.total_training_events > 0 {
            format!(
                "the policy invested in training {} times and converted that into {} promotions, which suggests deliberate skill building.",
                summary.total_training_events, summary.total_promotions
            )
        } else {
            format!(
                "income gains came from baseline career growth, with {} promotions and almost no extra education push.",
                summary.total_promotions
            )
        };

        let risk_story = if summary.total_shocks == 0 {
            "the run saw almost no adverse shocks, so balance sheet discipline mattered more than recovery behavior.".to_string()
        } else if summary.total_bankruptcies == 0 {
            format!(
                "the cohort absorbed {} shocks without triggering bankruptcies, which points to resilient cash-flow management.",
                summary.total_shocks
            )
        } else if summary.total_bankruptcies < summary.total_shocks / 4 {
            format!(
                "there were {} shocks and only {} bankruptcies, so most setbacks were absorbed without a full financial reset.",
                summary.total_shocks, summary.total_bankruptcies
            )
        } else {
            format!(
                "{} shocks translated into {} bankruptcies, so the winning policy likely beat others by surviving instability better, not by avoiding it.",
                summary.total_shocks, summary.total_bankruptcies
            )
        };

        let household_story = if summary.total_home_events > 0 || summary.total_family_events > 0 {
            format!(
                "it handled real life-cycle costs: {} family expansions, {} home purchases, and {} inheritances shaped the final balance sheet.",
                summary.total_family_events, summary.total_home_events, summary.total_inheritances
            )
        } else {
            format!(
                "household expansion stayed quiet, so the result was driven more by saving and career behavior than by major family or housing changes; inheritances occurred {} times.",
                summary.total_inheritances
            )
        };

        Self {
            career_story,
            risk_story,
            household_story,
        }
    }
}

fn label(value: f32, high: &str, mid: &str, low: &str) -> String {
    if value >= 0.67 {
        high.to_string()
    } else if value >= 0.34 {
        mid.to_string()
    } else {
        low.to_string()
    }
}

// Module declarations
pub mod agent;
pub mod config;
pub mod demographics;

pub mod profiling;
pub mod report;
/// HTML rendering helpers for financial example reports.
pub mod report_html;
pub mod runner;
pub mod state;
mod strategies;
pub mod strategy_utils;
pub mod summary_aggregation;
pub mod types;
pub mod utils;

// Re-exports for public API
#[allow(unused_imports)]
pub use agent::Person;
#[allow(unused_imports)]
pub use config::{AssetAllocation, Config, RetirementGoal};
#[allow(unused_imports)]
pub use demographics::*;
pub use state::FinancialState;
pub use strategies::*;
pub use types::*;

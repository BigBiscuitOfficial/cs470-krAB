// Module declarations
pub mod agent;
pub mod config;
pub mod demographics;
pub mod report;
pub mod runner;
pub mod state;
mod strategies;
pub mod types;
mod utils;

// Re-exports for public API
pub use agent::Person;
pub use config::{AssetAllocation, Config, RetirementGoal};
pub use demographics::*;
pub use state::FinancialState;
pub use strategies::*;
pub use types::*;

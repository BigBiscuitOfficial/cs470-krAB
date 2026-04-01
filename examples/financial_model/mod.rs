// Module declarations
pub mod agent;
pub mod config;
pub mod demographics;
#[cfg(feature = "distributed_mpi")]
pub mod mpi_utils;
pub mod partitioning;
pub mod profiling;
pub mod report;
pub mod runner;
pub mod state;
mod strategies;
pub mod types;
mod utils;

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

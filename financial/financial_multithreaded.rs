mod financial_model;

#[cfg(feature = "parallel")]
use financial_model::config::Config;
#[cfg(feature = "parallel")]
use financial_model::report::write_and_print_headless_sweep;
#[cfg(feature = "parallel")]
use financial_model::runner::{run_headless, ExecutionMode};

#[cfg(feature = "parallel")]
fn main() {
    let config_path = std::env::var("KRAB_CONFIG_PATH")
        .unwrap_or_else(|_| "examples/config_comprehensive.json".to_string());
    let config = Config::read_from(&config_path);
    let summaries = run_headless(&config, ExecutionMode::Multithreaded);

    let _ = write_and_print_headless_sweep(&config, "multithreaded", &summaries);
}

#[cfg(not(feature = "parallel"))]
fn main() {
    println!("Please enable the 'parallel' feature to run this example.");
}

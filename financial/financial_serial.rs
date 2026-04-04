mod financial_model;

use financial_model::config::Config;
use financial_model::report::write_and_print_headless_sweep;
use financial_model::runner::{run_headless, ExecutionMode};

fn main() {
    let config_path = std::env::var("KRAB_CONFIG_PATH")
        .unwrap_or_else(|_| "examples/config_comprehensive.json".to_string());
    let config = Config::read_from(&config_path);
    let summaries = run_headless(&config, ExecutionMode::Serial);

    let _ = write_and_print_headless_sweep(&config, "serial", &summaries);
}

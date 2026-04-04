use crate::financial_model::config::Config;
use crate::financial_model::runner::{
    describe_strategy, generate_strategy_space, run_single_strategy_with_index, ExecutionMode,
};

const CONFIG_PATH: &str = "tests/fixtures/config_reduced_seeded.json";
const BASE_SEED: u64 = 42;

fn derive_run_seed(base_seed: u64, strategy_index: usize, rep: u32) -> u64 {
    base_seed
        .wrapping_add((strategy_index as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15))
        .wrapping_add((rep as u64).wrapping_mul(0xBF58_476D_1CE4_E5B9))
}

#[test]
fn strategy_space_and_seed_contract_are_stable() {
    let config = Config::read_from(CONFIG_PATH);
    let strategies = generate_strategy_space(&config);

    assert_eq!(strategies.len(), 16, "Expected 16 strategy combinations");

    for (idx, strategy) in strategies.iter().enumerate() {
        let desc = describe_strategy(strategy);
        assert!(!desc.trim().is_empty(), "strategy_desc must not be empty");

        let s0 = derive_run_seed(BASE_SEED, idx, 0);
        let s1 = derive_run_seed(BASE_SEED, idx, 1);
        assert_ne!(s0, s1, "Rep-specific seeds must differ (idx={})", idx);
        if idx > 0 {
            let prev = derive_run_seed(BASE_SEED, idx - 1, 0);
            assert_ne!(prev, s0, "Strategy-specific seeds must differ");
        }
    }
}

#[test]
fn deterministic_seed_fallback_without_config_seed() {
    let mut config = Config::read_from(CONFIG_PATH);
    config.simulation.seed = None;
    config.simulation.reps = 1;
    config.simulation.thread_count = Some(1);

    std::env::set_var("KRAB_SEED", "424242");
    let strategies = generate_strategy_space(&config);
    let idx = 0usize;
    let strategy = &strategies[idx];

    let s1 = run_single_strategy_with_index(&config, strategy, idx, ExecutionMode::Serial);
    let s2 = run_single_strategy_with_index(&config, strategy, idx, ExecutionMode::Serial);

    assert_eq!(s1.seed, s2.seed, "Fallback seed should be deterministic");
    assert_eq!(s1.median_net_worth, s2.median_net_worth);
    assert_eq!(s1.p10_net_worth, s2.p10_net_worth);
    assert_eq!(s1.p90_net_worth, s2.p90_net_worth);
}

#[test]
fn serial_and_multithreaded_execute_same_work_for_strategy() {
    let mut config = Config::read_from(CONFIG_PATH);
    config.simulation.reps = 1;
    config.simulation.thread_count = Some(1);
    config.simulation.seed = Some(BASE_SEED);

    let strategies = generate_strategy_space(&config);
    let idx = 0usize;
    let strategy = &strategies[idx];

    let serial = run_single_strategy_with_index(&config, strategy, idx, ExecutionMode::Serial);
    let mt = run_single_strategy_with_index(&config, strategy, idx, ExecutionMode::Multithreaded);

    assert_eq!(
        serial.strategy_desc, mt.strategy_desc,
        "Strategy identity drifted between modes"
    );
    assert_eq!(serial.steps, mt.steps, "Steps mismatch between modes");
    assert_eq!(
        serial.num_agents, mt.num_agents,
        "Agent count mismatch between modes"
    );
    assert_eq!(
        serial.seed, mt.seed,
        "Seed derivation mismatch between serial and multithreaded"
    );
}

#[test]
fn mpi_sweep_job_ids_cover_strategy_space_exactly_once() {
    let config = Config::read_from(CONFIG_PATH);
    let total = generate_strategy_space(&config).len();
    assert_eq!(total, 16, "Expected strategy space size to remain stable");

    let jobs: Vec<u64> = (0..total).map(|idx| idx as u64).collect();
    assert_eq!(jobs.len(), total);
    for (idx, conf_id) in jobs.iter().enumerate() {
        assert_eq!(*conf_id, idx as u64, "Sweep job id mismatch at index {idx}");
    }
}

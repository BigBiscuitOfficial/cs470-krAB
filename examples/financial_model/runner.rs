use super::config::Config;
use super::profiling::{PerformanceProfile, ProfileContext, TimingEvent};
use super::strategy_utils::{generate_strategies, strategy_description};
use super::summary_aggregation::aggregate_summaries;
use super::{FinancialState, FinancialSummary, LifeStrategy};
use krabmaga::engine::run::{run_initialized_state, RunStats};
use krabmaga::engine::schedule::Schedule;
use krabmaga::engine::state::State;
use krabmaga::explore::local_sweep::{sweep_serial, SweepJob};
use std::env;
use std::time::Instant;

#[cfg(feature = "parallel")]
use krabmaga::explore::local_sweep::sweep_parallel_with_threads;

/// Headless strategy-sweep orchestration for the financial example.
///
/// This module keeps model execution on engine abstractions (`run_initialized_state`,
/// `sweep_serial`, `sweep_parallel_with_threads`) and isolates feature-gated
/// multithreading behavior in dedicated helper functions.

#[derive(Copy, Clone)]
pub enum ExecutionMode {
    Serial,
    Multithreaded,
}

const DEFAULT_BASE_SEED: u64 = 0x5EED_5EED_F00D_BAAD;

impl ExecutionMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            ExecutionMode::Serial => "serial",
            ExecutionMode::Multithreaded => "multithreaded",
        }
    }
}

fn configured_thread_count(config: &Config) -> usize {
    if let Ok(raw) = env::var("KRAB_THREAD_COUNT") {
        if let Ok(parsed) = raw.parse::<usize>() {
            return parsed.max(1);
        }
    }
    config.simulation.thread_count.unwrap_or(1).max(1)
}

fn derive_run_seed(base_seed: u64, strategy_index: usize, rep: u32) -> u64 {
    base_seed
        .wrapping_add((strategy_index as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15))
        .wrapping_add((rep as u64).wrapping_mul(0xBF58_476D_1CE4_E5B9))
}

fn configured_base_seed(config: &Config) -> u64 {
    if let Some(seed) = config.simulation.seed {
        return seed;
    }

    if let Ok(raw) = env::var("KRAB_SEED") {
        if let Ok(parsed) = raw.trim().parse::<u64>() {
            return parsed;
        }
    }

    DEFAULT_BASE_SEED
}

fn build_schedule(requested_threads: usize) -> Schedule {
    #[cfg(feature = "parallel")]
    {
        Schedule::with_threads(requested_threads.max(1))
    }
    #[cfg(not(feature = "parallel"))]
    {
        let _ = requested_threads;
        Schedule::new()
    }
}

pub fn generate_strategy_space(config: &Config) -> Vec<LifeStrategy> {
    generate_strategies(config)
}

pub fn describe_strategy(strategy: &LifeStrategy) -> String {
    strategy_description(strategy)
}

fn run_single_strategy_profiled(
    config: &Config,
    strategy: &LifeStrategy,
    strategy_index: Option<usize>,
    strategy_desc: &str,
    mode: ExecutionMode,
    mut profiler: Option<&mut PerformanceProfile>,
) -> FinancialSummary {
    let reps = config.simulation.reps.max(1);
    let base_seed = configured_base_seed(config);
    let threads = configured_thread_count(config);
    let mut rep_summaries: Vec<FinancialSummary> = Vec::with_capacity(reps as usize);
    let strategy_timer = Instant::now();

    for rep in 0..reps {
        let mut per_rep_config = config.clone();
        let idx = strategy_index.unwrap_or(0);
        per_rep_config.simulation.seed = Some(derive_run_seed(base_seed, idx, rep));

        // MT_OPT_BEGIN: avoid nested parallelism
        // Multithreaded mode parallelizes across independent strategies.
        // Keep each strategy simulation single-threaded to avoid oversubscription,
        // preserve comparable work, and improve determinism.
        if matches!(mode, ExecutionMode::Multithreaded) {
            per_rep_config.simulation.thread_count = Some(1);
        }
        // MT_OPT_END: avoid nested parallelism

        let mut state = FinancialState::new(per_rep_config, strategy.clone())
            .with_run_context(reps, mode.as_str());

        let mut schedule = match mode {
            ExecutionMode::Serial => Schedule::new(),
            ExecutionMode::Multithreaded => build_schedule(threads),
        };

        state.init(&mut schedule);
        let run_stats = run_initialized_state(state.as_state_mut(), &mut schedule);
        state.finalize_timing(run_stats.run_duration);

        if let Some(profile) = profiler.as_deref_mut() {
            let _ = rep;
            profile.record(
                TimingEvent::Init,
                strategy_index,
                strategy_desc,
                state.init_time,
                0.0,
                0.0,
                0.0,
                state.init_time,
            );
            profile.record(
                TimingEvent::StepCompute,
                strategy_index,
                strategy_desc,
                0.0,
                state.step_compute_time,
                0.0,
                0.0,
                state.step_compute_time,
            );
            profile.record(
                TimingEvent::MetricsCalc,
                strategy_index,
                strategy_desc,
                0.0,
                0.0,
                0.0,
                state.metrics_calc_time,
                state.metrics_calc_time,
            );
            profile.record(
                TimingEvent::RunDuration,
                strategy_index,
                strategy_desc,
                0.0,
                0.0,
                0.0,
                0.0,
                state.run_duration,
            );
            profile.record(
                TimingEvent::CommunicationOverhead,
                strategy_index,
                strategy_desc,
                0.0,
                0.0,
                state.communication_overhead,
                0.0,
                state.communication_overhead,
            );
        }

        rep_summaries.push(state.to_summary());
    }

    let mut aggregated = aggregate_summaries(mode.as_str(), &rep_summaries);
    aggregated.run_duration = strategy_timer.elapsed().as_secs_f32();
    let pure = aggregated.init_time + aggregated.step_compute_time + aggregated.metrics_calc_time;
    aggregated.communication_overhead = (aggregated.run_duration - pure).max(0.0);

    if let Some(profile) = profiler {
        profile.record(
            TimingEvent::StrategyTotal,
            strategy_index,
            strategy_desc,
            aggregated.init_time,
            aggregated.step_compute_time,
            aggregated.communication_overhead,
            aggregated.metrics_calc_time,
            aggregated.run_duration,
        );
    }

    aggregated
}

pub fn run_single_strategy_with_index(
    config: &Config,
    strategy: &LifeStrategy,
    strategy_index: usize,
    mode: ExecutionMode,
) -> FinancialSummary {
    let desc = strategy_description(strategy);
    run_single_strategy_profiled(config, strategy, Some(strategy_index), &desc, mode, None)
}

#[derive(Clone)]
struct StrategySweepTask {
    index: usize,
    strategy: LifeStrategy,
    desc: String,
}

fn run_sweep_serial(
    config: &Config,
    jobs: &[SweepJob<StrategySweepTask>],
    profile: &mut PerformanceProfile,
) -> Vec<FinancialSummary> {
    let records = sweep_serial(jobs, |job| {
        let task = &job.config;
        println!("  [{}/{}] {}", task.index + 1, jobs.len(), task.desc);
        let mut local_profile = PerformanceProfile::new();
        let summary = run_single_strategy_profiled(
            config,
            &task.strategy,
            Some(task.index),
            &task.desc,
            ExecutionMode::Serial,
            Some(&mut local_profile),
        );
        (
            (summary.clone(), local_profile),
            RunStats {
                run_duration: summary.run_duration,
                executed_steps: summary.steps as u64,
            },
        )
    });

    let mut serial_results = Vec::with_capacity(records.len());
    for record in records {
        let (summary, local_profile) = record.result;
        profile.merge_from(local_profile);
        serial_results.push(summary);
    }
    serial_results
}

#[cfg(feature = "parallel")]
fn run_sweep_multithreaded(
    config: &Config,
    jobs: &[SweepJob<StrategySweepTask>],
    num_threads: usize,
    profile: &mut PerformanceProfile,
) -> Vec<FinancialSummary> {
    let mut records = sweep_parallel_with_threads(jobs, num_threads, |job| {
        let task = &job.config;
        let mut local_profile = PerformanceProfile::new();
        let summary = run_single_strategy_profiled(
            config,
            &task.strategy,
            Some(task.index),
            &task.desc,
            ExecutionMode::Multithreaded,
            Some(&mut local_profile),
        );
        (
            (summary.clone(), local_profile),
            RunStats {
                run_duration: summary.run_duration,
                executed_steps: summary.steps as u64,
            },
        )
    });

    records.sort_by_key(|record| record.config.index);

    let mut mt_results = Vec::with_capacity(records.len());
    for record in records {
        let idx = record.config.index;
        println!("  [{}/{}] {}", idx + 1, jobs.len(), record.config.desc);
        let (summary, local_profile) = record.result;
        profile.merge_from(local_profile);
        mt_results.push(summary);
    }
    mt_results
}

#[cfg(not(feature = "parallel"))]
fn run_sweep_multithreaded(
    _config: &Config,
    _jobs: &[SweepJob<StrategySweepTask>],
    _num_threads: usize,
    _profile: &mut PerformanceProfile,
) -> Vec<FinancialSummary> {
    panic!("ExecutionMode::Multithreaded requires building with --features parallel")
}

pub fn run_headless(config: &Config, mode: ExecutionMode) -> Vec<FinancialSummary> {
    let strategies = generate_strategies(config);

    if strategies.is_empty() {
        panic!("No strategies generated from config. Check strategy_sweeps configuration.");
    }

    println!("Running {} strategy combinations...", strategies.len());

    let jobs: Vec<SweepJob<StrategySweepTask>> = strategies
        .into_iter()
        .enumerate()
        .map(|(idx, strategy)| {
            let desc = strategy_description(&strategy);
            SweepJob {
                conf_id: idx as u64,
                rep_id: 0,
                config: StrategySweepTask {
                    index: idx,
                    strategy,
                    desc,
                },
            }
        })
        .collect();

    let sweep_timer = Instant::now();
    let num_threads = match mode {
        ExecutionMode::Serial => 1,
        ExecutionMode::Multithreaded => configured_thread_count(config),
    };
    let profile_context = ProfileContext::new(
        mode.as_str(),
        config.simulation.num_agents,
        config.simulation.steps,
        config.simulation.reps,
        num_threads,
        Some(configured_base_seed(config)),
    );
    let mut profile = PerformanceProfile::new();
    let results = match mode {
        ExecutionMode::Serial => run_sweep_serial(config, &jobs, &mut profile),
        ExecutionMode::Multithreaded => {
            run_sweep_multithreaded(config, &jobs, num_threads, &mut profile)
        }
    };

    profile.record(
        TimingEvent::SweepTotal,
        None,
        "all_strategies",
        0.0,
        0.0,
        0.0,
        0.0,
        sweep_timer.elapsed().as_secs_f32(),
    );

    let profiling_base = format!("{}/profiling", config.output_base_dir());
    match profile.export_csv_in_dir(&profiling_base, &profile_context) {
        Ok(path) => println!("- profiling metrics: {}", path),
        Err(err) => eprintln!("Warning: unable to write profiling CSV: {}", err),
    }

    results
}

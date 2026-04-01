use super::config::{Config, RetirementGoal};
use super::profiling::{PerformanceProfile, ProfileContext, TimingEvent};
use super::{
    DebtStrategy, FinancialState, FinancialSummary, HousingStrategy, LifeStrategy, TimePoint,
};
use krabmaga::engine::schedule::Schedule;
use krabmaga::engine::state::State;
use std::env;
use std::time::Instant;

#[cfg(feature = "parallel")]
use rayon::prelude::*;

#[derive(Copy, Clone)]
pub enum ExecutionMode {
    Serial,
    Multithreaded,
}

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

fn generate_strategies(config: &Config) -> Vec<LifeStrategy> {
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

fn strategy_description(strategy: &LifeStrategy) -> String {
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

pub fn generate_strategy_space(config: &Config) -> Vec<LifeStrategy> {
    generate_strategies(config)
}

pub fn describe_strategy(strategy: &LifeStrategy) -> String {
    strategy_description(strategy)
}

fn aggregate_timeseries(all_series: &[Vec<TimePoint>]) -> Vec<TimePoint> {
    let max_len = all_series.iter().map(|s| s.len()).max().unwrap_or(0);
    let mut merged = Vec::with_capacity(max_len);

    for step_idx in 0..max_len {
        let mut avg_nw = 0.0_f32;
        let mut med_nw = 0.0_f32;
        let mut p10_nw = 0.0_f32;
        let mut p90_nw = 0.0_f32;
        let mut bankrupt = 0_u32;
        let mut liq = 0.0_f32;
        let mut k401 = 0.0_f32;
        let mut home_eq = 0.0_f32;
        let mut debt = 0.0_f32;
        let mut sample_count = 0_u32;
        let mut step = step_idx as u64;

        for series in all_series {
            if let Some(point) = series.get(step_idx) {
                step = point.step;
                avg_nw += point.average_net_worth;
                med_nw += point.median_net_worth;
                p10_nw += point.p10_net_worth;
                p90_nw += point.p90_net_worth;
                bankrupt += point.bankruptcy_count;
                liq += point.average_liquid_cash;
                k401 += point.average_401k;
                home_eq += point.average_home_equity;
                debt += point.average_debt;
                sample_count += 1;
            }
        }

        if sample_count > 0 {
            let sc = sample_count as f32;
            merged.push(TimePoint {
                step,
                average_net_worth: avg_nw / sc,
                median_net_worth: med_nw / sc,
                p10_net_worth: p10_nw / sc,
                p90_net_worth: p90_nw / sc,
                bankruptcy_count: (bankrupt as f32 / sc).round() as u32,
                average_liquid_cash: liq / sc,
                average_401k: k401 / sc,
                average_home_equity: home_eq / sc,
                average_debt: debt / sc,
            });
        }
    }

    merged
}

fn aggregate_summaries(mode: &str, summaries: &[FinancialSummary]) -> FinancialSummary {
    let first = summaries
        .first()
        .expect("Expected at least one summary for aggregation");
    let count = summaries.len() as f32;

    let mut avg_nw = 0.0_f32;
    let mut med_nw = 0.0_f32;
    let mut p10_nw = 0.0_f32;
    let mut p90_nw = 0.0_f32;
    let mut max_nw = f32::MIN;
    let mut min_nw = f32::MAX;
    let mut gini = 0.0_f32;
    let mut bankruptcies = 0_u32;
    let mut successful_retirements = 0_u32;
    let mut liq = 0.0_f32;
    let mut taxable = 0.0_f32;
    let mut k401 = 0.0_f32;
    let mut home_eq = 0.0_f32;
    let mut total_debt = 0.0_f32;
    let mut init_time = 0.0_f32;
    let mut step_time = 0.0_f32;
    let mut metrics_time = 0.0_f32;
    let mut run_duration = 0.0_f32;
    let mut overhead = 0.0_f32;
    let mut series = Vec::with_capacity(summaries.len());

    for s in summaries {
        avg_nw += s.average_net_worth;
        med_nw += s.median_net_worth;
        p10_nw += s.p10_net_worth;
        p90_nw += s.p90_net_worth;
        max_nw = max_nw.max(s.max_net_worth);
        min_nw = min_nw.min(s.min_net_worth);
        gini += s.gini_coefficient;
        bankruptcies += s.bankruptcy_count;
        successful_retirements += s.successful_retirement_count;
        liq += s.avg_liquid_cash;
        taxable += s.avg_taxable;
        k401 += s.avg_401k;
        home_eq += s.avg_home_equity;
        total_debt += s.avg_total_debt;
        init_time += s.init_time;
        step_time += s.step_compute_time;
        metrics_time += s.metrics_calc_time;
        run_duration += s.run_duration;
        overhead += s.communication_overhead;
        series.push(s.timeseries.clone());
    }

    FinancialSummary {
        mode: mode.to_string(),
        steps: first.steps,
        num_agents: first.num_agents,
        reps: first.reps,
        seed: first.seed,
        strategy_desc: first.strategy_desc.clone(),
        average_net_worth: avg_nw / count,
        median_net_worth: med_nw / count,
        p10_net_worth: p10_nw / count,
        p90_net_worth: p90_nw / count,
        max_net_worth: max_nw,
        min_net_worth: min_nw,
        gini_coefficient: (gini / count).clamp(0.0, 1.0),
        bankruptcy_count: (bankruptcies as f32 / count).round() as u32,
        successful_retirement_count: (successful_retirements as f32 / count).round() as u32,
        avg_liquid_cash: liq / count,
        avg_taxable: taxable / count,
        avg_401k: k401 / count,
        avg_home_equity: home_eq / count,
        avg_total_debt: total_debt / count,
        init_time: init_time / count,
        step_compute_time: step_time / count,
        metrics_calc_time: metrics_time / count,
        run_duration: run_duration / count,
        communication_overhead: overhead / count,
        timeseries: aggregate_timeseries(&series),
    }
}

fn run_single_strategy_profiled(
    config: &Config,
    strategy: &LifeStrategy,
    strategy_index: Option<usize>,
    strategy_desc: &str,
    mode: ExecutionMode,
    mut profiler: Option<&mut PerformanceProfile>,
) -> FinancialSummary {
    let steps = config.simulation.steps.max(1);
    let reps = config.simulation.reps.max(1);
    let threads = configured_thread_count(config);
    let mut rep_summaries: Vec<FinancialSummary> = Vec::with_capacity(reps as usize);
    let strategy_timer = Instant::now();

    for rep in 0..reps {
        let mut per_rep_config = config.clone();
        if let Some(base_seed) = config.simulation.seed {
            let idx = strategy_index.unwrap_or(0);
            per_rep_config.simulation.seed = Some(derive_run_seed(base_seed, idx, rep));
        }

        let mut state = FinancialState::new(per_rep_config, strategy.clone())
            .with_run_context(reps, mode.as_str());

        let mut schedule = match mode {
            ExecutionMode::Serial => Schedule::new(),
            ExecutionMode::Multithreaded => build_schedule(threads),
        };

        let timer = Instant::now();
        state.init(&mut schedule);
        for _ in 0..steps {
            schedule.step(state.as_state_mut());
            if state.end_condition(&mut schedule) {
                break;
            }
        }

        state.finalize_timing(timer.elapsed().as_secs_f32());

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

pub fn run_single_strategy(
    config: &Config,
    strategy: &LifeStrategy,
    mode: ExecutionMode,
) -> FinancialSummary {
    let desc = strategy_description(strategy);
    run_single_strategy_profiled(config, strategy, None, &desc, mode, None)
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

pub fn run_headless(config: &Config, mode: ExecutionMode) -> Vec<FinancialSummary> {
    let strategies = generate_strategies(config);

    if strategies.is_empty() {
        panic!("No strategies generated from config. Check strategy_sweeps configuration.");
    }

    println!("Running {} strategy combinations...", strategies.len());

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
        config.simulation.seed,
    );
    let mut profile = PerformanceProfile::new();
    let results = match mode {
        ExecutionMode::Serial => {
            let mut serial_results = Vec::with_capacity(strategies.len());
            for (idx, strategy) in strategies.iter().enumerate() {
                let desc = strategy_description(strategy);
                println!("  [{}/{}] {}", idx + 1, strategies.len(), desc);
                let summary = run_single_strategy_profiled(
                    config,
                    strategy,
                    Some(idx),
                    &desc,
                    mode,
                    Some(&mut profile),
                );
                serial_results.push(summary);
            }
            serial_results
        }
        ExecutionMode::Multithreaded => {
            #[cfg(feature = "parallel")]
            {
                let mut indexed_results: Vec<(usize, FinancialSummary, PerformanceProfile)> =
                    strategies
                        .par_iter()
                        .enumerate()
                        .map(|(idx, strategy)| {
                            let desc = strategy_description(strategy);
                            let mut local_profile = PerformanceProfile::new();
                            let mut summary = run_single_strategy_profiled(
                                config,
                                strategy,
                                Some(idx),
                                &desc,
                                ExecutionMode::Serial,
                                Some(&mut local_profile),
                            );
                            summary.mode = ExecutionMode::Multithreaded.as_str().to_string();
                            (idx, summary, local_profile)
                        })
                        .collect();

                indexed_results.sort_by_key(|(idx, _, _)| *idx);

                let mut mt_results = Vec::with_capacity(indexed_results.len());
                for (idx, summary, local_profile) in indexed_results {
                    println!(
                        "  [{}/{}] {}",
                        idx + 1,
                        strategies.len(),
                        strategy_description(&strategies[idx])
                    );
                    profile.merge_from(local_profile);
                    mt_results.push(summary);
                }
                mt_results
            }
            #[cfg(not(feature = "parallel"))]
            {
                let mut fallback_results = Vec::with_capacity(strategies.len());
                for (idx, strategy) in strategies.iter().enumerate() {
                    let desc = strategy_description(strategy);
                    println!("  [{}/{}] {}", idx + 1, strategies.len(), desc);
                    let summary = run_single_strategy_profiled(
                        config,
                        strategy,
                        Some(idx),
                        &desc,
                        mode,
                        Some(&mut profile),
                    );
                    fallback_results.push(summary);
                }
                fallback_results
            }
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

use crate::engine::run::RunStats;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};

/// One sweep work item.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SweepJob<C> {
    pub conf_id: u64,
    pub rep_id: u32,
    pub config: C,
}

/// One recorded sweep result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SweepRecord<C, R> {
    pub conf_id: u64,
    pub rep_id: u32,
    pub config: C,
    pub result: R,
    pub stats: RunStats,
}

/// Execute sweep jobs sequentially.
pub fn sweep_serial<C, R, F>(jobs: &[SweepJob<C>], run: F) -> Vec<SweepRecord<C, R>>
where
    C: Clone,
    F: Fn(&SweepJob<C>) -> (R, RunStats),
{
    jobs.iter()
        .map(|job| {
            let (result, stats) = run(job);
            SweepRecord {
                conf_id: job.conf_id,
                rep_id: job.rep_id,
                config: job.config.clone(),
                result,
                stats,
            }
        })
        .collect()
}

/// Execute sweep jobs in parallel using rayon.
pub fn sweep_parallel<C, R, F>(jobs: &[SweepJob<C>], run: F) -> Vec<SweepRecord<C, R>>
where
    C: Clone + Send + Sync,
    R: Send,
    F: Fn(&SweepJob<C>) -> (R, RunStats) + Send + Sync,
{
    jobs.par_iter()
        .map(|job| {
            let (result, stats) = run(job);
            SweepRecord {
                conf_id: job.conf_id,
                rep_id: job.rep_id,
                config: job.config.clone(),
                result,
                stats,
            }
        })
        .collect()
}

/// Execute sweep jobs in a dedicated rayon pool.
#[cfg(feature = "parallel")]
pub fn sweep_parallel_with_threads<C, R, F>(
    jobs: &[SweepJob<C>],
    num_threads: usize,
    run: F,
) -> Vec<SweepRecord<C, R>>
where
    C: Clone + Send + Sync,
    R: Send,
    F: Fn(&SweepJob<C>) -> (R, RunStats) + Send + Sync,
{
    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(num_threads.max(1))
        .build()
        .expect("Failed to build rayon thread pool for multithreaded sweep");

    pool.install(|| sweep_parallel(jobs, run))
}

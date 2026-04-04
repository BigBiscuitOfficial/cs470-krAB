use crate::engine::run::RunStats;
use crate::explore::local_sweep::{SweepJob, SweepRecord};
use mpi::collective::SystemOperation;
use mpi::topology::Communicator;
use mpi::traits::*;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::time::Instant;

#[derive(Debug, Clone)]
pub enum SweepMpiError {
    InvalidRootRank { root_rank: i32, world_size: i32 },
    Serialize(String),
    Deserialize(String),
}

#[derive(Debug, Clone, Copy, Default)]
pub struct SweepMpiSubroutineTimings {
    pub local_execute_s: f64,
    pub serialize_local_s: f64,
    pub collective_transfer_s: f64,
    pub deserialize_remote_s: f64,
    pub sort_records_s: f64,
    pub sweep_total_s: f64,
}

fn contiguous_bounds(total_items: usize, rank: i32, world_size: i32) -> (usize, usize) {
    let world_size = world_size as usize;
    let rank = rank as usize;
    let base = total_items / world_size;
    let rem = total_items % world_size;

    let start = rank * base + rank.min(rem);
    let size = base + usize::from(rank < rem);
    (start, start + size)
}

/// Deterministically sort sweep records by configuration/repetition identifiers.
pub fn sort_records_by_job_id<C, R>(records: &mut [SweepRecord<C, R>]) {
    records.sort_by_key(|r| (r.conf_id, r.rep_id));
}

/// Execute sweep jobs across MPI ranks and gather full ordered results at root.
///
/// Each rank processes a contiguous subset of `jobs`, serializes local records,
/// and sends them to `root_rank`. Root returns all gathered records sorted by
/// `(conf_id, rep_id)` for deterministic downstream processing.
///
/// # Example
/// ```ignore
/// use krabmaga::explore::local_sweep::SweepJob;
/// use krabmaga::explore::mpi::sweep::sweep_mpi;
///
/// // world: mpi communicator
/// let jobs = vec![
///     SweepJob { conf_id: 0, rep_id: 0, config: 0usize },
///     SweepJob { conf_id: 1, rep_id: 0, config: 1usize },
/// ];
///
/// let gathered = sweep_mpi(&world, &jobs, 0, |job| {
///     // run one configuration and return (result, stats)
///     let result = job.config as i32;
///     let stats = krabmaga::engine::run::RunStats { run_duration: 0.0, executed_steps: 0 };
///     (result, stats)
/// })?;
/// ```
pub fn sweep_mpi<C, R, F, W>(
    world: &W,
    jobs: &[SweepJob<C>],
    root_rank: i32,
    mut run: F,
) -> Result<Option<Vec<SweepRecord<C, R>>>, SweepMpiError>
where
    C: Clone + Serialize + DeserializeOwned,
    R: Serialize + DeserializeOwned,
    F: FnMut(&SweepJob<C>) -> (R, RunStats),
    W: Communicator,
{
    let rank = world.rank();
    let world_size = world.size();

    if root_rank < 0 || root_rank >= world_size {
        return Err(SweepMpiError::InvalidRootRank {
            root_rank,
            world_size,
        });
    }

    let (start, end) = contiguous_bounds(jobs.len(), rank, world_size);
    let mut local_records: Vec<SweepRecord<C, R>> = Vec::with_capacity(end.saturating_sub(start));

    for job in &jobs[start..end] {
        let (result, stats) = run(job);
        local_records.push(SweepRecord {
            conf_id: job.conf_id,
            rep_id: job.rep_id,
            config: job.config.clone(),
            result,
            stats,
        });
    }

    let local_payload =
        serde_json::to_vec(&local_records).map_err(|e| SweepMpiError::Serialize(e.to_string()))?;

    if rank == root_rank {
        let mut all_records = local_records;
        for src in 0..world_size {
            if src == root_rank {
                continue;
            }
            let (payload, _status) = world.process_at_rank(src).receive_vec::<u8>();
            let mut remote: Vec<SweepRecord<C, R>> = serde_json::from_slice(&payload)
                .map_err(|e| SweepMpiError::Deserialize(e.to_string()))?;
            all_records.append(&mut remote);
        }

        sort_records_by_job_id(&mut all_records);
        Ok(Some(all_records))
    } else {
        world.process_at_rank(root_rank).send(&local_payload[..]);
        Ok(None)
    }
}

/// MPI sweep variant that returns collective (max-reduced) timing totals
/// for each major sweep subroutine.
pub fn sweep_mpi_with_timings<C, R, F, W>(
    world: &W,
    jobs: &[SweepJob<C>],
    root_rank: i32,
    mut run: F,
) -> Result<(Option<Vec<SweepRecord<C, R>>>, SweepMpiSubroutineTimings), SweepMpiError>
where
    C: Clone + Serialize + DeserializeOwned,
    R: Serialize + DeserializeOwned,
    F: FnMut(&SweepJob<C>) -> (R, RunStats),
    W: Communicator + CommunicatorCollectives,
{
    let rank = world.rank();
    let world_size = world.size();

    if root_rank < 0 || root_rank >= world_size {
        return Err(SweepMpiError::InvalidRootRank {
            root_rank,
            world_size,
        });
    }

    let sweep_start = Instant::now();
    let (start, end) = contiguous_bounds(jobs.len(), rank, world_size);
    let mut local_records: Vec<SweepRecord<C, R>> = Vec::with_capacity(end.saturating_sub(start));

    let local_execute_start = Instant::now();
    for job in &jobs[start..end] {
        let (result, stats) = run(job);
        local_records.push(SweepRecord {
            conf_id: job.conf_id,
            rep_id: job.rep_id,
            config: job.config.clone(),
            result,
            stats,
        });
    }
    let local_execute_s = local_execute_start.elapsed().as_secs_f64();

    let serialize_start = Instant::now();
    let local_payload =
        serde_json::to_vec(&local_records).map_err(|e| SweepMpiError::Serialize(e.to_string()))?;
    let serialize_local_s = serialize_start.elapsed().as_secs_f64();

    let transfer_start = Instant::now();
    let mut deserialize_remote_s = 0.0f64;
    let mut sort_records_s = 0.0f64;

    let gathered = if rank == root_rank {
        let mut all_records = local_records;
        for src in 0..world_size {
            if src == root_rank {
                continue;
            }
            let (payload, _status) = world.process_at_rank(src).receive_vec::<u8>();
            let deserialize_start = Instant::now();
            let mut remote: Vec<SweepRecord<C, R>> = serde_json::from_slice(&payload)
                .map_err(|e| SweepMpiError::Deserialize(e.to_string()))?;
            deserialize_remote_s += deserialize_start.elapsed().as_secs_f64();
            all_records.append(&mut remote);
        }

        let sort_start = Instant::now();
        sort_records_by_job_id(&mut all_records);
        sort_records_s = sort_start.elapsed().as_secs_f64();
        Some(all_records)
    } else {
        world.process_at_rank(root_rank).send(&local_payload[..]);
        None
    };

    let collective_transfer_s = transfer_start.elapsed().as_secs_f64();
    let sweep_total_s = sweep_start.elapsed().as_secs_f64();

    let mut local_val = local_execute_s;
    let mut max_local_execute_s = 0.0;
    world.all_reduce_into(&local_val, &mut max_local_execute_s, SystemOperation::max());

    local_val = serialize_local_s;
    let mut max_serialize_local_s = 0.0;
    world.all_reduce_into(
        &local_val,
        &mut max_serialize_local_s,
        SystemOperation::max(),
    );

    local_val = collective_transfer_s;
    let mut max_collective_transfer_s = 0.0;
    world.all_reduce_into(
        &local_val,
        &mut max_collective_transfer_s,
        SystemOperation::max(),
    );

    local_val = deserialize_remote_s;
    let mut max_deserialize_remote_s = 0.0;
    world.all_reduce_into(
        &local_val,
        &mut max_deserialize_remote_s,
        SystemOperation::max(),
    );

    local_val = sort_records_s;
    let mut max_sort_records_s = 0.0;
    world.all_reduce_into(&local_val, &mut max_sort_records_s, SystemOperation::max());

    local_val = sweep_total_s;
    let mut max_sweep_total_s = 0.0;
    world.all_reduce_into(&local_val, &mut max_sweep_total_s, SystemOperation::max());

    Ok((
        gathered,
        SweepMpiSubroutineTimings {
            local_execute_s: max_local_execute_s,
            serialize_local_s: max_serialize_local_s,
            collective_transfer_s: max_collective_transfer_s,
            deserialize_remote_s: max_deserialize_remote_s,
            sort_records_s: max_sort_records_s,
            sweep_total_s: max_sweep_total_s,
        },
    ))
}

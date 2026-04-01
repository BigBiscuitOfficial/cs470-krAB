use super::StrategyRunSummary;
use mpi::collective::SystemOperation;
use mpi::topology::Communicator;
use mpi::traits::*;

/// Placeholder error type for first-wave MPI scaffolding.
#[derive(Debug, Clone)]
pub enum MpiScaffoldError {
    NotImplemented(&'static str),
}

/// Root-oriented gather API for per-rank strategy summaries.
///
/// Uses JSON byte payloads for pragmatic, robust variable-length transport.
pub fn gather_strategy_summaries_root<C: Communicator>(
    world: &C,
    root_rank: i32,
    local_runs: &[StrategyRunSummary],
) -> Result<Option<Vec<StrategyRunSummary>>, MpiScaffoldError> {
    let rank = world.rank();
    let local_payload = serde_json::to_vec(local_runs)
        .map_err(|_| MpiScaffoldError::NotImplemented("serialize local strategy summaries"))?;

    if rank == root_rank {
        let mut combined = Vec::new();

        let mut local_decoded: Vec<StrategyRunSummary> = serde_json::from_slice(&local_payload)
            .map_err(|_| MpiScaffoldError::NotImplemented("decode local strategy summaries"))?;
        combined.append(&mut local_decoded);

        for src in 0..world.size() {
            if src == root_rank {
                continue;
            }
            let (payload, _status) = world.process_at_rank(src).receive_vec::<u8>();
            let mut runs: Vec<StrategyRunSummary> =
                serde_json::from_slice(&payload).map_err(|_| {
                    MpiScaffoldError::NotImplemented("decode remote strategy summaries")
                })?;
            combined.append(&mut runs);
        }

        Ok(Some(combined))
    } else {
        world.process_at_rank(root_rank).send(&local_payload[..]);
        Ok(None)
    }
}

/// All-ranks reduction API for selecting a globally best score.
///
pub fn allreduce_best_score<C: Communicator>(
    world: &C,
    local_score: f32,
) -> Result<f32, MpiScaffoldError> {
    let mut global_best = f32::NEG_INFINITY;
    world.all_reduce_into(&local_score, &mut global_best, SystemOperation::max());
    Ok(global_best)
}

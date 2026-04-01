use super::config::Config;

/// Contiguous work assignment for one MPI rank.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RankWorkload {
    pub start: usize,
    pub end: usize,
}

impl RankWorkload {
    pub fn len(self) -> usize {
        self.end.saturating_sub(self.start)
    }

    pub fn is_empty(self) -> bool {
        self.start >= self.end
    }
}

/// Computes strategy sweep size from the configured Cartesian product.
pub fn strategy_space_size(config: &Config) -> usize {
    config
        .strategy_sweeps
        .housing_strategies
        .len()
        .saturating_mul(config.strategy_sweeps.debt_strategies.len())
        .saturating_mul(config.strategy_sweeps.asset_allocations.len())
        .saturating_mul(config.strategy_sweeps.retirement_goals.len())
}

/// Assigns a contiguous chunk of `[0, total_items)` to a rank.
pub fn contiguous_rank_workload(total_items: usize, rank: i32, world_size: i32) -> RankWorkload {
    assert!(world_size > 0, "world_size must be > 0");
    assert!(rank >= 0 && rank < world_size, "rank out of bounds");

    let world_size = world_size as usize;
    let rank = rank as usize;
    let base = total_items / world_size;
    let rem = total_items % world_size;

    let start = rank * base + rank.min(rem);
    let size = base + usize::from(rank < rem);
    RankWorkload {
        start,
        end: start + size,
    }
}

/// Convenience helper to clone only local items for this rank.
pub fn local_items_for_rank<T: Clone>(items: &[T], rank: i32, world_size: i32) -> Vec<T> {
    let r = contiguous_rank_workload(items.len(), rank, world_size);
    items[r.start..r.end].to_vec()
}

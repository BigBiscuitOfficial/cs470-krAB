# MPI Design Foundation (P0)

## Scope

This document defines the P0 MPI architecture for the financial strategy sweep in:

- `examples/financial_mpi.rs`
- `examples/financial_model/runner.rs`
- `examples/financial_model/partitioning.rs`
- `examples/financial_model/mpi_utils.rs`

Goal: distribute strategy combinations across ranks while preserving apples-to-apples comparability with serial/multithreaded modes.

---

## Current repo integration points (verified)

- `examples/financial_mpi.rs`: currently MPI-initialized, but sweep logic is TODO.
- `examples/financial_model/partitioning.rs`: already contains contiguous partition helpers:
  - `strategy_space_size`
  - `contiguous_rank_workload`
  - `local_items_for_rank`
- `examples/financial_model/mpi_utils.rs`: scaffold placeholders exist:
  - `gather_strategy_summaries_root`
  - `allreduce_best_score`
- `examples/financial_model/runner.rs`:
  - owns canonical strategy generation (`generate_strategies`, currently private)
  - runs one strategy (`run_single_strategy`)
  - currently uses non-deterministic RNG path (`FinancialState::new` + `rand::rng()` in state/agents)

---

## Design overview

1. **Partition by strategy index** (not by agent):
   - Every rank builds the same ordered strategy vector.
   - Each rank computes its contiguous `[start, end)` range using `contiguous_rank_workload`.
   - Rank executes only its local strategy indices.

2. **Deterministic seeding is rank-independent**:
   - Seed derivation must depend on `(global_seed, strategy_idx, rep_idx, agent_id)`.
   - **Must not include rank** in final simulation randomness.
   - This guarantees comparable outputs regardless of rank count.

3. **Gather results on root**:
   - Each rank sends fixed-width numeric result rows keyed by `strategy_idx`.
   - Root reconstructs `StrategyRunSummary` in canonical strategy order and writes artifacts.

4. **Root computes best strategy** using existing `score()` in `financial_mpi.rs`.

---

## Rank partitioning algorithm

Use existing helper in `partitioning.rs`:

`contiguous_rank_workload(total_items, rank, world_size) -> RankWorkload {start, end}`

Properties:

- Full coverage of `[0, total_items)`.
- No overlap between ranks.
- First `remainder` ranks get one extra item.
- Works when `world_size > total_items` (some ranks receive empty ranges).

This is sufficient for P0 and has predictable communication shape.

---

## Deterministic seeding scheme

## Inputs

- `global_seed: u64` (from env, e.g. `KRAB_BASE_SEED`, default constant)
- `strategy_idx: u32`
- `rep_idx: u32`
- `agent_id: u32`

## Derivation

Define in `partitioning.rs` (or `mpi_utils.rs`) a pure function:

```text
seed = mix64(global_seed ^ (strategy_idx << 32) ^ (rep_idx << 16) ^ agent_id)
```

where `mix64` is a stable integer mixer (SplitMix64-style).

### Critical constraint

- Do **not** include rank in this formula.

### Required integration hooks

- `FinancialState` should carry deterministic seed context (at minimum per-rep base seed).
- Replace `rand::rng()` creation sites with seeded RNG construction path.
- `runner.rs` must pass `(strategy_idx, rep_idx)` into state/run initialization.

---

## Communication pattern and wire layout

## Pattern

P0 uses **gather-to-root** (not all-to-all):

1. (Optional) Root broadcasts `global_seed` and `total_strategies`.
2. Each rank computes local strategy results.
3. Two-phase gather:
   - gather local row counts (`u32`) to root
   - `gatherv` fixed-width result rows to root
4. Root sorts by `strategy_idx`, reconstructs full sweep, computes best, writes artifacts.

## Wire row (fixed width)

Use a numeric-only DTO for MPI transport:

```text
StrategyResultWire {
  strategy_idx: u32,
  median_net_worth: f32,
  p10_net_worth: f32,
  p90_net_worth: f32,
  bankruptcy_rate: f32,
  successful_retirement_rate: f32,
  avg_liquid_cash: f32,
  avg_401k: f32,
  avg_home_equity: f32,
  avg_total_debt: f32,
  run_duration: f32,
}
```

Rationale:

- Avoid variable-length strings in MPI payload.
- Root reconstructs `strategy_desc` from canonical strategy list/index.

---

## Pseudocode

## Rank 0

```text
init MPI
world_size, rank
assert rank == 0

config = Config::read_from("examples/config_comprehensive.json")
strategies = runner::generate_strategies(config)          // make accessible from runner
total = strategies.len()
global_seed = read_env_or_default("KRAB_BASE_SEED")

broadcast(total, global_seed)

local_range = contiguous_rank_workload(total, rank, world_size)
local_rows = []
for strategy_idx in local_range:
    summary = run strategy(strategy_idx) with deterministic seeds
    local_rows.push(encode_wire(strategy_idx, summary))

all_rows = mpi_utils::gather_strategy_summaries_root(world, root=0, local_rows)
validate count == total and unique strategy_idx
sort by strategy_idx

runs = reconstruct StrategyRunSummary with strategy_desc from strategies[strategy_idx]
best = argmax(score(run))
write_sweep_artifacts(config, "mpi", runs, best)
```

## Rank 1..N-1

```text
init MPI
receive broadcast(total, global_seed)

config = Config::read_from("examples/config_comprehensive.json")
strategies = runner::generate_strategies(config)   // same canonical order as rank 0

local_range = contiguous_rank_workload(total, rank, world_size)
local_rows = []
for strategy_idx in local_range:
    summary = run strategy(strategy_idx) with deterministic seeds
    local_rows.push(encode_wire(strategy_idx, summary))

mpi_utils::gather_strategy_summaries_root(world, root=0, local_rows)
exit
```

---

## Correctness invariants

1. **Coverage**: union of all rank ranges equals `[0, total_strategies)`.
2. **Uniqueness**: each `strategy_idx` appears exactly once in gathered rows.
3. **Determinism**: fixed `(config, global_seed, reps)` yields same outputs independent of rank count.
4. **Canonical order**: final reporting order sorted by `strategy_idx`.
5. **Comparability**: scoring + summary fields match serial/multithreaded definitions.

---

## Failure modes and handling

- **Mismatched strategy count across ranks**
  - Cause: non-identical config/env.
  - Handling: broadcast expected count from root and hard-fail on mismatch.

- **Duplicate/missing strategy indices after gather**
  - Cause: partition bug or gather corruption.
  - Handling: root validates `len == total` and exact index set.

- **Nondeterministic results across runs**
  - Cause: unseeded RNG path still active.
  - Handling: block “comparable” mode until seeded RNG wiring is complete.

- **Variable-length payload issues**
  - Cause: transmitting `strategy_desc` strings directly.
  - Handling: keep wire format numeric-only; reconstruct strings on root.

---

## Concrete integration tasks by file

### `examples/financial_mpi.rs`

- Implement orchestration loop using `partitioning::contiguous_rank_workload`.
- Build local result rows and call `mpi_utils::gather_strategy_summaries_root`.
- Root reconstructs `StrategyRunSummary`, selects best with existing `score()`, writes artifacts.

### `examples/financial_model/runner.rs`

- Expose strategy generation/description helpers to MPI path (e.g., `pub(crate)`).
- Add seeded execution entrypoint that accepts `strategy_idx`, `global_seed` and derives per-rep seeds.

### `examples/financial_model/partitioning.rs`

- Keep existing range partition helpers.
- Add deterministic seed mixer helper(s) used by runner/state.

### `examples/financial_model/mpi_utils.rs`

- Replace scaffold `NotImplemented` with:
  - fixed-width encoding for wire rows
  - count gather + gatherv implementation
  - root-side decode

---

## 1-day implementation checklist (for coding)

1. Make runner strategy helpers accessible to `financial_mpi.rs`.
2. Add seed-derivation helper (`global_seed, strategy_idx, rep_idx, agent_id`).
3. Wire deterministic RNG initialization into state/agent RNG creation path.
4. Implement fixed-width wire DTO + gather in `mpi_utils.rs`.
5. Implement root/worker execution flow in `financial_mpi.rs`.
6. Root validation: coverage + uniqueness + ordering checks.
7. Produce artifacts via `write_sweep_artifacts(&config, "mpi", ...)`.
8. Delegate test/validation execution to `quality-agent`; delegate result analysis to `data-scientist`.

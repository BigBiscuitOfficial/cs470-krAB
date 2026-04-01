# Performance Metrics Schema

**Version:** 1.0  
**Last Updated:** 2026-04-01  
**Purpose:** Standardized profiling schema for scaling analysis across serial, multithreaded, MPI, and hybrid execution modes.

---

## Overview

This document defines the required metrics for performance analysis of the KrAB agent-based modeling framework. The schema supports:
- **Strong scaling**: fixed problem size, varying processor count
- **Weak scaling**: problem size scales with processor count
- **Load imbalance analysis**: work distribution across threads/ranks
- **Reproducibility**: metadata for experiment recreation

---

## 1. CSV Output Schema

### 1.1 Required Columns

All profiling runs MUST emit a CSV file with the following columns:

| Column | Type | Units | Description |
|--------|------|-------|-------------|
| `run_id` | string | - | Unique identifier for this run (e.g., timestamp or UUID) |
| `mode` | string | - | Execution mode: `serial`, `multithreaded`, `mpi`, `hybrid` |
| `num_agents` | integer | count | Total number of agents in simulation |
| `num_steps` | integer | count | Total simulation steps executed |
| `num_reps` | integer | count | Number of replications/repetitions |
| `num_threads` | integer | count | Number of threads (1 for serial/MPI, >1 for MT/hybrid) |
| `num_ranks` | integer | count | Number of MPI ranks (1 for serial/MT, >1 for MPI/hybrid) |
| `total_cores` | integer | count | Total cores used: `num_threads * num_ranks` |
| `init_time_s` | float | seconds | Initialization time (setup, agent creation) |
| `step_compute_s` | float | seconds | Pure computation time (agent updates, field ops) |
| `comm_overhead_s` | float | seconds | Communication/synchronization overhead |
| `metrics_calc_s` | float | seconds | Time spent calculating metrics/statistics |
| `total_runtime_s` | float | seconds | Wall-clock time for entire run |
| `strategy_desc` | string | - | Strategy/configuration description (optional) |
| `hostname` | string | - | Host machine identifier |
| `timestamp` | string | ISO8601 | Run start timestamp |

### 1.2 Optional Performance Columns

| Column | Type | Units | Description |
|--------|------|-------|-------------|
| `agents_per_step_per_s` | float | agent-steps/s | Throughput: `(num_agents * num_steps) / total_runtime_s` |
| `speedup` | float | ratio | Speedup vs. baseline: `T_baseline / T_current` |
| `efficiency` | float | ratio | Parallel efficiency: `speedup / total_cores` |
| `load_imbalance` | float | ratio | Max/mean work ratio across workers (1.0 = perfect) |
| `per_thread_time_max_s` | float | seconds | Max thread execution time |
| `per_thread_time_min_s` | float | seconds | Min thread execution time |
| `per_thread_time_std_s` | float | seconds | Std dev of thread execution times |

### 1.3 Reproducibility Metadata Columns

| Column | Type | Units | Description |
|--------|------|-------|-------------|
| `git_commit` | string | - | Git commit SHA (optional but recommended) |
| `rust_version` | string | - | Rust compiler version |
| `features` | string | - | Cargo features enabled (comma-separated) |
| `seed` | integer | - | Random seed for reproducibility |

---

## 2. CSV Example

```csv
run_id,mode,num_agents,num_steps,num_reps,num_threads,num_ranks,total_cores,init_time_s,step_compute_s,comm_overhead_s,metrics_calc_s,total_runtime_s,strategy_desc,hostname,timestamp
serial_001,serial,1000,100,5,1,1,1,0.123,12.456,0.000,0.234,12.813,baseline,node01,2026-04-01T10:00:00Z
mt_002,multithreaded,1000,100,5,4,1,4,0.134,3.567,0.089,0.245,4.035,baseline,node01,2026-04-01T10:15:00Z
mpi_003,mpi,1000,100,5,1,4,4,0.156,3.789,0.345,0.256,4.546,baseline,node02,2026-04-01T10:30:00Z
```

---

## 3. JSON Schema (Optional, Hierarchical Timing)

For detailed profiling with nested timing regions, use JSON format:

```json
{
  "run_id": "hybrid_004",
  "mode": "hybrid",
  "num_agents": 10000,
  "num_steps": 1000,
  "num_reps": 3,
  "num_threads": 8,
  "num_ranks": 4,
  "total_cores": 32,
  "hostname": "cluster-node03",
  "timestamp": "2026-04-01T12:00:00Z",
  "total_runtime_s": 45.67,
  "timing": {
    "init_s": 1.23,
    "simulation_s": 42.34,
    "simulation_breakdown": {
      "step_compute_s": 38.12,
      "field_updates_s": 15.23,
      "agent_updates_s": 22.89,
      "comm_overhead_s": 4.22,
      "mpi_allreduce_s": 2.10,
      "mpi_sendrecv_s": 2.12
    },
    "metrics_calc_s": 2.10
  },
  "load_balance": {
    "per_rank_time_s": [40.1, 39.8, 41.2, 40.5],
    "per_thread_time_s": [5.01, 4.98, 5.23, 5.11, 4.89, 5.05, 5.17, 4.92],
    "imbalance_ratio": 1.05
  },
  "metadata": {
    "git_commit": "a1b2c3d4",
    "rust_version": "1.78.0",
    "features": "parallel,distributed_mpi",
    "seed": 42
  }
}
```

---

## 4. Derived Metrics and Formulas

### 4.1 Speedup

**Definition:** Ratio of baseline execution time to current execution time.

```
speedup(p) = T_serial / T_parallel(p)
```

Where:
- `T_serial` = baseline single-core runtime
- `T_parallel(p)` = runtime using `p` cores
- Ideal speedup: `speedup(p) = p` (linear)

### 4.2 Parallel Efficiency

**Definition:** How effectively additional processors are utilized.

```
efficiency(p) = speedup(p) / p = T_serial / (p * T_parallel(p))
```

Where:
- Perfect efficiency: `efficiency(p) = 1.0` (100%)
- `efficiency < 1.0` indicates overhead/contention
- Values: `[0.0, 1.0]`, typically report as percentage

### 4.3 Load Imbalance

**Definition:** Ratio of maximum to mean worker execution time.

```
load_imbalance = max(worker_times) / mean(worker_times)
```

Where:
- Perfect balance: `load_imbalance = 1.0`
- `load_imbalance > 1.1` suggests significant imbalance
- Alternatively: `coefficient_of_variation = std(worker_times) / mean(worker_times)`

### 4.4 Throughput

**Definition:** Agent-steps processed per second.

```
throughput = (num_agents * num_steps) / total_runtime_s
```

Units: agent-steps/second

### 4.5 Overhead Fraction

**Definition:** Proportion of time spent on non-computation work.

```
overhead_fraction = comm_overhead_s / total_runtime_s
```

Values: `[0.0, 1.0]`, report as percentage

### 4.6 Strong Scaling Efficiency

**Fixed problem size, varying cores:**

```
strong_scaling_efficiency(p) = T_serial / (p * T_parallel(p))
```

Measure at: 1, 2, 4, 8, 16, 32, ... cores

### 4.7 Weak Scaling Efficiency

**Problem size scales with cores:**

```
weak_scaling_efficiency(p) = T_parallel(1) / T_parallel(p)
```

Where problem size at `p` cores = `p * problem_size(1)`

---

## 5. Instrumentation Guidelines

### 5.1 Timer Placement

**Initialization (`init_time_s`):**
- Start: Before state creation
- End: After all agents initialized, before first step

**Step Computation (`step_compute_s`):**
- Sum of all `schedule.step()` calls
- Includes agent updates, field operations
- Excludes metrics calculation and I/O

**Communication Overhead (`comm_overhead_s`):**
- MPI: `MPI_Allreduce`, `MPI_Send/Recv`, barrier time
- Multithreaded: Thread synchronization, mutex contention
- Calculate: `total_runtime_s - (init_time_s + step_compute_s + metrics_calc_s)`

**Metrics Calculation (`metrics_calc_s`):**
- Time for statistics: median, percentiles, Gini coefficient
- Post-step aggregation

### 5.2 Timestamp Collection

Use Rust `std::time::Instant` for high-resolution timing:

```rust
use std::time::Instant;

let timer = Instant::now();
// ... work ...
let elapsed_s = timer.elapsed().as_secs_f32();
```

### 5.3 Thread-Level Profiling

For load balance analysis, collect per-thread times:

```rust
// Pseudo-code for Rayon parallel execution
let thread_times: Vec<f32> = (0..num_threads)
    .into_par_iter()
    .map(|_| {
        let timer = Instant::now();
        // ... thread work ...
        timer.elapsed().as_secs_f32()
    })
    .collect();

let max_time = thread_times.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
let min_time = thread_times.iter().cloned().fold(f32::INFINITY, f32::min);
let mean_time = thread_times.iter().sum::<f32>() / thread_times.len() as f32;
let load_imbalance = max_time / mean_time;
```

---

## 6. Validation and Quality Checks

### 6.1 Sanity Checks

Parser should flag warnings for:

1. **Time accounting:** `total_runtime_s >= init_time_s + step_compute_s + metrics_calc_s`
2. **Negative overhead:** `comm_overhead_s >= 0.0`
3. **Efficiency bounds:** `0.0 <= efficiency <= 1.0`
4. **Core counts:** `total_cores = num_threads * num_ranks`
5. **Speedup anomalies:** `speedup > total_cores` (super-linear, rare but valid)

### 6.2 Required Fields

Parser must reject CSV files missing:
- `mode`, `num_agents`, `num_steps`, `total_cores`, `total_runtime_s`

---

## 7. Analysis Workflows

### 7.1 Strong Scaling Analysis

1. Run with fixed `num_agents`, `num_steps`
2. Vary `total_cores`: 1, 2, 4, 8, 16, 32, 64
3. Plot: `speedup(cores)` vs. `cores` (linear ideal line)
4. Plot: `efficiency(cores)` vs. `cores` (horizontal at 1.0 ideal)
5. Report: efficiency at 50% of max cores

### 7.2 Weak Scaling Analysis

1. Scale `num_agents` proportionally to `total_cores`
2. Keep `num_agents / total_cores` constant
3. Plot: `efficiency(cores)` vs. `cores`
4. Ideal: flat line at 1.0

### 7.3 Overhead Decomposition

1. Calculate time fractions:
   - Init: `init_time_s / total_runtime_s`
   - Compute: `step_compute_s / total_runtime_s`
   - Comm: `comm_overhead_s / total_runtime_s`
   - Metrics: `metrics_calc_s / total_runtime_s`
2. Stacked bar chart for each core count

### 7.4 Load Balance Report

For each run with `total_cores > 1`:
1. Check `load_imbalance < 1.2` (target)
2. If exceeded, recommend repartitioning
3. Plot: per-thread/rank time distribution (box plot)

---

## 8. CSV Naming Convention

```
profiling_<mode>_<agents>agents_<steps>steps_<cores>cores_<timestamp>.csv
```

Examples:
- `profiling_serial_1000agents_100steps_1cores_20260401_100000.csv`
- `profiling_multithreaded_10000agents_1000steps_16cores_20260401_103000.csv`
- `profiling_hybrid_100000agents_5000steps_128cores_20260401_120000.csv`

---

## 9. Integration with Existing Code

### 9.1 Current Timing Fields in `FinancialSummary`

The existing `FinancialSummary` struct already captures:
- `init_time: f32`
- `step_compute_time: f32`
- `metrics_calc_time: f32`
- `run_duration: f32`
- `communication_overhead: f32`

### 9.2 Required Additions

To align with this schema, add to profiling output:
1. **System metadata**: `hostname`, `timestamp`, `num_threads`, `num_ranks`
2. **Unique identifier**: `run_id`
3. **Derived metrics**: `speedup`, `efficiency`, `throughput`

### 9.3 Example Instrumentation

In `runner.rs`, extend summary emission:

```rust
// After aggregated summary creation
let profiling_record = ProfilingRecord {
    run_id: format!("{}_{}", mode.as_str(), SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs()),
    mode: mode.as_str().to_string(),
    num_agents: config.simulation.num_agents,
    num_steps: config.simulation.steps,
    num_reps: config.simulation.reps,
    num_threads: configured_thread_count(config),
    num_ranks: 1, // Update for MPI
    total_cores: configured_thread_count(config) * 1,
    init_time_s: aggregated.init_time,
    step_compute_s: aggregated.step_compute_time,
    comm_overhead_s: aggregated.communication_overhead,
    metrics_calc_s: aggregated.metrics_calc_time,
    total_runtime_s: aggregated.run_duration,
    strategy_desc: aggregated.strategy_desc.clone(),
    hostname: hostname::get().unwrap().to_string_lossy().to_string(),
    timestamp: chrono::Utc::now().to_rfc3339(),
};

// Append to profiling CSV
write_profiling_csv(&profiling_record, "output/profiling.csv")?;
```

---

## 10. Baseline Calibration

Before scaling studies, establish baseline with:
- **Mode:** `serial`
- **Cores:** 1
- **Problem sizes:** Small (100 agents, 10 steps), Medium (1000 agents, 100 steps), Large (10000 agents, 1000 steps)
- **Repetitions:** ≥5 for statistical significance

Record baseline `total_runtime_s` for each problem size. Use these as denominators for speedup calculations.

---

## 11. References

- **Amdahl's Law:** Maximum speedup limited by serial fraction
- **Gustafson's Law:** Weak scaling assumptions
- **Load Imbalance Factor:** [Bailey 1991, NASA Technical Report]

---

## Appendix A: Quick Reference

| Metric | Formula | Target |
|--------|---------|--------|
| Speedup | `T₁ / Tₚ` | Linear: `p` |
| Efficiency | `Speedup / p` | 0.7 - 1.0 |
| Load Imbalance | `max(tᵢ) / mean(tᵢ)` | < 1.2 |
| Throughput | `(agents × steps) / time` | Maximize |
| Overhead | `comm / total` | < 0.2 |

---

## Appendix B: Sample Analysis Commands

```bash
# Parse and consolidate profiling data
python scripts/analysis/parse_metrics.py \
    output/profiling_serial_*.csv \
    output/profiling_multithreaded_*.csv \
    --output output/consolidated_metrics.csv

# Calculate speedup and efficiency
python scripts/analysis/parse_metrics.py \
    output/profiling_*.csv \
    --baseline serial \
    --output output/scaling_analysis.csv

# Generate scaling plots
python scripts/analysis/plot_scaling.py \
    output/scaling_analysis.csv \
    --type strong \
    --output output/plots/strong_scaling.png
```

---

**End of METRICS_SCHEMA.md**

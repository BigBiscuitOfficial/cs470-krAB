# Financial Simulation Examples

This directory contains comprehensive personal finance life-path simulations built on the krABMaga agent-based modeling framework. These simulations model complete financial lifecycles from ages 22-65+ with realistic life events, market dynamics, and strategic decision-making.

## Overview

The simulations model:
- **10 major life events**: Marriage, divorce, children, inheritance, promotions, job changes, moves, unemployment, medical emergencies, retirement
- **Multiple financial instruments**: 401(k), taxable brokerage, cash, home equity, student loans, auto loans, mortgages, credit cards
- **Strategic dimensions**: Housing (rent vs. buy), debt management (minimum vs. aggressive), asset allocation (80/20, 60/40, 100/0), retirement goals (traditional vs. FIRE)
- **Demographic factors**: Education, gender, race/ethnicity, geography, health status
- **Market dynamics**: Stock/bond returns with volatility, inflation, career growth
- **Tax and healthcare**: Progressive tax brackets, healthcare costs, Social Security

## Available Examples

### 1. financial_serial (Serial Execution)

**Description:** Single-threaded simulation that runs strategy sweeps sequentially.

**What it does:**
- Runs 1000 agents through 45 years of financial life
- Tests 12 different strategy combinations
- 3 repetitions per strategy for statistical significance
- Generates comprehensive reports with charts and recommendations

**How to run:**
```bash
cargo run --example financial_serial --release
```

**Output:** Creates a timestamped directory in `output/` with:
- `report.html` - Interactive visualization with charts
- `summary.json` - Detailed metrics
- `sweep_results.csv` - Strategy comparison table
- `advice.txt` - Best strategy recommendations

**Runtime:** Typically 30-120 seconds depending on hardware

---

### 2. financial_multithreaded (Parallel Execution)

**Description:** Multi-threaded version using Rayon for faster execution.

**What it does:** Same as serial, but parallelizes agent scheduling across CPU cores.

**How to run:**
```bash
# Basic usage
cargo run --example financial_multithreaded --release --features parallel

# Specify thread count
KRAB_THREAD_COUNT=8 cargo run --example financial_multithreaded --release --features parallel
```

**Performance:** 2-8x speedup depending on core count

**Output:** Same format as serial version

---

### 3. financial_mpi (Distributed MPI Execution)

**Description:** MPI-based distributed execution for running across multiple nodes.

**Status:** ⚠️ Currently a placeholder - implementation in progress

**How to run:**
```bash
# Build with MPI support
cargo build --example financial_mpi --release --features distributed_mpi

# Run with MPI (example for 4 processes)
mpirun -np 4 target/release/examples/financial_mpi
```

**Note:** The MPI implementation is incomplete. Currently prints "MPI distributed sweep not yet implemented."

---

## Configuration

### Default Configuration
All examples use `examples/config_comprehensive.json` by default, which includes:
- 45 simulation steps (years)
- 1000 agents per run
- 3 repetitions per strategy
- All life events enabled
- Full demographic modeling

### Customizing Configuration
Edit `examples/config_comprehensive.json` to adjust:
- Number of agents
- Simulation duration
- Strategy combinations
- Life event probabilities
- Initial agent demographics

## Understanding the Output

### report.html
Interactive HTML report with:
- **Net Worth Funnel Chart**: P10/Median/P90 trajectories over time
- **Strategy Scatter Plot**: Bankruptcy rate vs. median net worth
- **Key Metrics Dashboard**: Success rates, account balances
- **Top 10 Strategies Table**: Ranked by composite score

### summary.json
Detailed metrics including:
- Net worth distribution (mean, median, percentiles)
- Gini coefficient (wealth inequality)
- Bankruptcy and retirement success counts
- Account composition breakdown
- Full timeseries data

### sweep_results.csv
Comparison table with all strategies showing:
- Median net worth
- Bankruptcy rate
- Retirement success rate
- Account breakdowns
- Composite score

### advice.txt
Plain-text recommendation with:
- Best strategy identification
- Expected outcomes
- Risk assessment

## Strategy Combinations

The simulations test combinations of:

**Housing Strategy:**
- Rent (flexible, lower commitment)
- Buy (equity building, tax benefits)

**Debt Strategy:**
- Minimum (pay only required amounts)
- Aggressive (avalanche method, highest interest first)

**Asset Allocation:**
- 80/20 stocks/bonds (Aggressive)
- 60/40 stocks/bonds (Balanced)
- 100/0 stocks/bonds (Very Aggressive)

**Retirement Goal:**
- Traditional (retire at age 65)
- FIRE (Financial Independence Retire Early - 25x expenses)

Total combinations: 2 × 2 × 3 × 2 = 12 strategies

## Quick Start

```bash
# Run your first simulation
cargo run --example financial_serial --release

# View the results
open output/financial_serial_*/report.html

# For faster execution (requires parallel feature)
cargo run --example financial_multithreaded --release --features parallel
```

## Performance Characteristics

**Serial mode:**
- 12 strategies × 3 reps = 36 simulation runs
- Each run: 45 steps × 1000 agents = 45,000 agent-years
- Total: ~1.62 million agent-years per sweep

**Multithreaded mode:**
- Parallelizes agent scheduling within each step
- Best for single-strategy deep analysis
- Speedup scales with core count

**MPI mode:**
- (Not yet implemented)
- Would distribute strategy combinations across nodes

## Documentation

For more details, see:
- `examples/LIFE_EVENTS.md` - Comprehensive guide to all life events
- `examples/financial_model/mod.rs` - Implementation details
- Main `README.md` - krABMaga framework documentation

## Viewing Results

After running a simulation, navigate to the output directory:
```bash
cd output
ls -t | head -1  # Shows most recent run
cd <latest_directory>
open report.html  # Or use your browser
```

On Linux, use `xdg-open` instead of `open`.

## Tips

1. **Always use --release**: Debug builds are 10-100x slower
2. **Start with serial**: Simplest to run, no feature flags needed
3. **Check report.html first**: Best visualization of results
4. **Compare strategies**: Look for patterns in bankruptcy rates vs. net worth
5. **Experiment with config**: Adjust agent count for faster iteration

## Requirements

- Rust 1.70 or later
- For parallel execution: `parallel` feature flag
- For MPI execution: `distributed_mpi` feature flag and MPI runtime (OpenMPI or MPICH)

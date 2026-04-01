# MPI Docker Usage Guide

**Purpose**: Reproduce MPI smoke tests locally without cluster access.

**Scope**: This guide covers building and running the KrAB financial simulation examples (serial, multithreaded, MPI) inside the Docker MPI cluster environment.

---

## Prerequisites

- **Docker** (v20.10+): `docker --version`
- **Docker Compose** (v1.29+): `docker-compose --version`
- **Rust** (1.70+, local): `rustc --version` (for host-side compilation; container has its own)
- **Git**: To clone/pull the repo

Optional but recommended:
- `jq`: For parsing JSON output (`output/summary.json`)
- A text editor or viewer for HTML reports

---

## Quick Start (5 minutes)

### 1. Start the Docker MPI Cluster

```bash
cd mpi-cluster-docker
docker-compose up -d --build
```

**What this does**:
- Builds base image with Ubuntu 22.04, OpenMPI, SSH, and Rust toolchain
- Starts 3 containers: `master`, `worker1`, `worker2`
- Mounts `./workdir` as shared volume at `/home/mpiuser/workdir`
- Sets up passwordless SSH across all nodes

**Verify cluster is running**:
```bash
docker-compose ps
# Output should show all three containers in "Up" state
```

---

### 2. Enter the Master Container

Option A (recommended): Direct shell access
```bash
docker-compose exec master sudo -u mpiuser -i
```

Option B: SSH (if needed)
```bash
ssh -p 2222 mpiuser@localhost
# Password: mpi
```

You are now inside `/home/mpiuser/` on the master container.

---

### 3. Clone/Prepare the Repo Inside Container

If the repo is not already mounted:
```bash
# Inside container
cd workdir
git clone https://github.com/YOUR_ORG/cs470-krAB.git
cd cs470-krAB
```

Or if already mounted via docker-compose volume:
```bash
# Inside container
cd /home/mpiuser/workdir/cs470-krAB
ls -la  # Verify you see Cargo.toml, examples/, docs/, etc.
```

---

### 4. Build the Examples

#### 4.1 Serial Mode (baseline)

```bash
# Inside container, in repo root
cargo build --release --example financial_serial
```

**Expected output**: Binary at `target/release/examples/financial_serial`

#### 4.2 Multithreaded Mode

Requires the `parallel` feature:

```bash
cargo build --release --example financial_multithreaded --features parallel
```

#### 4.3 MPI Mode (requires distributed_mpi feature)

```bash
cargo build --release --example financial_mpi --features distributed_mpi
```

**Build time**: 2–5 minutes depending on container state.

---

## Smoke Test Execution

### Smoke Test 1: Serial Mode

```bash
# Inside container, repo root
export KRAB_OUTPUT_DIR=/home/mpiuser/workdir/output/serial_run
mkdir -p $KRAB_OUTPUT_DIR

timeout 60s ./target/release/examples/financial_serial 2>&1 | head -50
```

**Expected output**:
```
Headless run artifacts:
- run dir: output/serial_001/
- report: output/serial_001/report.html
- summary: output/serial_001/summary.json
- sweep results: output/serial_001/sweep_results.csv

Best strategy: Buy/AggDebt/60%stocks/40%bonds/Age65
  Median net worth: $...
  P10-P90 range: $... - $...
```

**Verification**:
```bash
ls -lh $KRAB_OUTPUT_DIR/output/serial_001/
# Should contain: report.html, summary.json, sweep_results.csv, timeseries.csv, profiling_serial_*.csv
```

---

### Smoke Test 2: Multithreaded Mode (4 threads)

```bash
# Inside container, repo root
export KRAB_OUTPUT_DIR=/home/mpiuser/workdir/output/mt_run
export KRAB_THREAD_COUNT=4
mkdir -p $KRAB_OUTPUT_DIR

timeout 60s ./target/release/examples/financial_multithreaded 2>&1 | head -50
```

**Expected output**: Same structure as serial, but with timing improvements in profiling CSV.

**Verification**:
```bash
ls -lh $KRAB_OUTPUT_DIR/output/multithreaded_*/
# Check profiling CSV for num_threads=4
head -2 $KRAB_OUTPUT_DIR/output/multithreaded_*/profiling_multithreaded_*.csv
```

---

### Smoke Test 3: MPI Mode (2 ranks)

```bash
# Inside container, repo root
export KRAB_OUTPUT_DIR=/home/mpiuser/workdir/output/mpi_run
mkdir -p $KRAB_OUTPUT_DIR

# Small config for quick test (or use examples/config.json if smaller)
mpirun -np 2 ./target/release/examples/financial_mpi 2>&1 | head -50
```

**Expected output** (current wave 1):
```
MPI distributed sweep not yet implemented for comprehensive strategy model.
TODO: Refactor to distribute strategy combinations across MPI ranks.
For now, use serial or multithreaded modes.
```

**Note**: Wave 1 (current) has MPI initialization and scaffold functions. Full distributed sweep is P1.

**Verification** (Wave 1):
```bash
mpirun -np 2 ./target/release/examples/financial_mpi 2>&1 | grep -i "MPI init"
# Should succeed without MPI errors (transport layer working)
```

---

## Output Artifact Verification

After running smoke tests, check shared output directory:

```bash
# From host machine (outside container)
cd mpi-cluster-docker/workdir/output

# List all generated artifacts
find . -type f -name "*.json" -o -name "*.csv" -o -name "*.html"

# Quick metrics check
head -3 */profiling_*.csv  # Verify CSV schema
jq '.strategy_desc, .median_net_worth' */summary.json  # View summaries
```

---

## Profiling CSV Schema Verification

Expected columns in `profiling_*.csv`:

```
mode,event,strategy_index,strategy_desc,rep_index,duration_seconds
```

Example row:
```
serial,sweep_total,,baseline,0,12.345
serial,init,,baseline,0,0.123
serial,step_compute,,baseline,0,11.234
```

**Validation script** (Python, inside or outside container):

```python
import csv
import sys

def verify_csv(filename):
    required = {'mode', 'event', 'duration_seconds'}
    with open(filename) as f:
        reader = csv.DictReader(f)
        if not reader.fieldnames:
            print(f"FAIL: {filename} - no header")
            return False
        cols = set(reader.fieldnames)
        if not required.issubset(cols):
            print(f"FAIL: {filename} - missing {required - cols}")
            return False
        print(f"OK: {filename} - schema valid, {len(reader.fieldnames)} columns")
        return True

for fn in sys.argv[1:]:
    verify_csv(fn)
```

Run:
```bash
python verify_profiling.py output/*/profiling_*.csv
```

---

## Metrics Collection Checklist

After each smoke test run, verify:

- [ ] `summary.json` exists and contains:
  - `strategy_desc` (string)
  - `median_net_worth` (float)
  - `p10_net_worth`, `p90_net_worth` (floats)
  - `bankruptcy_rate` (float)

- [ ] `profiling_*.csv` exists and contains:
  - At least 5 rows (init, step_compute, sweep_total, metrics_calc, run_duration)
  - `mode` field matches expected (serial, multithreaded, mpi)
  - `duration_seconds` is numeric and positive

- [ ] `sweep_results.csv` exists (for strategy sweep runs):
  - One row per strategy tested
  - Columns: strategy_desc, median_net_worth, p10_net_worth, etc.

- [ ] `timeseries.csv` exists (if enabled in config):
  - Time-indexed wealth and metric traces

---

## Common Issues & Troubleshooting

### Issue: Docker containers fail to build
**Solution**: Clear Docker cache and rebuild
```bash
docker-compose down -v
docker-compose up -d --build
```

### Issue: `mpirun` command not found in container
**Solution**: Ensure you're inside the container (output of `docker-compose exec master...`), and OpenMPI was installed. Verify:
```bash
which mpirun
mpirun --version
```

### Issue: "Permission denied" on workdir files
**Solution**: Files mounted from host may have different ownership. Use `sudo` inside container or ensure host files are world-readable.

### Issue: Cargo build fails in container
**Solution**: Ensure Rust is installed in container image (check Dockerfile). Try:
```bash
rustc --version
cargo --version
```

If missing, update `mpi-cluster-docker/Dockerfile` to include Rust toolchain installation.

### Issue: MPI example runs but produces no output files
**Solution**: Verify `KRAB_OUTPUT_DIR` exists and is writable:
```bash
mkdir -p $KRAB_OUTPUT_DIR
touch $KRAB_OUTPUT_DIR/test.txt  # Check permissions
ls -ld $KRAB_OUTPUT_DIR
```

---

## Cleanup & Shutdown

To stop the cluster:

```bash
cd mpi-cluster-docker
docker-compose down
```

To also remove volumes and rebuild next time:

```bash
docker-compose down -v
```

To remove the Docker image entirely:

```bash
docker rmi mpi-cluster-base
docker-compose up -d --build  # Rebuild on next use
```

---

## Integration with Continuous Metrics Collection

For scaling studies, run smoke tests across different core counts:

```bash
# Inside container, repo root

# Baseline: serial
./target/release/examples/financial_serial

# 4-thread multithreaded
KRAB_THREAD_COUNT=4 ./target/release/examples/financial_multithreaded

# 8-thread multithreaded (if host supports)
KRAB_THREAD_COUNT=8 ./target/release/examples/financial_multithreaded

# MPI (after P1 implementation)
# mpirun -np 2 ./target/release/examples/financial_mpi
# mpirun -np 4 ./target/release/examples/financial_mpi
```

Collect all `profiling_*.csv` files and run analysis:

```bash
python scripts/analysis/parse_metrics.py \
    output/*/profiling_*.csv \
    --output output/consolidated_metrics.csv
```

---

## References

- **KrAB Metrics Schema**: `docs/METRICS_SCHEMA.md`
- **MPI Design Document**: `docs/MPI_DESIGN.md`
- **Agent Workflow**: `docs/AGENT_WORKFLOW.md`
- **OpenMPI Documentation**: https://www.open-mpi.org/doc/current/
- **Docker Compose Reference**: https://docs.docker.com/compose/

---

**Last Updated**: 2026-04-01  
**Status**: Wave 1 (MPI stubs; full distributed sweep in P1)

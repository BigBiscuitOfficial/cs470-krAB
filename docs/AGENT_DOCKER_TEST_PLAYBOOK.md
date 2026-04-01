# Agent Docker Test Playbook

Use this when working on MPI/distributed changes and when validating serial vs multithreaded vs MPI parity.

## Host Preflight

Run from repo root:

```bash
docker --version
docker compose version
mpirun --version
mpicc --version
clang --version
```

Pass criteria:
- Docker and Docker Compose available
- `mpirun`/`mpicc` available on host
- `clang` available on host

## Fast Local Checks (No Docker)

Run these first for quick feedback:

```bash
cargo check --example financial_serial
cargo check --example financial_multithreaded --features parallel
cargo test --test lib integration::serial_correctness::serial_baseline_matches_financial_fixture
```

Use reduced deterministic config manually if needed:

```bash
KRAB_CONFIG_PATH=tests/fixtures/config_reduced_seeded.json cargo run --example financial_serial
```

## Docker MPI Parity Test (Preferred for MPI PRs)

This test runs serial + Docker MPI and compares parity.

```bash
RUN_MPI_DOCKER_TESTS=1 cargo test --test lib integration::mpi_smoke_test::mpi_smoke_via_docker_script -- --nocapture
```

What it does:
- Uses `tests/fixtures/config_reduced_seeded.json`
- Runs Docker MPI via `run_mpi_docker.sh`
- Compares best strategy and key metrics with tolerance

## Direct Docker Script Run

Default smoke run:

```bash
./run_mpi_docker.sh
```

With explicit overrides:

```bash
KRAB_CONFIG_PATH=tests/fixtures/config_reduced_seeded.json \
KRAB_OUTPUT_DIR=/home/mpiuser/workdir/output/mpi_parity_test \
KRAB_MPI_NP=2 \
./run_mpi_docker.sh
```

## MPI Scaling Sweep (Large Workloads)

Use this when you need meaningful MPI scaling data where communication overhead is amortized.

```bash
KRAB_CONFIG_PATH=examples/config_comprehensive.json \
KRAB_MPI_NP_VALUES=2,4 \
KRAB_SWEEP_OUTPUT_ROOT=output/mpi_scale_sweep \
./scripts/mpi_scale_sweep.sh
```

Outputs:
- `output/mpi_scale_sweep/np*/` run artifacts
- `output/mpi_scale_sweep/consolidated_metrics.csv`
- `output/mpi_scale_sweep/mpi_sweep_timings.csv` (wall-clock by NP)
- `output/mpi_scale_sweep/mpi_scaling.svg` (wall-clock scaling graph)
- `output/mpi_scale_sweep/mpi_speedup_efficiency.svg` (observed vs ideal speedup and efficiency labels)
- `output/mpi_scale_sweep/mpi_scaling_report.md` (shareable markdown summary with table + graphs)

Note:
- MPI profiling CSV location may vary by mode/config. The sweep script consolidates profiling when files are present and always records wall-clock timings by NP.

## When Agents Must Run Docker Tests

- `distributed-dev`: any changes to MPI behavior, partitioning, gather/reduce, or `examples/financial_mpi.rs`
- `quality-agent`: any MPI-related test, parity, or integration validation work
- `dev-agent`: if shared runner/config/profiling changes can affect MPI parity

Minimum requirement for MPI-related changes:
1. local compile checks
2. Docker MPI parity test command above

## Common Failures

- Docker not running: start Docker Desktop/daemon and retry
- Missing MPI toolchain on host: install MPICH/OpenMPI so host checks can run
- Docker test cannot find summary path: ensure `run_mpi_docker.sh` completed and printed `- summary:` line
- Permission issues in output: set writable `KRAB_OUTPUT_DIR`

## Output Expectations

After successful Docker MPI run, expect:
- `summary.json`
- `sweep_results.csv`
- `report.html`
- profiling CSV under configured output directory

Use this playbook as the authoritative command sequence for agent-run MPI validation.

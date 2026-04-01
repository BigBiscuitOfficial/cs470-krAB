# EOD Status 2026-04-01

## Decision

- **GO for demo** with serial + multithreaded + Docker MPI parity validation.

## Multithreading Outcome

- Medium workload benchmark (`2000 agents`, `20 steps`, `16 strategies`, `seed=42`) from profiling artifacts:
  - Serial `sweep_total`: `0.207226 s` from `/tmp/krab-eod/serial-medium/profiling/profiling_serial_2000agents_20steps_1cores_20260401_204216_456.csv`
  - Multithreaded `sweep_total` (`KRAB_THREAD_COUNT=4`): `0.053482 s` from `/tmp/krab-eod/mt-medium/profiling/profiling_multithreaded_2000agents_20steps_4cores_20260401_204216_315.csv`
  - Speedup: `3.87x`
  - Efficiency on 4 cores: `96.9%`

## Semantic Parity Status

- Serial and multithreaded summaries match on strategy ranking and key metrics for validated reduced/medium runs.
- Example parity point (medium run):
  - Best strategy: `Housing: Buy, Debt: Minimum, Stocks: 100%, Retirement: Age { target: 65 }`
  - Median net worth: `$1,288,527.9` in both serial and multithreaded summary outputs.

## Validation Checklist (Executed)

- `cargo check --example financial_serial` passed
- `cargo check --example financial_multithreaded --features parallel` passed
- `cargo check --example financial_mpi --features distributed_mpi` passed
- `cargo test --test lib integration::serial_correctness::serial_baseline_matches_financial_fixture` passed
- `RUN_MPI_DOCKER_TESTS=1 cargo test --test lib integration::mpi_smoke_test::mpi_smoke_via_docker_script -- --nocapture` passed

## Demo Commands

```bash
# Serial (reduced deterministic)
KRAB_CONFIG_PATH=tests/fixtures/config_reduced_seeded.json \
cargo run --release --example financial_serial

# Multithreaded (4 threads, reduced deterministic)
KRAB_CONFIG_PATH=tests/fixtures/config_reduced_seeded.json \
KRAB_THREAD_COUNT=4 \
cargo run --release --features parallel --example financial_multithreaded

# Docker MPI parity test
RUN_MPI_DOCKER_TESTS=1 \
cargo test --test lib integration::mpi_smoke_test::mpi_smoke_via_docker_script -- --nocapture
```

## Known Limitations

- Hybrid MPI+multithreading mode is not implemented yet.
- Current MPI transport and parity path are validated via Docker integration tests.
- Additional large-scale scaling matrix automation (strong/weak scaling batches) remains follow-up work.

## MPI Benchmark Reliability Note

- Small workloads can hide MPI benefit because startup/communication overhead dominates.
- For MPI scaling claims, use larger workloads (for example `>= 10000 agents`, `>= 45 steps`, `>= 24 strategy combinations`) and compare serial vs MPI on the same config.
- Recommended command pattern:

```bash
# Example large-workload MPI run in Docker (adjust NP as needed)
KRAB_CONFIG_PATH=examples/config_comprehensive.json \
KRAB_MPI_NP=4 \
./run_mpi_docker.sh
```

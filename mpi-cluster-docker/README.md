# Docker MPI Cluster

This sets up a simple virtual cluster using Docker Compose to run MPI applications.

It uses `ubuntu:22.04` and OpenMPI, generating a shared SSH key at build time so the master node can run passwordless SSH commands on the worker nodes.

## How to start it

1. Make sure you have Docker and Docker Compose installed.
2. Build and start the cluster:
   ```bash
   cd mpi-cluster
   docker-compose up -d --build
   ```

## How to use it

1. SSH into the master node (we mapped port 2222 on your local machine to port 22 on the master container):
   ```bash
   ssh -p 2222 mpiuser@localhost
   # Password is: mpi
   ```
   Or alternatively, run a bash shell directly using docker-compose:
   ```bash
   docker-compose exec master sudo -u mpiuser -i
   ```

2. Inside the master node, you can run the provided exampleMPI C program across the cluster:
   ```bash
   mpirun --hostfile workdir/hostfile -np 3 ./hello_mpi
   ```

## Shared Workspace

The `./workdir` directory on your host machine is mapped to `/home/mpiuser/workdir` inside all containers. You can compile your MPI code in the container and place the executable there, so all nodes can access it instantly.

## KrAB Financial Simulation (Serial, Multithreaded, MPI)

### Quick Start for KrAB Examples

**See full instructions in**: `docs/MPI_DOCKER_USAGE.md` (comprehensive guide with reproducible commands)

### Quick Reference

Inside the master container, compile and run:

```bash
# Build all three modes
cargo build --release --example financial_serial
cargo build --release --example financial_multithreaded --features parallel
cargo build --release --example financial_mpi --features distributed_mpi

# Serial baseline
./target/release/examples/financial_serial

# Multithreaded (4 threads)
KRAB_THREAD_COUNT=4 ./target/release/examples/financial_multithreaded

# MPI smoke test (2 ranks)
mpirun -np 2 ./target/release/examples/financial_mpi
```

### Artifact Output

For all modes, set `KRAB_OUTPUT_DIR` to persist results outside the container:

```bash
KRAB_OUTPUT_DIR=/home/mpiuser/workdir/output ./target/release/examples/financial_serial
```

Generated artifacts are visible on the host under:

```text
mpi-cluster-docker/workdir/output/
├── summary.json           # Strategy results and metrics
├── timeseries.csv         # Time-indexed wealth traces
├── sweep_results.csv      # Per-strategy comparison
├── report.html            # Interactive visualization
└── profiling_*.csv        # Timing breakdown (init, compute, comm, metrics)
```

### Metrics & Profiling

All three modes emit `profiling_*.csv` with consistent schema:

- `mode`: serial | multithreaded | mpi
- `event`: init | step_compute | metrics_calc | sweep_total | run_duration
- `duration_seconds`: numeric timing for each phase

Use for scaling analysis (strong/weak) and load balance studies.

See `docs/METRICS_SCHEMA.md` for full schema specification and `docs/MPI_DESIGN.md` for distributed strategy sweep architecture.

### Debugging

If `per_rank_debug` is enabled in `examples/config.json`, each rank writes a debug file under:

```text
<KRAB_OUTPUT_DIR>/mpi_rank_debug/rank_<rank>.txt
```

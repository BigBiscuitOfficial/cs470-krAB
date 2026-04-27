# cs470-krAB

Final project repository for CS470 by Landon Mann and Giovani Nunez-Lopez.

This project uses the `krABMaga` agent-based modeling framework to run a distributed MPI genetic algorithm for financial life-cycle simulation. The main target is the `financial_life_exploration` example, which evaluates policy genomes across many simulated households and uses parallel execution to study runtime scaling on the school cluster.

## Project Overview

The core experiment models households over a long financial horizon and scores candidate policy profiles against outcomes such as:

- net-worth growth
- bankruptcy avoidance
- debt management
- retirement readiness

Each candidate policy is represented by a 7-value genome:

1. `frugality`
2. `savings_discipline`
3. `career_drive`
4. `risk_tolerance`
5. `resilience`
6. `family_pressure`
7. `education_investment`

The distributed genetic algorithm evaluates many individuals across repeated simulations, which makes the workload a good fit for MPI-based scaling experiments.

This repository also includes original upstream examples from `krABMaga`. For this project, those examples were useful as control cases while troubleshooting MPI behavior on the school cluster, especially when distributed runs would hang indefinitely across multiple nodes.

## Project Contributions

The main original implementation for this final project is `examples/financial_life_exploration/`.

In addition to building that simulation, this project includes framework-level and MPI-level fixes made during cluster debugging:

- implemented the financial life-cycle simulation and its distributed GA workflow
- used an upstream example as a comparison case to isolate multi-node MPI hangs
- fixed an MPI GA macro issue by guarding an early exit path that could leave ranks out of sync
- added an explicit barrier in the financial simulation's main execution flow to keep ranks aligned
- added the `mpi_verbose_timing` feature for MPI GA timing visibility during debugging and scaling work
- added a hybrid MPI + multithreading execution path by evaluating each rank's assigned workload with a Rayon parallel iterator
- fixed `krABMaga` for headless compilation on the JMU cluster
- declared the required `plotters` features explicitly so the framework can compile without an X server

The main files involved in those changes are:

- `examples/financial_life_exploration/src/main.rs`: the project simulation entrypoint, including the distributed GA invocation and the explicit `world.barrier()` synchronization after the run
- `examples/financial_life_exploration/Cargo.toml`: feature wiring for `distributed_mpi`, `parallel`, and `mpi_verbose_timing`
- `krABMaga/src/explore/mpi/genetic.rs`: the distributed GA macro implementation, including the MPI early-exit/synchronization logic, the Rayon-based per-rank parallel evaluation path, and the verbose per-generation timing instrumentation
- `krABMaga/Cargo.toml`: framework feature declarations, including `mpi_verbose_timing` and the headless-safe `plotters` dependency configuration with explicit bitmap-related features

## Repository Layout

- `examples/financial_life_exploration/`: primary CS470 simulation and scaling scripts
- `examples/sir_ga_exploration/`: upstream comparison example used during MPI troubleshooting
- `krABMaga/`: simulation framework source used by the examples

## School Cluster Prerequisites

These instructions are intended for the school cluster environment.

You need:

- access to the cluster login node and scheduler
- `rustup` for installing Rust
- an MPI-enabled environment on the cluster
- `libclang` available for crates that depend on `bindgen`

Install Rust with `rustup` if needed:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Export the cluster-specific `clang` settings in your shell or add them to your shell startup file:

```bash
export LIBCLANG_PATH=/shared/common/clang+llvm-14.0.0/lib/
export BINDGEN_EXTRA_CLANG_ARGS="-I/usr/lib/gcc/x86_64-redhat-linux/8/include"
```

These environment variables are required because some Rust MPI dependencies rely on `bindgen`, which needs access to the cluster's `clang` and GCC headers.

## Quick Start

Build the financial example:

```bash
cd examples/financial_life_exploration
cargo build --release --features "distributed_mpi mpi_verbose_timing"
```

The executable will be written to:

```text
examples/target/release/finance_life_exploration
```

Request an allocation and launch the program with MPI (there is a current cluster issue with MPI batch jobs):

```bash
salloc -Q -n <nprocs>
mpirun ./target/release/finance_life_exploration
```

If your cluster workflow prefers a one-line launch from inside an allocation, adapt the process count as needed:

```bash
mpirun -n 4 ./target/release/finance_life_exploration
```

## Financial Model

Each simulation run evolves a cohort of households through annual life stages that include:

- income growth and career changes
- savings and spending behavior
- retirement transition
- housing, family, health, and financial shocks

The search process evaluates genomes over repeated runs and pushes the population toward more stable long-term financial outcomes.

At a high level, the computational workload scales with:

```text
INDIVIDUALS * REPETITIONS * HORIZON * HOUSEHOLDS
```

## Runtime Configuration

The simulation supports runtime overrides through environment variables:

| Variable | Default | Meaning |
| --- | --- | --- |
| `FIN_HORIZON` | `60` | Number of simulated years per household run |
| `FIN_REPETITIONS` | `8` | Repeated evaluations used to reduce noise |
| `FIN_MAX_GENERATION` | `150` | Maximum GA generations |
| `FIN_HOUSEHOLDS` | `48` | Number of households in a cohort |
| `FIN_INDIVIDUALS` | `256` | Population size for the GA |
| `FIN_SEED` | `1592655742` | Random seed for reproducible runs |

`FIN_INDIVIDUALS` is the most important variable for MPI scaling experiments because it directly changes how much candidate-evaluation work can be distributed across processes.

Example:

```bash
FIN_INDIVIDUALS=512 FIN_HOUSEHOLDS=48 FIN_HORIZON=60 FIN_REPETITIONS=8 \
mpirun -n 64 ./target/release/finance_life_exploration
```

## Scaling Experiments

The repository includes scripts in `examples/financial_life_exploration/` for scaling studies.

Use:

- `run_weak_scaling.sh` as a template for weak scaling experiments
- `run_strong_scaling.sh` as a template for strong scaling experiments
- `runsim.sh` for direct execution patterns

Recommended interpretation:

- weak scaling: keep `FIN_HOUSEHOLDS`, `FIN_HORIZON`, and `FIN_REPETITIONS` fixed while increasing `FIN_INDIVIDUALS` with process count
- strong scaling: keep the total problem size fixed while increasing the number of MPI processes

The codebase also supports a hybrid MPI + threading configuration. In that mode, MPI distributes individuals across ranks and each rank evaluates its local workload with a Rayon `par_iter()` through the `parallel` feature.

Build command for the hybrid path:

```bash
cargo build --release --features "distributed_mpi parallel mpi_verbose_timing"
```

When using the hybrid path on the cluster, set `RAYON_NUM_THREADS` to match the CPU cores allocated per MPI rank. Otherwise, each rank may try to use a full Rayon worker pool and oversubscribe the node.

Example:

```bash
salloc -Q -n 8 --cpus-per-task=4
export RAYON_NUM_THREADS=4
mpirun ./target/release/finance_life_exploration
```

Generated logs and result files in the example directory can be used to compare runtime trends across process counts.

## Interpreting Results

For a run that saves stdout to a log file, you can translate the best final genome into a short plain-English summary with:

```bash
FIN_SEED=1234 salloc -Q -n 4 mpirun ./target/release/finance_life_exploration | tee finance_life_run.log
python3 tools/interpret_financial_run.py finance_life_run.log -o financial_interpretation.txt
```

This helper reads the best-individual block from the run log and produces a readable explanation of the winning policy profile.

## Additional Documentation

- `examples/financial_life_exploration/README.md`: model-specific notes and example commands
- `examples/README.md`: overview of additional krABMaga example simulations
- `krABMaga/README.md`: upstream framework details

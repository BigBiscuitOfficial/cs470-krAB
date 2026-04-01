# Agent Workflow Guide

Use `architect` as the default entry point. It should decompose requests, delegate to specialists, and synthesize results.

## Agent Map

- `architect`: orchestration only
- `dev-agent`: core Rust simulation (non-MPI)
- `data-scientist`: calibration, scaling metrics, result analysis
- `quality-agent`: tests, builds, local validation
- `mpi-architect`: MPI algorithm and communication design
- `distributed-dev`: MPI Rust implementation
- `slurm-engineer`: Slurm scripts for cluster runs
- `cluster-coordinator`: deployment docs and integration

## When To Use Which Agent

- New core model feature or bug fix -> `dev-agent`
- MPI distribution design -> `mpi-architect`
- MPI code implementation -> `distributed-dev`
- Slurm run scripts -> `slurm-engineer`
- Scaling analysis and plots -> `data-scientist`
- Test/build validation -> `quality-agent`
- Deployment guide/checklists -> `cluster-coordinator`

## Standard Flows

### Core feature flow
1. `architect` delegates requirements to `dev-agent`
2. `quality-agent` validates build/tests
3. `architect` reports outcome and next actions

### MPI flow
1. `architect` asks `mpi-architect` for design
2. `architect` hands design to `distributed-dev`
3. `quality-agent` validates locally (e.g., Docker MPI)
4. `slurm-engineer` prepares job scripts
5. `cluster-coordinator` updates deployment docs

### MPI test execution requirements
- For MPI-related code changes, `distributed-dev` and `quality-agent` must run Docker-backed MPI validation using:
  - `RUN_MPI_DOCKER_TESTS=1 cargo test --test lib integration::mpi_smoke_test::mpi_smoke_via_docker_script -- --nocapture`
- Run fast local checks before Docker:
  - `cargo check --example financial_serial`
  - `cargo check --example financial_multithreaded --features parallel`
- Use `docs/AGENT_DOCKER_TEST_PLAYBOOK.md` as the canonical command reference.

### Scaling study flow
1. `data-scientist` prepares configs/analysis templates
2. `slurm-engineer` prepares strong/weak scaling scripts
3. User runs jobs on cluster
4. `data-scientist` analyzes outputs

## Boundaries

- `architect` must not implement code or scripts.
- `mpi-architect` designs only; implementation goes to `distributed-dev`.
- `slurm-engineer` prepares scripts only; user runs `sbatch`/`srun`.
- `quality-agent` validates; it does not own feature implementation.
- `data-scientist` owns performance analysis; others should delegate that work.

## Manual Invocation

You can invoke specialists directly with `@agent-name` for narrow tasks.

Examples:
- `@mpi-architect design load-balanced work partitioning`
- `@slurm-engineer create strong scaling script`
- `@data-scientist analyze results in ./results/`

## Troubleshooting

- If `architect` starts implementing: remind it to delegate.
- If an agent reports out-of-scope: follow its delegation recommendation.
- If unsure where to start: ask `architect` and provide your goal plus constraints.

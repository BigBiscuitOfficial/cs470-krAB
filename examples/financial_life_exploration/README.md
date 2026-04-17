# Financial Life Exploration

A distributed MPI GA example for personal finance lifecycle dynamics.

Each simulation run evolves a cohort of households that start from different financial situations and then move through annual life stages:
- income growth and career changes
- savings and spending habits
- retirement transition
- housing, family, health, and shock events

The GA searches for a policy profile that keeps the cohort close to a healthy financial benchmark: steady net-worth growth, low bankruptcy, manageable debt, and adequate retirement coverage.

## Policy genome

### For this simulation, the input size is effectively
- INDIVIDUALS * REPETITIONS * HORIZON * HOUSEHOLDS

The genome is a `;`-separated list of 7 values in `[0, 1]`:
1. `frugality`
2. `savings_discipline`
3. `career_drive`
4. `risk_tolerance`
5. `resilience`
6. `family_pressure`
7. `education_investment`

## Run

Use MPI to launch the distributed GA:

```bash
mpirun -n 4 cargo run --release --features distributed_mpi
```

### Scaling overrides (drop-in)

The example now supports runtime scaling through env vars while keeping defaults unchanged.

- `FIN_HOUSEHOLDS` (default: `48`)
- `FIN_HORIZON` (default: `60`)
- `FIN_INDIVIDUALS` (default: `256`)
- `FIN_MAX_GENERATION` (default: `150`)
- `FIN_REPETITIONS` (default: `8`)

Example:

```bash
FIN_INDIVIDUALS=512 FIN_HOUSEHOLDS=48 FIN_HORIZON=60 FIN_REPETITIONS=8 \
mpirun -n 64 cargo run --release --features distributed_mpi
```

For weak scaling of MPI distribution, keep `HOUSEHOLDS/HORIZON/REPETITIONS` fixed and scale `FIN_INDIVIDUALS` with rank count.

#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."

repo_path="/home/biscuit/cs470-krAB"

echo "Starting Slurm Docker cluster..."
./scripts/start.sh

if ! docker compose exec -T controller test -d "${repo_path}/examples/financial_life_exploration"; then
  echo "Repository is not present in ${repo_path}."
  echo "Run ./scripts/setup-ssh-login.sh and recreate the containers so the host repo bind mount is active."
else
  echo "Repository already exists in ${repo_path}."
fi

echo
echo "Cluster login node is ready."
echo "Inside the shell, use:"
echo "  cd ${repo_path}"
echo "  cd examples/financial_life_exploration"
echo "  cargo build --release --features \"distributed_mpi mpi_verbose_timing\""
echo "  FIN_INDIVIDUALS=32 FIN_MAX_GENERATION=5 FIN_HOUSEHOLDS=8 FIN_REPETITIONS=2 srun -N 2 --ntasks=2 --mpi=pmi2 ../target/release/finance_life_exploration"
echo

docker compose exec controller bash -l

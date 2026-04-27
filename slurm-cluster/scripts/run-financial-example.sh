#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."

tasks="${1:-2}"
nodes="${2:-2}"

if ! docker compose exec -T controller test -x /shared/cs470-krAB/examples/target/release/finance_life_exploration; then
  ./scripts/build-financial-example.sh
fi

docker compose exec -T controller bash -lc "
set -euo pipefail
cd /shared/cs470-krAB/examples/financial_life_exploration
FIN_INDIVIDUALS=\${FIN_INDIVIDUALS:-32} \
FIN_MAX_GENERATION=\${FIN_MAX_GENERATION:-100} \
FIN_HOUSEHOLDS=\${FIN_HOUSEHOLDS:-8} \
FIN_REPETITIONS=\${FIN_REPETITIONS:-20} \
srun -N ${nodes} --ntasks=${tasks} --mpi=pmi2 ../target/release/finance_life_exploration
"

#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."

if ! docker compose exec -T controller test -d /shared/cs470-krAB/examples/financial_life_exploration; then
  ./scripts/sync-repo.sh
fi

docker compose exec -T controller bash -lc '
set -euo pipefail
cd /shared/cs470-krAB/examples/financial_life_exploration
echo "LIBCLANG_PATH=${LIBCLANG_PATH}"
echo "BINDGEN_EXTRA_CLANG_ARGS=${BINDGEN_EXTRA_CLANG_ARGS}"
cargo build --release --features "distributed_mpi mpi_verbose_timing"
'

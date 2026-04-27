#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."

docker compose ps
echo
docker compose exec -T controller sinfo
echo
docker compose exec -T controller squeue

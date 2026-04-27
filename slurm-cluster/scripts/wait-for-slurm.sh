#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."

echo "Waiting for Slurm workers to report idle..."
for _ in $(seq 1 90); do
  node_states="$(docker compose exec -T controller sinfo -N -h -o '%n %t' 2>/dev/null || true)"
  if grep -q 'worker01 idle' <<<"${node_states}" && grep -q 'worker02 idle' <<<"${node_states}"; then
    exit 0
  fi

  docker compose exec -T controller sinfo 2>/dev/null || true
  sleep 2
done

echo "Timed out waiting for worker01 and worker02 to become idle." >&2
docker compose ps >&2 || true
docker compose logs --no-color controller worker01 worker02 >&2 || true
exit 1

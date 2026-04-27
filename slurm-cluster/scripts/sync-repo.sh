#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."

./scripts/start.sh

docker compose exec -T controller rm -rf /shared/cs470-krAB
docker compose cp .. controller:/shared/cs470-krAB

echo "Repository copied into controller:/shared/cs470-krAB"

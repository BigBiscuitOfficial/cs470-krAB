#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."

service="${1:-controller}"
docker compose exec "${service}" bash

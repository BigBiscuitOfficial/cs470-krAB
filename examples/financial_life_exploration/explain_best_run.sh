#!/bin/bash
set -euo pipefail

if [ $# -lt 1 ]; then
  echo "usage: $0 <run-log>"
  exit 1
fi

LOG_FILE="$1"

if [ ! -f "$LOG_FILE" ]; then
  echo "log file not found: $LOG_FILE"
  exit 1
fi

GENOME="$(awk -F'string:[[:space:]]*' '/string:/ {print $2}' "$LOG_FILE" | tail -n 1 | tr -d '\r')"
CONFIG_LINE="$(grep -m1 '^Scale config:' "$LOG_FILE" || true)"

if [ -z "$GENOME" ]; then
  echo "could not find best policy genome in $LOG_FILE"
  exit 1
fi

if [ -n "$CONFIG_LINE" ]; then
  for key in households horizon individuals max_generation repetitions seed; do
    value="$(printf '%s\n' "$CONFIG_LINE" | sed -n "s/.*$key=\\([0-9][0-9]*\\).*/\\1/p")"
    if [ -n "$value" ]; then
      case "$key" in
        households) export FIN_HOUSEHOLDS="$value" ;;
        horizon) export FIN_HORIZON="$value" ;;
        individuals) export FIN_INDIVIDUALS="$value" ;;
        max_generation) export FIN_MAX_GENERATION="$value" ;;
        repetitions) export FIN_REPETITIONS="$value" ;;
        seed) export FIN_SEED="$value" ;;
      esac
    fi
  done
fi

export FIN_EXPLAIN_GENOME="$GENOME"
mpirun -n 1 cargo run --release --features distributed_mpi

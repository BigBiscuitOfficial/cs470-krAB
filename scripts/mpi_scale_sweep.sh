#!/usr/bin/env bash
set -euo pipefail

# Large-workload MPI scaling sweep via Docker cluster.
# Produces one run directory per NP and consolidates profiling CSVs.

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
OUT_ROOT="${KRAB_SWEEP_OUTPUT_ROOT:-$ROOT_DIR/output/mpi_scale_sweep}"
CONFIG_PATH="${KRAB_CONFIG_PATH:-examples/config_comprehensive.json}"
NP_VALUES="${KRAB_MPI_NP_VALUES:-2,4}"

if [[ "$OUT_ROOT" != /* ]]; then
  OUT_ROOT="$ROOT_DIR/$OUT_ROOT"
fi

if [[ "$OUT_ROOT" = "$ROOT_DIR"/* ]]; then
  OUT_ROOT_REL="${OUT_ROOT#$ROOT_DIR/}"
else
  echo "KRAB_SWEEP_OUTPUT_ROOT must be inside repo root: $ROOT_DIR" >&2
  echo "Current value: $OUT_ROOT" >&2
  exit 1
fi

mkdir -p "$OUT_ROOT"
timing_csv="$OUT_ROOT/mpi_sweep_timings.csv"
echo "np,elapsed_seconds,run_dir" > "$timing_csv"

echo "MPI scale sweep"
echo "- root: $ROOT_DIR"
echo "- config: $CONFIG_PATH"
echo "- np values: $NP_VALUES"
echo "- output root: $OUT_ROOT"

for np in ${NP_VALUES//,/ }; do
  run_out="$OUT_ROOT/np${np}"
  run_out_rel="$OUT_ROOT_REL/np${np}"
  echo ""
  echo "==> Running MPI sweep at NP=$np"
  t0_ns=$(date +%s%N)
  KRAB_MPI_NP="$np" \
  KRAB_CONFIG_PATH="$CONFIG_PATH" \
  KRAB_OUTPUT_DIR="$run_out_rel" \
  "$ROOT_DIR/run_mpi_docker.sh"
  t1_ns=$(date +%s%N)

  elapsed_s=$(python3 - <<PY
t0 = int("$t0_ns")
t1 = int("$t1_ns")
print(f"{(t1 - t0) / 1_000_000_000.0:.6f}")
PY
)

  latest_run=$(python3 - <<PY
import glob
import os

base = "$run_out"
cands = glob.glob(os.path.join(base, "financial_mpi_*"))
if not cands:
    print("")
else:
    cands.sort(key=os.path.getmtime, reverse=True)
    print(cands[0])
PY
)
  echo "$np,$elapsed_s,$latest_run" >> "$timing_csv"
done

echo ""
echo "Consolidating profiling CSV files..."
shopt -s nullglob
profiling_files=(
  "$OUT_ROOT"/np*/profiling/profiling_*.csv
  "$OUT_ROOT"/np*/financial_mpi_*/profiling/profiling_*.csv
)
shopt -u nullglob

if [ "${#profiling_files[@]}" -gt 0 ]; then
  python3 "$ROOT_DIR/scripts/analysis/parse_metrics.py" \
    "${profiling_files[@]}" \
    --baseline mpi \
    --output "$OUT_ROOT/consolidated_metrics.csv"
  echo "- consolidated metrics: $OUT_ROOT/consolidated_metrics.csv"
else
  echo "No profiling CSV files found for MPI runs; skipping parse_metrics consolidation."
fi

echo "Done."
echo "- sweep timings: $timing_csv"

graph_svg="$OUT_ROOT/mpi_scaling.svg"
python3 "$ROOT_DIR/scripts/analysis/plot_mpi_scaling.py" "$timing_csv" --output "$graph_svg" >/dev/null
echo "- scaling graph: $graph_svg"

speedup_svg="$OUT_ROOT/mpi_speedup_efficiency.svg"
python3 "$ROOT_DIR/scripts/analysis/plot_mpi_speedup_efficiency.py" "$timing_csv" --output "$speedup_svg" >/dev/null
echo "- speedup/efficiency graph: $speedup_svg"

report_md="$OUT_ROOT/mpi_scaling_report.md"
python3 "$ROOT_DIR/scripts/analysis/generate_mpi_scaling_report.py" \
  "$timing_csv" \
  --scaling-svg "$graph_svg" \
  --speedup-svg "$speedup_svg" \
  --output "$report_md" >/dev/null
echo "- markdown report: $report_md"

echo "- per-run artifacts: $OUT_ROOT/np*/"

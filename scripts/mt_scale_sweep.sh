#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
OUT_ROOT="${KRAB_MT_SWEEP_OUTPUT_ROOT:-$ROOT_DIR/output/mt_scale_sweep}"
CONFIG_PATH="${KRAB_CONFIG_PATH:-/tmp/krab-medium.json}"
THREAD_VALUES="${KRAB_MT_THREADS:-2,4,8}"

if [[ "$OUT_ROOT" != /* ]]; then
  OUT_ROOT="$ROOT_DIR/$OUT_ROOT"
fi

mkdir -p "$OUT_ROOT"

timings_csv="$OUT_ROOT/mt_sweep_timings.csv"
echo "mode,threads,elapsed_seconds,sweep_total_seconds,strategy_total_sum_seconds,strategy_pure_sum_seconds,run_dir,profiling_csv" > "$timings_csv"

accuracy_csv="$OUT_ROOT/mt_accuracy.csv"
echo "threads,max_abs_delta_median,max_abs_delta_p10,max_abs_delta_p90,max_abs_delta_bankruptcy_rate,max_abs_delta_success_rate" > "$accuracy_csv"

echo "MT scale sweep"
echo "- config: $CONFIG_PATH"
echo "- threads: $THREAD_VALUES"
echo "- output: $OUT_ROOT"

run_mode() {
  local mode="$1"
  local threads="$2"
  local out_dir="$OUT_ROOT/${mode}_${threads}t"

  local t0_ns
  t0_ns=$(date +%s%N)

  if [[ "$mode" == "serial" ]]; then
    KRAB_OUTPUT_DIR="$out_dir" KRAB_CONFIG_PATH="$CONFIG_PATH" cargo run --release --example financial_serial
  else
    KRAB_OUTPUT_DIR="$out_dir" KRAB_CONFIG_PATH="$CONFIG_PATH" KRAB_THREAD_COUNT="$threads" \
      cargo run --release --features parallel --example financial_multithreaded
  fi

  local t1_ns
  t1_ns=$(date +%s%N)
  local elapsed_s
  elapsed_s=$(python3 - <<PY
t0 = int("$t0_ns")
t1 = int("$t1_ns")
print(f"{(t1 - t0) / 1_000_000_000.0:.6f}")
PY
)

  local run_dir
  run_dir=$(python3 - <<PY
import glob
import os
base = "$out_dir"
mode = "$mode"
prefix = "financial_serial_*" if mode == "serial" else "financial_multithreaded_*"
cands = glob.glob(os.path.join(base, prefix))
if not cands:
    print("")
else:
    cands.sort(key=os.path.getmtime, reverse=True)
    print(cands[0])
PY
)

  local profiling_csv
  profiling_csv=$(python3 - <<PY
import glob
import os
base = "$out_dir/profiling"
cands = glob.glob(os.path.join(base, "profiling_*.csv"))
if not cands:
    print("")
else:
    cands.sort(key=os.path.getmtime, reverse=True)
    print(cands[0])
PY
)

  local sweep_total
  local strategy_total_sum
  local strategy_pure_sum
  read -r sweep_total strategy_total_sum strategy_pure_sum < <(python3 - <<PY
import csv
path = "$profiling_csv"
if not path:
    print("", "", "")
else:
    try:
        sweep = ""
        strategy_sum = 0.0
        strategy_pure_sum = 0.0
        with open(path, "r", encoding="utf-8") as f:
            for row in csv.DictReader(f):
                if row.get("event") == "sweep_total":
                    sweep = row.get("total_runtime_s", "")
                if row.get("event") == "strategy_total":
                    try:
                        strategy_sum += float(row.get("total_runtime_s", "0") or 0)
                        strategy_pure_sum += (
                            float(row.get("init_time_s", "0") or 0)
                            + float(row.get("step_compute_s", "0") or 0)
                            + float(row.get("metrics_calc_s", "0") or 0)
                        )
                    except ValueError:
                        pass
        print(sweep, f"{strategy_sum:.6f}", f"{strategy_pure_sum:.6f}")
    except FileNotFoundError:
        print("", "", "")
PY
)

  echo "$mode,$threads,$elapsed_s,$sweep_total,$strategy_total_sum,$strategy_pure_sum,$run_dir,$profiling_csv" >> "$timings_csv"
}

run_mode serial 1

serial_summary=$(python3 - <<PY
import glob
import os
base = "$OUT_ROOT/serial_1t"
cands = glob.glob(os.path.join(base, "financial_serial_*", "summary.json"))
if not cands:
    print("")
else:
    cands.sort(key=os.path.getmtime, reverse=True)
    print(cands[0])
PY
)

for t in ${THREAD_VALUES//,/ }; do
  run_mode mt "$t"

  mt_summary=$(python3 - <<PY
import glob
import os
base = "$OUT_ROOT/mt_${t}t"
cands = glob.glob(os.path.join(base, "financial_multithreaded_*", "summary.json"))
if not cands:
    print("")
else:
    cands.sort(key=os.path.getmtime, reverse=True)
    print(cands[0])
PY
)

  python3 - <<PY >> "$accuracy_csv"
import json

serial_path = "$serial_summary"
mt_path = "$mt_summary"
threads = "$t"

if not serial_path or not mt_path:
    print(f"{threads},,,,,")
else:
    with open(serial_path, "r", encoding="utf-8") as f:
        s_rows = json.load(f)
    with open(mt_path, "r", encoding="utf-8") as f:
        m_rows = json.load(f)

    s_map = {row["strategy_desc"]: row for row in s_rows}
    m_map = {row["strategy_desc"]: row for row in m_rows}
    keys = sorted(set(s_map.keys()) & set(m_map.keys()))
    if not keys:
        print(f"{threads},,,,,")
    else:
        def max_delta(field):
            return max(abs(float(s_map[k][field]) - float(m_map[k][field])) for k in keys)

        print(
            f"{threads},{max_delta('median_net_worth'):.6f},{max_delta('p10_net_worth'):.6f},{max_delta('p90_net_worth'):.6f},{max_delta('bankruptcy_rate'):.6f},{max_delta('successful_retirement_rate'):.6f}"
        )
PY
done

graph_svg="$OUT_ROOT/mt_speedup_efficiency.svg"
python3 "$ROOT_DIR/scripts/analysis/plot_mt_speedup_efficiency.py" "$timings_csv" --output "$graph_svg" >/dev/null

time_log_graph="$OUT_ROOT/mt_time_log_threads.svg"
python3 "$ROOT_DIR/scripts/analysis/plot_mt_time_log_threads.py" "$timings_csv" --output "$time_log_graph" >/dev/null

accuracy_graph="$OUT_ROOT/mt_accuracy.svg"
python3 "$ROOT_DIR/scripts/analysis/plot_mt_accuracy.py" "$accuracy_csv" --output "$accuracy_graph" >/dev/null

report_md="$OUT_ROOT/mt_scaling_report.md"
python3 "$ROOT_DIR/scripts/analysis/generate_mt_scaling_report.py" \
  "$timings_csv" \
  --graph "$graph_svg" \
  --time-log-graph "$time_log_graph" \
  --accuracy-csv "$accuracy_csv" \
  --accuracy-graph "$accuracy_graph" \
  --output "$report_md" >/dev/null

echo "Done."
echo "- timings: $timings_csv"
echo "- accuracy: $accuracy_csv"
echo "- graph: $graph_svg"
echo "- time-log graph: $time_log_graph"
echo "- accuracy graph: $accuracy_graph"
echo "- report: $report_md"

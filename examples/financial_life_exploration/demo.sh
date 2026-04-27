#!/bin/bash

module load mpi

export LIBCLANG_PATH=/shared/common/clang+llvm-14.0.0/lib/
export BINDGEN_EXTRA_CLANG_ARGS="-I/usr/lib/gcc/x86_64-redhat-linux/8/include"

mkdir -p runs outcomes
cp parsetocsv.py runs/parsetocsv.py

# INDIVIDUALS MUST BE >= NUM PROCESSES
# defaults
export FIN_SEED=1234
export FIN_HORIZON=30
export FIN_REPETITIONS=4
export FIN_MAX_GENERATION=12
export FIN_HOUSEHOLDS=24
export FIN_INDIVIDUALS=64
export DEMO_PROCS="${DEMO_PROCS:-1 2 4 8 16 32 64}"



SUMMARY_FILE="outcomes/demo_summary.tsv"
DEMO_DAY_SUMMARY_FILE="outcomes/demo_day_summary.txt"

echo "Building financial_life_exploration..."
cargo build --release --features "distributed_mpi mpi_verbose_timing"

echo -e "procs\trun_elapsed_seconds\tmpi_avg_total_seconds\tspeedup\tefficiency\tavg_overhead_ratio\tbest_fitness\tinterpretation_status" > "$SUMMARY_FILE"
baseline_file=""
baseline_mpi_avg_total=""
baseline_procs=""

echo "Demo configuration:"
echo "  FIN_SEED=$FIN_SEED"
echo "  FIN_HORIZON=$FIN_HORIZON"
echo "  FIN_REPETITIONS=$FIN_REPETITIONS"
echo "  FIN_MAX_GENERATION=$FIN_MAX_GENERATION"
echo "  FIN_HOUSEHOLDS=$FIN_HOUSEHOLDS"
echo "  FIN_INDIVIDUALS=$FIN_INDIVIDUALS"
echo "  DEMO_PROCS=$DEMO_PROCS"
echo

for i in $DEMO_PROCS
do
    log_file="runs/finance_life_run_${i}_procs.log"
    interpretation_file="outcomes/financial_interpretation_${i}_procs.txt"
    diff_file="outcomes/financial_interpretation_${i}_procs.diff"

    echo "Running with ${i} MPI process(es)..."
    salloc -Q -n "$i" mpirun ../target/release/finance_life_exploration > "$log_file" 2>&1


    python3 tools/interpret_financial_run.py "$log_file" -o "$interpretation_file"

    metrics=$(python3 tools/demo_day_summary.py metrics "$log_file")
    IFS=$'\t' read -r mpi_avg_total avg_overhead total_run best_fitness <<< "$metrics"

    if [ -z "$baseline_mpi_avg_total" ]; then
        baseline_mpi_avg_total="$mpi_avg_total"
        baseline_procs="$i"
    fi

    speedup=$(python3 tools/demo_day_summary.py speedup "$baseline_mpi_avg_total" "$mpi_avg_total")
    efficiency=$(
        python3 tools/demo_day_summary.py efficiency "$speedup" "$i" "$baseline_procs"
    )

    if [ -z "$baseline_file" ]; then
        baseline_file="$interpretation_file"
        interpretation_status="baseline"
        printf "Baseline interpretation: %s\n" "$baseline_file" > "$diff_file"
    elif cmp -s "$baseline_file" "$interpretation_file"; then
        interpretation_status="matches_baseline"
        printf "No differences from baseline (%s)\n" "$baseline_file" > "$diff_file"
    else
        interpretation_status="DIFFERS_FROM_BASELINE"
        diff -u "$baseline_file" "$interpretation_file" > "$diff_file" || true
    fi

    echo -e "${i}\t${total_run}\t${mpi_avg_total}\t${speedup}\t${efficiency}\t${avg_overhead}\t${best_fitness}\t${interpretation_status}" >> "$SUMMARY_FILE"
    echo "Completed ${i} proc run in ${total_run}s: speedup=${speedup}, efficiency=${efficiency}, best_fitness=${best_fitness} (${interpretation_status})"
    echo
done

python3 tools/demo_day_summary.py summary "$SUMMARY_FILE" "$baseline_file" "$DEMO_DAY_SUMMARY_FILE" "$FIN_SEED"

echo "Seed used: ${FIN_SEED}"
echo "Summary written to ${SUMMARY_FILE}"
echo "Demo-day summary written to ${DEMO_DAY_SUMMARY_FILE}"
echo "Interpretations written to outcomes/"
echo
cat "$DEMO_DAY_SUMMARY_FILE"
echo
echo "Interpretation consistency summary:"
for i in $DEMO_PROCS
do
    diff_file="outcomes/financial_interpretation_${i}_procs.diff"
    if grep -q "Baseline interpretation" "$diff_file"; then
        echo "  ${i} proc(s): baseline interpretation"
    elif grep -q "No differences from baseline" "$diff_file"; then
        echo "  ${i} proc(s): interpretation matches baseline"
    else
        echo "  ${i} proc(s): interpretation differs from baseline"
        echo "    See ${diff_file}"
    fi
done

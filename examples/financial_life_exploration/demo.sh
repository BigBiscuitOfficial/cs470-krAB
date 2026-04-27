#!/bin/bash
module load mpi

export LIBCLANG_PATH=/shared/common/clang+llvm-14.0.0/lib/
export BINDGEN_EXTRA_CLANG_ARGS="-I/usr/lib/gcc/x86_64-redhat-linux/8/include"

mkdir -p runs outcomes

# defaults
export FIN_SEED=1234
export FIN_HORIZON=30
export FIN_REPETITIONS=4
export FIN_MAX_GENERATION=12
export FIN_HOUSEHOLDS=24
export FIN_INDIVIDUALS=64


SUMMARY_FILE="outcomes/demo_summary.tsv"

echo "CLEANING BUILD CACHE"
cargo clean

echo "Building financial_life_exploration..."
cargo build --release --features "distributed_mpi mpi_verbose_timing"

echo -e "procs\telapsed_seconds\tinterpretation_status" > "$SUMMARY_FILE"
baseline_file=""

echo "Demo configuration:"
echo "  FIN_SEED=$FIN_SEED"
echo "  FIN_HORIZON=$FIN_HORIZON"
echo "  FIN_REPETITIONS=$FIN_REPETITIONS"
echo "  FIN_MAX_GENERATION=$FIN_MAX_GENERATION"
echo "  FIN_HOUSEHOLDS=$FIN_HOUSEHOLDS"
echo "  FIN_INDIVIDUALS=$FIN_INDIVIDUALS"
echo

for i in 1 2 4 8 16 32 64 128
do
    log_file="runs/finance_life_run_${i}_procs.log"
    interpretation_file="outcomes/financial_interpretation_${i}_procs.txt"

    echo "Running with ${i} MPI process(es)..."
    start_time=$SECONDS
    salloc -Q -n "$i" mpirun ../target/release/finance_life_exploration 2>&1 | tee "$log_file"
    elapsed=$((SECONDS - start_time))

    python3 tools/interpret_financial_run.py "$log_file" -o "$interpretation_file"

    if [ -z "$baseline_file" ]; then
        baseline_file="$interpretation_file"
        interpretation_status="baseline"
    elif cmp -s "$baseline_file" "$interpretation_file"; then
        interpretation_status="matches_baseline"
    else
        interpretation_status="DIFFERS_FROM_BASELINE"
    fi

    echo -e "${i}\t${elapsed}\t${interpretation_status}" >> "$SUMMARY_FILE"
    echo "Completed ${i} proc run in ${elapsed}s (${interpretation_status})"
    echo
done

echo "Seed used: ${FIN_SEED}"
echo "Summary written to ${SUMMARY_FILE}"
echo "Interpretations written to outcomes/"

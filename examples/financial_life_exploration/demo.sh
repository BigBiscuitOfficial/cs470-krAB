#!/bin/bash

module load mpi

mkdir -p outcomes/

cargo build --release --features "distributed_mpi mpi_verbose_timing"

export LIBCLANG_PATH=/shared/common/clang+llvm-14.0.0/lib/
export BINDGEN_EXTRA_CLANG_ARGS="-I/usr/lib/gcc/x86_64-redhat-linux/8/include"


export FIN_SEED=$(echo $RANDOM) 
export FIN_HORIZON=60 
export FIN_REPETITIONS=20
export FIN_MAX_GENERATION=75
export FIN_HOUSEHOLDS=48
export FIN_INDIVIDUALS=128

for i in 1 2 4 8 16 32 64 128
do
salloc -Q -n $i mpirun ../target/release/finance_life_exploration | tee runs/finance_life_run_${i}_procs.log

python3 tools/interpret_financial_run.py runs/finance_life_run_${i}_procs.log -o "outcomes/financial_interpretation_${i}_procs.txt"

echo "SEED WAS: ${FIN_SEED}"
done





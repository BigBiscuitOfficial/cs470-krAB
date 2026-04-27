# Setup Instructions

## First install rustup
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```
## Export or add to bashrc/zshrc
```bash
export LIBCLANG_PATH=/shared/common/clang+llvm-14.0.0/lib/
export BINDGEN_EXTRA_CLANG_ARGS="-I/usr/lib/gcc/x86_64-redhat-linux/8/include"
```

## Build the financial example
*The executable will be in ./examples/financial_life_exploration/target/release/*
```bash
cd examples/financial_life_exploration
cargo build --release --features "distributed_mpi verbose_mpi_timing"
```
*Include "parallel" in the features block for multi-threading (in progress)*


## To run
```bash 
salloc -Q -n <nprocs> mpirun finance_life_exploration
```
If you wish to test scaling, it's best to use the provided scripts as a template (run_weak/run_strong)


Adjustable environmental variables

FIN_HORIZON=60 
FIN_REPETITIONS=8
FIN_MAX_GENERATION=150
FIN_HOUSEHOLDS=48
FIN_INDIVIDUALS=128
*Individuals is the most relevant to test scaling due to the MPI and multi-threading implementations*

#!/bin/bash
# Full sweep for scaling comparison.
module load mpi
RANKS=(1 8 16 32 64 128)
KRAB_EXE="./target/release/examples/financial_mpi"

salloc -Q -n 1 mpirun $KRAB_EXE
salloc -Q -n 2 mpirun $KRAB_EXE
salloc -Q -n 4 mpirun $KRAB_EXE
salloc -Q -n 8 mpirun $KRAB_EXE
salloc -Q -n 16 mpirun $KRAB_EXE
salloc -Q -n 32 mpirun $KRAB_EXE
salloc -Q -n 64 mpirun $KRAB_EXE
salloc -Q -n 128 mpirun $KRAB_EXE


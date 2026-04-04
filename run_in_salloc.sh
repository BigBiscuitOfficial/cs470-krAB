#!/bin/bash

module load mpi
# Define the number of processes to test
PROCS=(1 2 4 8 16 32 64)

# Path to your executable
EXE="./target/release/examples/financial_mpi"

echo "Starting MPI performance runs..."
echo "--------------------------------"

for np in "${PROCS[@]}"
do
    echo "Running with np = $np..."
    
    # Executing the MPI command
    # time is used to capture the wall-clock duration of each run
    { time mpirun -np $np $EXE > /dev/null; } 2>> time_results.txt
    echo "Completed np = $np"
    echo "--------------------------------"
done

echo "All runs finished."

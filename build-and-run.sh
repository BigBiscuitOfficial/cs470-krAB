#!/bin/bash 


#mfs always tryna ice skate uphill


rm -f ./slurm/financial_serial ./slurm/financial_mpi
module load mpi

echo "Building Serial Example"
cargo build --release --example financial_serial
echo "SERIAL BUILD COMPLETE"
echo ""
echo "Building MPI Example"
cargo build --release --features distributed_mpi --example financial_mpi
echo "MPI BUILD COMPLETE"


echo "Running simulations..."
./run_in_salloc.sh > /dev/null


echo "Simulations complete, collecting data..."
./collect_results.sh

#!/bin/bash

CSV_FILE="./output/scaling_results/mpi/scaling_results.csv"

if [ ! -f "$CSV_FILE" ]; then
    echo "Error: $CSV_FILE not found."
    exit 1
fi

echo "----------------------------------------------------------"
echo -e "Ranks\tTime (s)\tSpeedup\t\tEfficiency"
echo "----------------------------------------------------------"

awk -F',' '
NR > 1 {
    ranks = $3
    time = $14
    
    # Store the time of the first entry (1 rank) as the baseline
    if (NR == 2) {
        t1 = time
    }
    
    speedup = t1 / time
    efficiency = speedup / ranks
    
    printf "%d\t%.4f\t\t%.2fx\t\t%.2f%%\n", ranks, time, speedup, efficiency * 100
}' "$CSV_FILE"


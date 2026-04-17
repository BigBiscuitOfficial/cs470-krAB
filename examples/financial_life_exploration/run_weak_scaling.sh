#!/bin/bash

module load mpi

# Keep these strictly constant for weak scaling
export FIN_HORIZON=60 
export FIN_REPETITIONS=8
export FIN_MAX_GENERATION=150
export FIN_HOUSEHOLDS=48



for i in 256 128 64 32 16 8 4 2 1
do
  # Scale households linearly with the number of processors

  export FIN_INDIVIDUALS=$((i * 1000))
  
  echo "Running MPI with $i tasks, $FIN_INDIVIDUALS individuals..."
  
  # Run the simulation and pipe the output to a specific log file
  salloc -Q -n $i mpirun ../target/release/finance_life_exploration 2>&1 | tee "scalingweak_run_${i}_procs.log"
  
  echo "Finished $i tasks. Log saved to scaling_run_${i}_procs.log"
  echo "---------------------------------------------------"
done

#!/bin/bash
mkdir -p scale-strong-res
cp parsetocsv.py scale-strong-res/parsetocsv.py
./build.sh

module load mpi

rm ./scale-strong-res/scaling_run_*

# Keep these strictly constant for weak scaling
export FIN_HORIZON=60 
export FIN_REPETITIONS=8
export FIN_MAX_GENERATION=150
export FIN_HOUSEHOLDS=48
export FIN_INDIVIDUALS=128



# when people aint being resource hogs
# for i in 1 2 4 8 16 32 64 128 256
#
for i in  1 2 4 8 16 32 64 128
do
  # Scale households linearly with the number of processors


  
  echo "Running MPI with $i tasks, $FIN_INDIVIDUALS individuals..."
  
  # Run the simulation and pipe the output to a specific log file
  salloc -Q -n $i mpirun ../target/release/finance_life_exploration 2>&1 | tee "scale-strong-res/scaling_run_${i}_procs.log"
  
  echo "Finished $i tasks. Log saved to ./scale-strong-res/scaling_run_${i}_procs.log"
  echo "---------------------------------------------------"
done

echo
echo "Parsing results to CSV"
echo "---------------------------------------------------"
cd scale-strong-res
python3 parsetocsv.py 

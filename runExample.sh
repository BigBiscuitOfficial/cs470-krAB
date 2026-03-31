#!/bin/bash
#
#SBATCH --job-name=NAME
#SBATCH --ntasks=NTASKS

module load mpi
srun EXENAME

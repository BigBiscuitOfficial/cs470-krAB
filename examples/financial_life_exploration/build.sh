#!/bin/bash

module load mpi

# Distributed MPI 
cargo build --release --features "distributed_mpi mpi_verbose_timing"


# Distributed MPI + Multi-threading
# cargo build --release --features "distributed_mpi parallel mpi_verbose_timing"

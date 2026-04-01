#!/bin/bash
set -e

KRAB_MPI_NP="${KRAB_MPI_NP:-2}"

# Navigate to the project root
cd "$(dirname "$0")"

echo "Building and starting Docker MPI cluster..."
cd mpi-cluster-docker
docker compose up -d --build

echo "Compiling MPI program inside the master container..."
docker compose exec -u mpiuser master bash -c "cd /home/mpiuser/workdir && cargo build --release --example financial_mpi --features distributed_mpi"

echo "Running MPI program inside the master container across ${KRAB_MPI_NP} ranks..."
docker compose exec -u mpiuser \
  -e KRAB_CONFIG_PATH \
  -e KRAB_OUTPUT_DIR \
  -e KRAB_MPI_NP \
  master \
  bash -c "cd /home/mpiuser/workdir && export KRAB_OUTPUT_DIR=\${KRAB_OUTPUT_DIR:-/home/mpiuser/workdir/output/mpi_run} && export KRAB_MPI_NP=\${KRAB_MPI_NP:-2} && mkdir -p \$KRAB_OUTPUT_DIR && mpirun -np \$KRAB_MPI_NP ./target/release/examples/financial_mpi"

echo ""
echo "Done! The cluster is still running in the background."
echo "You can view the outputs in mpi-cluster-docker/workdir/output/mpi_run"
echo "To stop the cluster, run: cd mpi-cluster-docker && docker compose down"

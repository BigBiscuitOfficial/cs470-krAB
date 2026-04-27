#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."

./scripts/start.sh

docker compose exec -T controller sinfo

docker compose exec -T controller bash -lc "cat >/shared/hello_mpi.c" <<'C_EOF'
#include <mpi.h>
#include <stdio.h>
#include <unistd.h>

int main(int argc, char **argv) {
    MPI_Init(&argc, &argv);

    int rank = -1;
    int size = -1;
    char host[256];
    gethostname(host, sizeof(host));

    MPI_Comm_rank(MPI_COMM_WORLD, &rank);
    MPI_Comm_size(MPI_COMM_WORLD, &size);

    printf("Hello from MPI rank %d of %d on %s\n", rank, size, host);

    MPI_Finalize();
    return 0;
}
C_EOF

docker compose exec -T controller mpicc /shared/hello_mpi.c -o /shared/hello_mpi
docker compose exec -T controller srun -N 2 --ntasks=2 --mpi=pmi2 /shared/hello_mpi

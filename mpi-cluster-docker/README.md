# Docker MPI Cluster

This sets up a simple virtual cluster using Docker Compose to run MPI applications.

It uses `ubuntu:22.04` and OpenMPI, generating a shared SSH key at build time so the master node can run passwordless SSH commands on the worker nodes.

## How to start it

1. Make sure you have Docker and Docker Compose installed.
2. Build and start the cluster:
   ```bash
   cd mpi-cluster
   docker-compose up -d --build
   ```

## How to use it

1. SSH into the master node (we mapped port 2222 on your local machine to port 22 on the master container):
   ```bash
   ssh -p 2222 mpiuser@localhost
   # Password is: mpi
   ```
   Or alternatively, run a bash shell directly using docker-compose:
   ```bash
   docker-compose exec master sudo -u mpiuser -i
   ```

2. Inside the master node, you can run the provided exampleMPI C program across the cluster:
   ```bash
   mpirun --hostfile workdir/hostfile -np 3 ./hello_mpi
   ```

## Shared Workspace

The `./workdir` directory on your host machine is mapped to `/home/mpiuser/workdir` inside all containers. You can compile your MPI code in the container and place the executable there, so all nodes can access it instantly.

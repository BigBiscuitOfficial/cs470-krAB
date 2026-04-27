# Rocky 8 Slurm Docker Cluster

This folder is a self-contained Docker Compose Slurm cluster. It is intentionally isolated from the rest of the repository so the Docker, Slurm, Munge, MariaDB, and MPI files stay separate from the project source.

The image is based on Rocky Linux 8 to approximate a RHEL 8.10 school environment. MPICH 4.2.0 is compiled from source in the Dockerfile with Slurm PMI support. Environment Modules is installed, so `module load mpi` loads this MPICH build.

The cluster also has a persistent shared `/home` volume mounted on the controller and both workers for login-node style work. Rebuilding the image can still require re-registering the Linux user with `setup-ssh-login.sh`, but files under `/home/<user>` survive container recreation and are visible to Slurm jobs on workers.

The image also includes Rust/Cargo and LLVM/Clang 14 for Rust MPI crates that use `bindgen`, matching the common cluster workaround:

```bash
LIBCLANG_PATH=/opt/llvm/clang+llvm-14.0.0-x86_64-linux-gnu-ubuntu-18.04/lib
BINDGEN_EXTRA_CLANG_ARGS="-I/usr/lib/gcc/x86_64-redhat-linux/8/include"
```

## Folder Layout

```text
slurm-cluster/
  Dockerfile
  docker-compose.yml
  README.md
  config/
    munge.key
    slurm.conf
    slurmdbd.conf
  scripts/
    build.sh
    start.sh
    wait-for-slurm.sh
    verify.sh
    sync-repo.sh
    build-financial-example.sh
    run-financial-example.sh
    join-cluster.sh
    setup-ssh-login.sh
    status.sh
    logs.sh
    shell.sh
    stop.sh
    reset.sh
    entrypoint.sh
```

Everything needed to build, run, verify, inspect, and tear down the emulated cluster is inside this directory.

## Architecture

Topology:

- `db`: MariaDB 10.11 for Slurm accounting.
- `controller`: runs `munged`, `slurmdbd`, and `slurmctld`.
- `worker01`, `worker02`: run `munged` and `slurmd`.
- SSH to the controller is exposed on `127.0.0.1:2222` for login-node style work.

Shared pieces:

- `config/munge.key` is mounted read-only into all Slurm containers so Munge authentication is synchronized.
- `shared-home` is a Docker volume mounted at `/shared` so compiled test binaries are visible to the controller and workers.
- `controller-home` is a Docker volume mounted at `/home` on the controller and workers so SSH keys, shell config, GitHub CLI auth, and user-installed files survive image rebuilds.
- The host repository is bind-mounted at `/home/biscuit/cs470-krAB` on controller and workers so Slurm jobs see the same project path everywhere.
- Slurm accounting is configured through `slurmdbd` and MariaDB.
- LLVM/Clang 14 is installed under `/opt/llvm` and exported for `bindgen`/`mpi-sys`.
- `module load mpi` loads MPICH 4.2.0 from `/opt/mpich`. MPICH is built with Slurm PMI support.
- `/opt/mpich/bin/mpiexec` and `/opt/mpich/bin/mpirun` are Slurm-native wrappers that launch through `srun --mpi=pmi2`, which is the correct launcher path for this emulated Slurm cluster.
- Base developer tools include Git, curl, vim, nano, tmux, CMake, gdb, strace, lsof, sudo, GitHub CLI, NVM, Node.js, Rust/Cargo, and the GNU compiler toolchain.

## Prerequisites

- Docker Engine with permission to access the Docker socket.
- Docker Compose v2 or newer through `docker compose`.
- Network access on first build to pull Rocky Linux, MariaDB, RPM packages, and the MPICH 4.2.0 source tarball.

Check locally:

```bash
docker --version
docker compose version
```

## Quick Verification

```bash
cd slurm-cluster
./scripts/verify.sh
```

`verify.sh` builds the image, starts the cluster, waits for both workers to become `idle`, runs `sinfo`, compiles `/shared/hello_mpi.c` with `mpicc`, and launches it across both workers:

```bash
srun -N 2 --ntasks=2 --mpi=pmi2 /shared/hello_mpi
```

Expected MPI output is similar to:

```text
Hello from MPI rank 0 of 2 on worker01
Hello from MPI rank 1 of 2 on worker02
```

## Scripts

Run all scripts from the repository root or from inside `slurm-cluster`; they resolve paths relative to this folder.

```bash
./scripts/build.sh            # Build the Rocky 8 Slurm/MPICH image.
./scripts/start.sh            # Build if needed, start services, wait for idle workers.
./scripts/verify.sh           # Full health check plus MPI Hello World through srun.
./scripts/join-cluster.sh     # Start the cluster, ensure the repo is in /shared, and open a controller shell.
./scripts/setup-ssh-login.sh  # Create an SSH login and print a ~/.ssh/config entry.
./scripts/sync-repo.sh        # Copy this repo into /shared/cs470-krAB.
./scripts/build-financial-example.sh # Build the MPI financial example in the controller.
./scripts/run-financial-example.sh   # Run the financial example with srun.
./scripts/status.sh           # Show Compose status, sinfo, and squeue.
./scripts/logs.sh             # Follow all service logs.
./scripts/logs.sh controller  # Follow one service log.
./scripts/shell.sh            # Open a shell on the controller.
./scripts/shell.sh worker01   # Open a shell on worker01.
./scripts/stop.sh             # Stop containers but keep Docker volumes.
./scripts/reset.sh            # Stop containers and delete cluster volumes.
```

## Manual MPI Test

After `./scripts/start.sh`, you can run commands inside the controller:

```bash
./scripts/shell.sh controller
module load mpi
module list
mpichversion
sinfo
mpicc /shared/hello_mpi.c -o /shared/hello_mpi
srun -N 2 --ntasks=2 --mpi=pmi2 /shared/hello_mpi
```

If `/shared/hello_mpi.c` does not exist yet, run `./scripts/verify.sh` once or create your own MPI source under `/shared`.

## Financial Example

The Docker cluster can run the financial example without changing the example source. The scripts copy the current repository into the shared Docker volume at `/shared/cs470-krAB`, build inside the controller, and run through Slurm.

```bash
cd slurm-cluster
./scripts/sync-repo.sh
./scripts/build-financial-example.sh
./scripts/run-financial-example.sh
```

If you want to work like you are on a cluster login node, use:

```bash
./scripts/join-cluster.sh
```

That starts the Slurm cluster, makes sure the repo exists under `/shared/cs470-krAB`, and opens a shell on `controller`.

## SSH Login

To make the controller feel more like a cluster login node:

```bash
cd slurm-cluster
./scripts/sync-repo.sh
./scripts/setup-ssh-login.sh
```

The setup script prompts for a username and password, creates that user on `controller`, `worker01`, and `worker02`, and prints a `~/.ssh/config` entry like:

```sshconfig
Host jmu-docker-slurm
  HostName localhost
  Port 2222
  User your_username
  StrictHostKeyChecking accept-new
```

Then connect from your host with:

```bash
ssh jmu-docker-slurm
```

Your repo is available inside the cluster at `/home/biscuit/cs470-krAB` on the controller and both workers.

The login user is added to the `wheel` group for sudo access. Test it after SSH login:

```bash
sudo whoami
```

Expected:

```text
root
```

The cluster's `/home` directory is persistent and shared by controller, worker01, and worker02. The host repo is also mounted at `/home/biscuit/cs470-krAB` on all nodes. After rebuilding/recreating containers, run `./scripts/setup-ssh-login.sh` again to recreate `/etc/passwd` entries while keeping `/home/your_username` intact.

By default `run-financial-example.sh` uses small runtime values so the local Docker cluster finishes quickly:

```text
FIN_INDIVIDUALS=32
FIN_MAX_GENERATION=5
FIN_HOUSEHOLDS=8
FIN_REPETITIONS=2
```

Override them when you want a larger run:

```bash
FIN_INDIVIDUALS=512 FIN_MAX_GENERATION=50 ./scripts/run-financial-example.sh 2 2
```

The first argument is task count and the second argument is node count:

```bash
./scripts/run-financial-example.sh 2 2
```

## Cleanup

Use `./scripts/stop.sh` when you want to pause the cluster and preserve database/state volumes.

Use `./scripts/reset.sh` when you want a clean Slurm/MariaDB state. This deletes Docker volumes for the cluster, including accounting data.

## Notes

- This is a local teaching/development cluster, not a secure production Slurm deployment.
- The Munge key in `config/munge.key` is intentionally static so every container shares credentials. Replace it if you adapt this for anything beyond local Docker.
- The first build is slow because MPICH 4.2.0 is compiled from source inside the Rocky 8 image.

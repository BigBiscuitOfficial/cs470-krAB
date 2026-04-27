# Slurm Docker Cluster Setup Steps

Quick reference for rebuilding, logging in, loading MPICH, and making the cluster feel like a school login node.

## 1. Build And Start

Run from the repository root:

```bash
cd slurm-cluster
./scripts/build.sh
./scripts/start.sh
```

Verify Slurm and MPI:

```bash
./scripts/verify.sh
```

## 2. Create SSH Login

Create or recreate your login user:

```bash
./scripts/setup-ssh-login.sh
```

Add the printed block to your host machine's `~/.ssh/config`.

Example:

```sshconfig
Host docker-cluster
  HostName localhost
  Port 2222
  User biscuit
  StrictHostKeyChecking accept-new
```

Then connect:

```bash
ssh docker-cluster
```

## 3. Load MPICH 4.2.0

Inside the cluster login shell:

```bash
module load mpi
which mpicc
which mpiexec
which mpirun
mpichversion | head
```

Expected path:

```text
/opt/mpich/bin/mpicc
```

The `mpiexec` and `mpirun` commands are Slurm-native wrappers that call `srun --mpi=pmi2`.

Expected version:

```text
MPICH Version: 4.2.0
```

## 4. Run A Slurm MPI Test

Inside the controller:

```bash
sinfo
cd /shared/cs470-krAB
srun -N 2 --ntasks=2 --mpi=pmi2 hostname
```

For the financial example:

```bash
cd /shared/cs470-krAB/slurm-cluster
./scripts/build-financial-example.sh
./scripts/run-financial-example.sh 2 2
```

## 5. User Data Survives Rebuilds

The controller and workers mount the same persistent Docker volume at `/home`, so files like shell config, SSH keys, `gh` auth, and local user tools survive image rebuilds. The host repo is bind-mounted at `/home/biscuit/cs470-krAB` on all nodes, so Slurm jobs see the same project path everywhere.

After a rebuild, rerun:

```bash
./scripts/setup-ssh-login.sh
```

That recreates the Linux account, while `/home/biscuit` stays intact.

## 6. Sudo Access

`setup-ssh-login.sh` adds the login user to `wheel` and enables password sudo.

After login setup:

```bash
ssh docker-cluster
sudo whoami
```

Expected:

```text
root
```

## 7. Base Dev Tools

The image includes:

```text
git
curl
vim
nano
tmux
unzip
zip
cmake
gdb
strace
lsof
openssh-clients
sudo
gh
nvm
node
npm
```

The image also has core build tools like `gcc`, `gcc-c++`, `make`, `wget`, `python3`, Rust/Cargo, LLVM/Clang 14, Slurm, Munge, and MPICH.

## 8. GitHub CLI

Inside SSH:

```bash
gh auth login
```

With persistent `/home`, your GitHub auth survives rebuilds.

## 9. NVM And Node

Inside SSH:

```bash
source /etc/profile
nvm --version
node --version
npm --version
```

## 10. Normal Workflow

After the persistent home and dev-tool changes are in place:

```bash
cd slurm-cluster
./scripts/build.sh
./scripts/start.sh
./scripts/setup-ssh-login.sh
ssh docker-cluster
```

Then inside the cluster:

```bash
module load mpi
cd /shared/cs470-krAB
```

For login-node style work, prefer:

```bash
cd /home/biscuit/cs470-krAB
```

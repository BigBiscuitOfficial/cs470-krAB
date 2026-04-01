
w3.cs.jmu.edu
CS 470 - JMU Cluster
15–19 minutes
Hardware

The CS 470 cluster is located in the Frye Building data center with the following hardware:

    9x Dell PowerEdge R6525 w/ 2x AMD EPYC 7252 (16C, 3.1 Ghz, HT) 64 GB – compute nodes
    8x Dell PowerEdge R6525 w/ 2x AMD EPYC 7252 (16C, 3.1 Ghz, HT) 64 GB and NVIDIA A2 GPU – compute nodes
    Dell PowerEdge R6525 w/ 2x AMD EPYC 7252 (16C, 3.1 Ghz, HT) 64 GB – login node
    Dell PowerEdge R730 w/ Xeon E5-2640v3 (16C, 2.6Ghz, HT) 32 GB – NFS server
    (in above) 8x 1.2TB 10K SAS HDD w/ RAID - storage
    Dell N3024 Switch 24x1GbE, 2xCombo, 2x10GbE SFP+

Software

All of the newer nodes are running RHEL8 with Slurm 20.11 for job management. An environment module is available for OpenMPI. Run module avail to see all available modules, and you can find additional software available in the /shared folder. In particular, you will find several useful utilities in /shared/cs470/bin, and I recommend either adding that folder to your PATH environment variable or making symlinks to a folder that is.

Several command-line text editors are installed by default, including nano, vim, and emacs.

If you need software that is not already installed or available via module, it is recommended that you build it from source in your home directory. Check the documentation for the software for instructions on how to do that. If you run into issues or your software is not available in source form, please email the system admin Pete Morris (morrispj) or the faculty contact Mike Lam (lam2mo) to request assistance.
On-campus Access

The login node of the newer cluster is accessible via SSH as login02.cluster.cs.jmu.edu from the campus network.

It is recommended that you set up public/private key SSH access from your most frequent point of access machines (e.g., your personal laptop). To do this, first generate a public/private keypair from a terminal if you have never done so on that machine:

ssh-keygen -t ed25519

If prompted, accept the default location and passphrase options by pressing enter twice. Then, copy the public key to the login node using one of the following commands based on your machine's operating system (run in a terminal open in your home folder):

(on Linux or macOS)
  ssh-copy-id <eid>@login02.cluster.cs.jmu.edu

(on Windows)
  type .ssh\id_rsa.pub | ssh <eid>@login02.cluster.cs.jmu.edu "cat >> .ssh/authorized_keys"

Now you won't need to enter your password every time you log in from that machine. Here is a slightly longer tutorial if you'd like to learn a bit more about this process.

It is also recommended that you edit your ~/.ssh/config file to add an SSH alias. Here is an example entry:

Host cluster
    HostName login02.cluster.cs.jmu.edu
    User <eid>

Now you can log into the cluster from your machine simply by typing this command:

ssh cluster

The firewall settings for our data center eventually "time out" idle SSH connections. To prevent this, you can add the following to your ~/.ssh/config:

TCPKeepAlive yes
ServerAliveInterval 15

Off-campus Access

If you are off-campus, you will need to proxy your SSH connection through an on-campus point of access (for CS students, this will probably be stu). To transparently proxy ssh sessions through stu, you can use the "-J" option if it is available:

ssh -J <eid>@stu.cs.jmu.edu <eid>@login02.cluster.cs.jmu.edu

Obviously, it is also recommended that you set up your ~/.ssh/config on your home machine so that you don't have to type all that every time. Assuming your SSH client supports it, you can even do this transparently by adding "ProxyJump stu" to the ~/.ssh/config configuration for the cluster host. Here is an example full SSH config file:

Host stu
    HostName stu.cs.jmu.edu
    User <eid>

Host cluster
    HostName login02.cluster.cs.jmu.edu
    User <eid>
    ProxyJump stu

Properly configured, you should be able to log into the cluster from off-campus very easily and without having to enter your password with the following command in a terminal:

ssh cluster

In addition, with all of the above properly configured the VS Code Remote SSH extension should allow you to write programs for CS 470 using a graphical IDE on your personal computer (of course you can always just use a command-line text editor on the login node as well -- see the Software section above).

For more information on proxies and jump hosts, see this Wikibook page.

If you are on Windows, you can also use PuTTY and WinSCP, both of which can be configured with public/private key access (the keys generated above will need to be converted to a .ppk file first, with WinSCP does automatically) and transparent proxying through stu. Other popular Windows SSH/SCP clients include Bitvise and MobaXterm.
Home Directories

If you are a student in CS 470, you should have an account already on the cluster, with a 250MB disk quota in your home directory (/nfs/home/[eid]). To check your disk usage, use the following command:

quota -s

If you need more space temporarily, use your designated scratch space (/scratch/[eid]). CAUTION: The scratch storage space may be wiped between semesters! If you need more permanent space, please contact your instructor or the cluster admin.

You can connect directly to your cluster home directory or scratch directory from a Linux lab machine:

    Open the file manager and select File -> Connect to server.
    Enter the following settings:

    Server:   login02.cluster.cs.jmu.edu
    Type:     ssh
    Folder:   /scratch/<eid> or /nfs/home/<eid>
    Username: <eid>
    Password: <eid password>

Transferring Files

If you need to transfer files back and forth between the cluster and another Unix-based machine (e.g., running Linux or macOS), you can use the scp command (here is a tutorial). If you are off campus, use the -o option to use stu as your jump host (e.g., -o 'ProxyJump stu.cs.jmu.edu' (and you should also consider adding stu to your SSH config as described above so that you can shorten the host name).

If you would prefer to use a graphical interface, I recommend FileZilla on Linux, CyberDuck on macOS, and WinSCP on Windows.

For a more seamless experience, you can also mount the remote filesystem locally using SSH. If you are doing this from off campus, use the following option to sshfs to jump through stu: -o ssh_command="ssh -J <eid>@stu.cs.jmu.edu"

Here is a script that you may find helpful: mount_cluster.sh
Submitting Interactive Jobs

You may use the login node to compile your programs and perform other incidental tasks. YOU SHOULD NOT EXECUTE HEAVY COMPUTATION ON THE LOGIN NODE! To properly run compute jobs, you must submit them using Slurm. You can find various Slurm tutorials on their website.

To run simple jobs interactively, use the srun command:

srun [Slurm options] /path/to/program [program options]

The most important Slurm options are the number of processes/tasks (-n) and the number of allocated nodes (-N). If not specified, the number of nodes will be set to the minimum number necessary to satisfy the process requirement.

The cluster has seventeen compute nodes, each of which has two eight-core AMD processors. Hyperthreading is enabled on the hardware but disabled in Slurm, so the maximum number of processes per node according to Slurm is sixteen. This minimizes unpredictable performance artifacts due to hyperthreading.

Here are some examples:

srun -n 4  hostname                         # 4  processes (single node)
srun -n 32 hostname                         # 32 processes (requires two nodes)
srun -N 4  hostname                         # 4  processes (4 nodes)
srun -N 4 -n 32 hostname                    # 32 processes (across 4 nodes)

Here are some examples of running an MPI program:

srun -n 1 /shared/cs470/mpi-hello/hello
srun -n 16 /shared/cs470/mpi-hello/hello
srun -n 32 /shared/cs470/mpi-hello/hello

If you'd like to open an interactive shell on a compute node for debugging purposes, you can do so using the following command (switch out "bash" if you prefer a different shell):

srun --pty /usr/bin/bash -i

Eight of the nodes have an NVIDIA A2 GPU suitable for running CUDA code. If you wish to take advantage of the GPUs, you must make sure your job is allocated to these nodes by adding --gres=gpu to the command line when you launch your job.

WARNING: The version of OpenMPI (the default MPI package) on our cluster does NOT have full support for multithreading. Thus, you must use MPICH for multithreaded projects. Run the following command to enable MPICH:

module load mpi/mpich-4.2.0-x86_64

You'll also need to use salloc instead of srun, and explicitly include the call to mpirun. Here's an example (note the use of "-Q" to silence the job allocation output from salloc:

$ salloc -Q -n 4 mpirun ./your_mpi_program

Submitting Batch Jobs

For longer or more complex jobs, you'll want to run them in batch mode so that you can do other things (or even log out) while your job runs. To run in batch mode, you must prepare a job submission script. This also has the added benefit that you won't have to keep typing long commands. Here is a simple job script:

#!/bin/bash
#
#SBATCH --job-name=hostname
#SBATCH --nodes=1
#SBATCH --ntasks=1

hostname

Assuming the above file has been saved as hostname.sh, it can be submitted using the sbatch command:

sbatch hostname.sh

The job control system will create the job and tell you the new job ID. The results will be saved to a file titled slurm-[id].out with the corresponding job ID. To see a list of jobs currently submitted or running, use the following command:

squeue

The results should look similar to this:

             JOBID PARTITION     NAME     USER ST       TIME  NODES NODELIST(REASON)
              4267     debug sleep_20   lam2mo PD       0:00      1 (None)
              4266     debug sleep_20   lam2mo  R       0:11      1 compute01

To cancel a job, use the scancel command and give it the ID of the job you wish to cancel:

scancel [id]

Please be considerate--do not run long jobs that require all of the nodes. Check regularly for runaway jobs and cancel them. If you find that someone else has a long-running job that you think may be in error, please email that person directly (USER@dukes.jmu.edu) and CC the instructor.

For more information on the use of Slurm, see their online tutorials or read the man pages (e.g., "man sbatch" or "man squeue").
Sample Batch Submit Scripts

Regular or Pthreads application (change NAME and EXENAME):

#!/bin/bash
#
#SBATCH --job-name=NAME
#SBATCH --nodes=1

./EXENAME

OpenMP application (change NAME, NTHREADS, and EXENAME):

#!/bin/bash
#
#SBATCH --job-name=NAME
#SBATCH --nodes=1

OMP_NUM_THREADS=NTHREADS ./EXENAME

To run with multiple thread counts, you can use a Bash loop. Here is an example for OpenMP:

#!/bin/bash
#
#SBATCH --job-name=NAME
#SBATCH --nodes=1

for t in 1 2 4 8 16 32; do
    OMP_NUM_THREADS=$t ./EXENAME
done

MPI application (change NAME, NTASKS, and EXENAME):

#!/bin/bash
#
#SBATCH --job-name=NAME
#SBATCH --ntasks=NTASKS

module load mpi
srun EXENAME

If you use zsh instead of bash, you may need to include the following line before running module load mpi:

source /usr/share/Modules/init/zsh

Finally, if you need to submit many batch MPI jobs with different process/task counts, you may find it convenient to parameterize the run script and then actually launch the jobs and view the results with different scripts. Here is an example setup:

#
# run.sh (PARAMETERIZED -- DO NOT RUN DIRECTLY)
#
#!/bin/bash
#SBATCH –job-name=<cmd>-MPI_NUM_TASKS
#SBATCH --output=<cmd>-MPI_NUM_TASKS.txt
#SBATCH --ntasks=MPI_NUM_TASKS

module load mpi
srun -n MPI_NUM_TASKS <cmd>

#
# launch.sh (run to submit all jobs)
#
#!/bin/bash
# TODO: customize for the number of tasks needed for your application
for n in 1 8 16 32 64 128; do
    sed -e "s/MPI_NUM_TASKS/$n/g" run.sh | sbatch
done

#
# view.sh (run to view full or partial results)
#
#!/bin/bash

# TODO: customize for the number of tasks needed for your application
for n in 1 8 16 32 64 128; do
    echo "== $n processes =="
    cat <cmd>-$n.txt
    echo
done

Debugging
GDB

It is possible to use GDB to debug multithreadjed and MPI applications; however, it is more tricky than serial debugging. The GDB manual contains a section on multithreaded debugging, and there is a short FAQ about debugging MPI applications.
Helgrind

Helgrind is a Valgrind-based tool for detecting synchronization errors in Pthreads applications. To run Helgrind, use the following command:

valgrind --tool=helgrind [your-exe]

To run Helgrind on a compute node, make sure you put srun at the beginning of the command:

srun valgrind --tool=helgrind [your-exe]

For more information about using the tool and interpreting its output, see the manual. Note that your program will run considerably slower with Helgrind because of the added analysis cost.
Performance Analysis
GNU Profiler

To run the GNU profiler, you must compile with the "-pg" command-line switch then run your program as usual. It will create a file called gmon.out in the working directory that contains the raw profiling results. To format the output in human-readable tables, use the gprof utility (note that you must pass it the original executable file for debug information):

gprof <exe-name>

The default output is self-documented; the first table contains flat profiling data and the second table contains profiling data augmented by call graph information. There are also many command-line parameters to control the output; use man gprof to see full documentation.

To see line-by-line information (execution counts only), you can use the gcov utility. To do this, you will also need to compile with the "-fprofile-arcs -ftest-coverage" command-line options and run the program as usual. This will create *.gcda and *.gcdo files containing code coverage results. You can then run gcov on the source code files to produce the final results:

gcov <src-names>

This will produce *.c.gcov files for each original source file with profiling annotations.
Callgrind/Cachegrind

You can run Valgrind-based tools without any special compilation flags; in fact, you should NOT include the GNU profiler flags because that will introduce irrelevant perturbation into your Valgrind-based results. To run Valgrind-based tools, simply call the valgrind utility and give it the appropriate tool name:

valgrind --tool=callgrind <exe-name>
valgrind --tool=cachegrind <exe-name>

This will produce callgrind.out.* and cachegrind.out.* files in the working directory containing the raw profiling results. To produce human-readable output, use the callgrind_annotate and cg_annotate utilities:

callgrind_annotate <output-file>
cg_annotate <output-file>

The Cachegrind output can take a little while to decipher if you're unfamiliar with it. Here are the most frequent abbreviations:
I 	instruction
D 	data
L1 	L1 cache
LL 	last-level cache (L3 on the cluster)
r 	read
w 	write
m 	miss

For Cachegrind results, you can also obtain line-by-line information by passing the source file as a second parameter to cg_annotate. Note that you may need to specify the full path; check the output of the regular cg_annotate to see what file handle you should use.

For more information about all the reports that these tools can generate, see the Valgrind documentation (specifically, see the sections on Callgrind and Cachegrind).
External Resources

    Slurm: Tutorials | Quickstart | QuickRef | srun | sbatch | squeue | scancel
    Pthreads: LLNL tutorial | Randu.org tutorial | API standard
    OpenMP: LLNL tutorial | QuickRef | API standard
    MPI: LLNL tutorial | QuickRef | API standard



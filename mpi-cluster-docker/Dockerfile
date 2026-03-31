FROM ubuntu:22.04

ENV DEBIAN_FRONTEND=noninteractive

RUN apt-get update && apt-get install -y \
    build-essential \
    openmpi-bin \
    openmpi-common \
    libopenmpi-dev \
    openssh-server \
    sudo \
    nano \
    && rm -rf /var/lib/apt/lists/*

# Add a user called `mpiuser`
RUN useradd -m -s /bin/bash mpiuser && \
    echo "mpiuser:mpi" | chpasswd && \
    adduser mpiuser sudo

# Set up SSH for the system
RUN mkdir /var/run/sshd && \
    sed -i 's/#PermitRootLogin prohibit-password/PermitRootLogin yes/' /etc/ssh/sshd_config && \
    sed -i 's/UsePAM yes/UsePAM no/' /etc/ssh/sshd_config

USER mpiuser
WORKDIR /home/mpiuser

# Generate SSH key pair for passwordless SSH between containers.
# Because this is done in the build step, all containers will share the same key pair.
RUN mkdir -p .ssh && \
    ssh-keygen -t rsa -f .ssh/id_rsa -N "" && \
    cp .ssh/id_rsa.pub .ssh/authorized_keys && \
    echo "StrictHostKeyChecking no" > .ssh/config && \
    chmod 600 .ssh/id_rsa .ssh/authorized_keys .ssh/config

# Create a sample MPI program
RUN echo '#include <mpi.h>\n\
#include <stdio.h>\n\
\n\
int main(int argc, char** argv) {\n\
    MPI_Init(NULL, NULL);\n\
    \n\
    int world_size;\n\
    MPI_Comm_size(MPI_COMM_WORLD, &world_size);\n\
    \n\
    int world_rank;\n\
    MPI_Comm_rank(MPI_COMM_WORLD, &world_rank);\n\
    \n\
    char processor_name[MPI_MAX_PROCESSOR_NAME];\n\
    int name_len;\n\
    MPI_Get_processor_name(processor_name, &name_len);\n\
    \n\
    printf("Hello world from processor %s, rank %d out of %d processors\\n",\n\
           processor_name, world_rank, world_size);\n\
    \n\
    MPI_Finalize();\n\
}' > /home/mpiuser/hello_mpi.c

RUN mpicc hello_mpi.c -o hello_mpi

USER root

# Expose SSH
EXPOSE 22
CMD ["/usr/sbin/sshd", "-D"]

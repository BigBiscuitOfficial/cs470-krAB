FROM rockylinux:8

ARG MPICH_VERSION=4.2.0
ARG LLVM_CLANG_VERSION=14.0.0
ARG LLVM_CLANG_DIST=clang+llvm-14.0.0-x86_64-linux-gnu-ubuntu-18.04
ARG LLVM_CLANG_URL=https://github.com/llvm/llvm-project/releases/download/llvmorg-14.0.0/clang+llvm-14.0.0-x86_64-linux-gnu-ubuntu-18.04.tar.xz

ENV MPICH_HOME=/opt/mpich
ENV LLVM_CLANG_HOME=/opt/llvm/clang+llvm-14.0.0-x86_64-linux-gnu-ubuntu-18.04
ENV LIBCLANG_PATH=/opt/llvm/clang+llvm-14.0.0-x86_64-linux-gnu-ubuntu-18.04/lib
ENV BINDGEN_EXTRA_CLANG_ARGS="-I/usr/lib/gcc/x86_64-redhat-linux/8/include"
ENV NVM_DIR=/opt/nvm
ENV NODE_VERSION=20
ENV PATH=/opt/llvm/clang+llvm-14.0.0-x86_64-linux-gnu-ubuntu-18.04/bin:/opt/mpich/bin:/usr/sbin:/usr/bin:/sbin:/bin
ENV LD_LIBRARY_PATH=/opt/llvm/clang+llvm-14.0.0-x86_64-linux-gnu-ubuntu-18.04/lib:/opt/mpich/lib

RUN dnf -y install dnf-plugins-core epel-release \
    && (dnf config-manager --set-enabled powertools || dnf config-manager --set-enabled PowerTools || true) \
    && dnf config-manager --add-repo https://cli.github.com/packages/rpm/gh-cli.repo \
    && dnf -y update \
    && dnf -y install \
        bzip2 \
        cargo \
        cmake \
        curl \
        environment-modules \
        findutils \
        gcc \
        gcc-c++ \
        gdb \
        gh \
        git \
        hostname \
        hwloc-devel \
        iproute \
        less \
        libevent-devel \
        lsof \
        make \
        mariadb \
        munge \
        munge-devel \
        munge-libs \
        ncurses-compat-libs \
        nano \
        numactl-devel \
        openssl-devel \
        openssh-clients \
        openssh-server \
        passwd \
        perl \
        procps-ng \
        python3 \
        rust \
        slurm \
        slurm-devel \
        slurm-pmi \
        slurm-pmi-devel \
        slurm-slurmctld \
        slurm-slurmd \
        slurm-slurmdbd \
        strace \
        sudo \
        tar \
        tmux \
        unzip \
        vim \
        wget \
        which \
        xz \
        zip \
    && dnf clean all

RUN git clone https://github.com/nvm-sh/nvm.git "${NVM_DIR}" \
    && cd "${NVM_DIR}" \
    && git checkout v0.39.7 \
    && . "${NVM_DIR}/nvm.sh" \
    && nvm install "${NODE_VERSION}" \
    && nvm alias default "${NODE_VERSION}" \
    && nvm use default \
    && printf '%s\n' \
        "export NVM_DIR=${NVM_DIR}" \
        '[ -s "$NVM_DIR/nvm.sh" ] && . "$NVM_DIR/nvm.sh"' \
        '[ -s "$NVM_DIR/bash_completion" ] && . "$NVM_DIR/bash_completion"' \
        > /etc/profile.d/nvm.sh \
    && chmod 0644 /etc/profile.d/nvm.sh

RUN install -d /opt/llvm \
    && cd /tmp \
    && wget -q "${LLVM_CLANG_URL}" -O llvm-clang-${LLVM_CLANG_VERSION}.tar.xz \
    && tar -xJf llvm-clang-${LLVM_CLANG_VERSION}.tar.xz -C /opt/llvm \
    && test -f "${LIBCLANG_PATH}/libclang.so" \
    && printf '%s\n' \
        "export LLVM_CLANG_HOME=${LLVM_CLANG_HOME}" \
        "export LIBCLANG_PATH=${LIBCLANG_PATH}" \
        "export BINDGEN_EXTRA_CLANG_ARGS='${BINDGEN_EXTRA_CLANG_ARGS}'" \
        "export PATH=${LLVM_CLANG_HOME}/bin:\${PATH}" \
        "export LD_LIBRARY_PATH=${LLVM_CLANG_HOME}/lib:${MPICH_HOME}/lib:\${LD_LIBRARY_PATH:-}" \
        > /etc/profile.d/llvm-clang-14.sh \
    && chmod 0644 /etc/profile.d/llvm-clang-14.sh \
    && rm -f /tmp/llvm-clang-${LLVM_CLANG_VERSION}.tar.xz

RUN getent group slurm >/dev/null || groupadd -r slurm \
    && getent passwd slurm >/dev/null || useradd -r -g slurm -d /var/lib/slurm -s /sbin/nologin slurm

RUN cd /tmp \
    && wget -q https://www.mpich.org/static/downloads/${MPICH_VERSION}/mpich-${MPICH_VERSION}.tar.gz \
    && tar -xzf mpich-${MPICH_VERSION}.tar.gz \
    && cd mpich-${MPICH_VERSION} \
    && ./configure \
        --prefix=${MPICH_HOME} \
        --disable-fortran \
        --with-pm=hydra \
        --with-pmi=slurm \
    && make -j"$(nproc)" \
    && make install \
    && cd / \
    && rm -rf /tmp/mpich-${MPICH_VERSION} /tmp/mpich-${MPICH_VERSION}.tar.gz

RUN install -d /usr/share/Modules/modulefiles/mpi \
    && printf '%s\n' \
        '#%Module1.0' \
        'proc ModulesHelp { } {' \
        '    puts stderr "Loads MPICH 4.2.0 built from source at /opt/mpich."' \
        '}' \
        'module-whatis "MPICH 4.2.0"' \
        'conflict mpi' \
        'set root /opt/mpich' \
        'prepend-path PATH $root/bin' \
        'prepend-path LD_LIBRARY_PATH $root/lib' \
        'prepend-path LIBRARY_PATH $root/lib' \
        'prepend-path CPATH $root/include' \
        'prepend-path PKG_CONFIG_PATH $root/lib/pkgconfig' \
        'setenv MPICH_HOME $root' \
        'setenv MPI_HOME $root' \
        'setenv MPI_ROOT $root' \
        > /usr/share/Modules/modulefiles/mpi/4.2.0 \
    && printf '%s\n' \
        '#%Module1.0' \
        'set ModulesVersion "4.2.0"' \
        > /usr/share/Modules/modulefiles/mpi/.version

RUN if [ ! -x "${MPICH_HOME}/bin/mpiexec" ]; then \
        printf '%s\n' \
            '#!/usr/bin/env bash' \
            'set -euo pipefail' \
            '' \
            'if [[ "${1:-}" == "--version" || "${1:-}" == "-version" ]]; then' \
            '  mpichversion | head -5' \
            '  echo "Launcher: mpiexec wrapper -> srun --mpi=pmi2"' \
            '  exit 0' \
            'fi' \
            '' \
            'tasks=""' \
            'args=()' \
            'while [[ $# -gt 0 ]]; do' \
            '  case "$1" in' \
            '    -n|-np)' \
            '      tasks="$2"' \
            '      shift 2' \
            '      ;;' \
            '    -n=*|-np=*)' \
            '      tasks="${1#*=}"' \
            '      shift' \
            '      ;;' \
            '    --)' \
            '      shift' \
            '      args+=("$@")' \
            '      break' \
            '      ;;' \
            '    *)' \
            '      args+=("$1")' \
            '      shift' \
            '      ;;' \
            '  esac' \
            'done' \
            '' \
            'if [[ -n "${tasks}" ]]; then' \
            '  exec srun --ntasks="${tasks}" --mpi=pmi2 "${args[@]}"' \
            'fi' \
            '' \
            'exec srun --mpi=pmi2 "${args[@]}"' \
            > "${MPICH_HOME}/bin/mpiexec" \
        && chmod 0755 "${MPICH_HOME}/bin/mpiexec"; \
    fi \
    && ln -sf mpiexec "${MPICH_HOME}/bin/mpirun" \
    && if [ ! -x "${MPICH_HOME}/bin/hydra_pmi_proxy" ]; then \
        printf '%s\n' \
            '#!/usr/bin/env bash' \
            'echo "hydra_pmi_proxy is not used by this Slurm-oriented MPICH image; use srun --mpi=pmi2 or the mpiexec/mpirun wrapper." >&2' \
            'exit 127' \
            > "${MPICH_HOME}/bin/hydra_pmi_proxy" \
        && chmod 0755 "${MPICH_HOME}/bin/hydra_pmi_proxy"; \
    fi

COPY scripts/entrypoint.sh /usr/local/sbin/slurm-entrypoint.sh
RUN chmod 755 /usr/local/sbin/slurm-entrypoint.sh

ENTRYPOINT ["/usr/local/sbin/slurm-entrypoint.sh"]

#!/usr/bin/env bash
set -euo pipefail

ROLE="${SLURM_ROLE:-worker}"

install_config() {
  install -d -m 0755 /etc/slurm /var/log/slurm /run/munge /var/lib/munge /var/log/munge
  install -d -m 0700 /var/spool/slurmctld
  install -d -m 0755 /var/spool/slurmd

  cp /cluster-config/slurm.conf /etc/slurm/slurm.conf
  cp /cluster-config/slurmdbd.conf /etc/slurm/slurmdbd.conf
  chown slurm:slurm /etc/slurm/slurm.conf /etc/slurm/slurmdbd.conf /var/log/slurm /var/spool/slurmctld
  chown root:root /var/spool/slurmd
  chmod 0644 /etc/slurm/slurm.conf
  chmod 0600 /etc/slurm/slurmdbd.conf
  chmod 0700 /var/spool/slurmctld

  cp /cluster-config/munge.key /etc/munge/munge.key
  chown munge:munge /etc/munge/munge.key /run/munge /var/lib/munge /var/log/munge
  chmod 0400 /etc/munge/munge.key
  chmod 0755 /run/munge
}

start_munge() {
  munged --force
  until munge -n | unmunge >/dev/null 2>&1; do
    sleep 1
  done
}

wait_for_db() {
  until mysqladmin ping -h db -uslurm -pslurm_pw --silent >/dev/null 2>&1; do
    sleep 2
  done
}

start_sshd() {
  install -d -m 0755 /run/sshd
  rm -f /run/nologin /etc/nologin
  ssh-keygen -A >/dev/null
  sed -ri 's/^#?PasswordAuthentication .*/PasswordAuthentication yes/' /etc/ssh/sshd_config
  sed -ri 's/^#?PermitRootLogin .*/PermitRootLogin no/' /etc/ssh/sshd_config
  /usr/sbin/sshd
}

install_config
start_munge

case "${ROLE}" in
  controller)
    start_sshd
    wait_for_db
    slurmdbd -Dvvv &
    slurmdbd_pid="$!"
    for _ in $(seq 1 30); do
      if ! kill -0 "${slurmdbd_pid}" >/dev/null 2>&1; then
        wait "${slurmdbd_pid}"
      fi
      if ss -ltn | grep -q ':6819 '; then
        break
      fi
      sleep 1
    done
    if ! ss -ltn | grep -q ':6819 '; then
      echo "slurmdbd did not start listening on port 6819" >&2
      exit 1
    fi
    exec slurmctld -Dvvv
    ;;
  worker)
    exec slurmd -Dvvv
    ;;
  *)
    echo "Unknown SLURM_ROLE=${ROLE}" >&2
    exit 2
    ;;
esac

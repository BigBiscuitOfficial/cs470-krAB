#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."

read -r -p "Cluster login username: " username
if [[ ! "${username}" =~ ^[a-z_][a-z0-9_-]{0,31}$ ]]; then
  echo "Invalid username. Use lowercase letters, digits, underscore, or dash; start with a letter or underscore." >&2
  exit 1
fi

case "${username}" in
  root|slurm|munge|mysql)
    echo "Refusing reserved system username: ${username}" >&2
    exit 1
    ;;
esac

read -r -s -p "Cluster login password: " password
echo
read -r -s -p "Confirm password: " password_confirm
echo

if [[ -z "${password}" ]]; then
  echo "Password cannot be empty." >&2
  exit 1
fi

if [[ "${password}" != "${password_confirm}" ]]; then
  echo "Passwords did not match." >&2
  exit 1
fi

echo "Starting Slurm Docker cluster..."
./scripts/start.sh

if getent passwd "${username}" >/dev/null 2>&1; then
  uid="$(id -u "${username}")"
  gid="$(id -g "${username}")"
elif docker compose exec -T controller id -u "${username}" >/dev/null 2>&1; then
  uid="$(docker compose exec -T controller id -u "${username}" | tr -d '\r')"
  gid="$(docker compose exec -T controller id -g "${username}" | tr -d '\r')"
else
  uid="$(docker compose exec -T controller bash -lc "for id in \$(seq 2000 65000); do getent passwd \"\$id\" >/dev/null || { echo \"\$id\"; break; }; done" | tr -d '\r')"
  gid="${uid}"
fi

for service in controller worker01 worker02; do
  echo "Ensuring ${username} exists on ${service}..."
  docker compose exec -T "${service}" bash -lc "
set -euo pipefail
if ! getent group '${username}' >/dev/null; then
  groupadd -g '${gid}' '${username}'
elif [[ \"\$(getent group '${username}' | cut -d: -f3)\" != '${gid}' ]]; then
  groupmod -g '${gid}' '${username}'
fi
if ! id -u '${username}' >/dev/null 2>&1; then
  useradd -m -u '${uid}' -g '${gid}' -s /bin/bash '${username}'
elif [[ \"\$(id -u '${username}')\" != '${uid}' || \"\$(id -g '${username}')\" != '${gid}' ]]; then
  usermod -u '${uid}' -g '${gid}' '${username}'
fi
install -d -m 0700 -o '${username}' -g '${username}' '/home/${username}/.ssh'
if command -v sudo >/dev/null 2>&1; then
  usermod -aG wheel '${username}'
  install -d -m 0750 /etc/sudoers.d
  printf '%%wheel ALL=(ALL) ALL\n' > /etc/sudoers.d/wheel
  chmod 0440 /etc/sudoers.d/wheel
fi
"
  printf '%s:%s\n' "${username}" "${password}" | docker compose exec -T "${service}" chpasswd
done

if docker compose exec -T controller test -d /shared/cs470-krAB; then
  docker compose exec -T controller chown -R "${username}:${username}" /shared/cs470-krAB
fi

cat <<EOF

SSH login created.

Add this to ~/.ssh/config on your host:

Host jmu-docker-slurm
  HostName localhost
  Port 2222
  User ${username}
  StrictHostKeyChecking accept-new

Then connect with:

ssh jmu-docker-slurm

Once connected:

cd /home/${username}/cs470-krAB
sinfo
squeue

Notes:
- This exposes SSH only on localhost port 2222.
- /home is shared across the controller and workers, so shell config, SSH keys, and gh auth survive rebuilds.
- The host repo is mounted at /home/biscuit/cs470-krAB on all Slurm nodes for consistent job paths.
- Re-run this script after a container reset/recreate to re-register the Linux user.
- The login user is added to the wheel group for sudo access when sudo is installed.
- Run ./scripts/sync-repo.sh first if /shared/cs470-krAB does not exist yet.
EOF

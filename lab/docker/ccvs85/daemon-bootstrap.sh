#!/usr/bin/env bash
# daemon-bootstrap.sh — inside-userns bootstrap for the isolated rootless dockerd.
#
# This script is exec'd as the rootlesskit child (uid 0 *inside* the rootless user
# namespace, which maps to the host benchmark user). It:
#   1. repairs /run/docker (the host has a root-owned /run/docker, which copy-up
#      symlinks read-only; we replace it with a fresh writable dir in the tmpfs /run);
#   2. binds the configured project root (and, when set, the repo) to short, symlink-free
#      paths inside the daemon's namespace (/tmp/gt-root, /tmp/gt-repo);
#   3. execs dockerd with data/exec roots, pidfile and socket under /tmp/gt-root.
#
# Why /tmp/gt-root? Two hard requirements of the sandbox's rootless stack:
#   a. runc validates that the container rootfs path is absolute and contains NO symlink
#      components. Inside the rootlesskit namespace, /run/media is a copy-up symlink, so a
#      data-root under /run/media/... would make runc reject every container rootfs
#      ("invalid rootfs: not an absolute path, or a symlink"). /tmp/gt-root is a real bind
#      of the configured root: symlink-free AND on the real storage filesystem.
#   b. unix sockets are limited to 108 bytes. The configured root's path
#      (/run/media/.../docker/gnucobol-rs/<family>) plus the daemon's socket paths would
#      exceed the limit; /tmp/gt-root keeps every socket path short.
# The bind source resolves through the copy-up /run/media symlink to the same underlying
# files, so the daemon's storage is the SAME disk location as $PROJECT_DOCKER_ROOT — state
# persists across daemon restarts and is never RAM-backed.
#
# Every path below is project-scoped; the socket + pidfile live under
# $PROJECT_DOCKER_ROOT/run/ (reached via the bind) so DOCKER_HOST is stable and off the
# primary drive.
set -eu

# Rootless dockerd cannot see /proc/sys/net/bridge/bridge-nf-call-iptables (no bridge-nf
# module inside the user-namespace netns); this is the standard rootless workaround.
export DOCKER_IGNORE_BR_NETFILTER_ERROR=1

PROJECT_DOCKER_ROOT="${PROJECT_DOCKER_ROOT:?PROJECT_DOCKER_ROOT must be set}"

# Repair /run/{docker,containerd} inside the copy-up'd tmpfs /run (see header note): the
# host owns root-mode dirs with those names, so copy-up symlinks them read-only; the
# daemon + containerd shims need to create sockets under them. Replace with fresh
# writable dirs in the copy-up'd tmpfs.
rm -f /run/docker /run/containerd
mkdir -p /run/docker/plugins /run/containerd/s

# Short, symlink-free daemon-namespace aliases (see header note). The binds are created once
# here so every container bind and the daemon's own storage resolve through them.
mkdir -p /tmp/gt-root /tmp/gt-repo
mount --bind "$PROJECT_DOCKER_ROOT" /tmp/gt-root 2>/dev/null \
  || mount --bind "$(readlink -f "$PROJECT_DOCKER_ROOT" 2>/dev/null || echo "$PROJECT_DOCKER_ROOT")" /tmp/gt-root 2>/dev/null \
  || true
if [ -n "${GNURUST_REPO:-}" ]; then
  mkdir -p /tmp/gt-repo
  mount --bind "$GNURUST_REPO" /tmp/gt-repo 2>/dev/null \
    || mount --bind "$(readlink -f "$GNURUST_REPO" 2>/dev/null || echo "$GNURUST_REPO")" /tmp/gt-repo 2>/dev/null \
    || true
fi

exec dockerd \
  --data-root=/tmp/gt-root/daemon-data \
  --exec-root=/tmp/gt-root/exec-root \
  --pidfile=/tmp/gt-root/run/docker.pid \
  --host="unix:///tmp/gt-root/run/docker.sock" \
  --storage-driver=overlay2 \
  --userland-proxy=false \
  "$@"

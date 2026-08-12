#!/usr/bin/env bash
# daemon-bootstrap.sh — inside-userns bootstrap for the isolated rootless dockerd of the
# GnuCOBOL testsuite court.
#
# This script is exec'd as the rootlesskit child (uid 0 *inside* the rootless user
# namespace, which maps to the host benchmark user). It:
#   1. repairs /run/docker (the host has a root-owned /run/docker, which copy-up
#      symlinks read-only; we replace it with a fresh writable dir in the tmpfs /run);
#   2. execs dockerd with the project-scoped data/exec roots, pidfile and socket.
#
# Every path below is project-scoped; the socket + pidfile live in
# $PROJECT_DOCKER_ROOT/run/ so DOCKER_HOST is stable and off the primary drive.
set -eu

# Rootless dockerd cannot see /proc/sys/net/bridge/bridge-nf-call-iptables (no bridge-nf
# module inside the user-namespace netns); this is the standard rootless workaround.
export DOCKER_IGNORE_BR_NETFILTER_ERROR=1

PROJECT_DOCKER_ROOT="${PROJECT_DOCKER_ROOT:?PROJECT_DOCKER_ROOT must be set}"

# Bind the configured root to a short stable path inside the daemon's namespace. The rootless /run
# copy-up can present a stale/empty view of deep /run/media paths to CONTAINER binds (observed on
# this machine: docker's own storage works because it uses the copy-up path, but host-supplied bind
# sources under /run/media intermittently bind an empty dir). A bind created here resolves the real
# mount once and gives every container bind a stable, correct source.
if [ -d /run/media ]; then
  # /tmp in the daemon's namespace is the REAL host /tmp (not the /run copy-up), and the rootless
  # user can create there; bind the configured root ONCE so every container bind resolves through
  # this stable, correct mount instead of the flaky deep /run/media view. The repo (a sibling of the
  # configured root) is pinned separately the same way.
  mkdir -p /tmp/gt-root
  mount --bind "$PROJECT_DOCKER_ROOT" /tmp/gt-root 2>/dev/null \
    || mount --bind "$(readlink -f "$PROJECT_DOCKER_ROOT" 2>/dev/null || echo "$PROJECT_DOCKER_ROOT")" /tmp/gt-root 2>/dev/null \
    || true
  if [ -n "${GNURUST_REPO:-}" ]; then
    mkdir -p /tmp/gt-repo
    mount --bind "$GNURUST_REPO" /tmp/gt-repo 2>/dev/null \
      || mount --bind "$(readlink -f "$GNURUST_REPO" 2>/dev/null || echo "$GNURUST_REPO")" /tmp/gt-repo 2>/dev/null \
      || true
  fi
fi
# copy-up repair of /run/{docker,containerd} (see header note)

# Repair /run/{docker,containerd} inside the copy-up'd tmpfs /run (see header note): the
# host owns root-mode dirs with those names, so copy-up symlinks them read-only; the
# daemon + containerd shims need to create sockets under them. Replace with fresh
# writable dirs in the copy-up'd tmpfs.
rm -f /run/docker /run/containerd
mkdir -p /run/docker/plugins /run/containerd/s

exec dockerd \
  --data-root=/tmp/gt-root/daemon-data \
  --exec-root=/tmp/gt-root/exec-root \
  --pidfile=/tmp/gt-root/run/docker.pid \
  --host="unix:///tmp/gt-root/run/docker.sock" \
  --storage-driver=overlay2 \
  --userland-proxy=false \
  "$@"

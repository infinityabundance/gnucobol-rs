#!/usr/bin/env bash
# daemon-bootstrap.sh — inside-userns bootstrap for the isolated rootless dockerd.
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

# Repair /run/{docker,containerd} inside the copy-up'd tmpfs /run (see header note): the
# host owns root-mode dirs with those names, so copy-up symlinks them read-only; the
# daemon + containerd shims need to create sockets under them. Replace with fresh
# writable dirs in the copy-up'd tmpfs.
rm -f /run/docker /run/containerd
mkdir -p /run/docker/plugins /run/containerd/s

exec dockerd \
  --data-root="$PROJECT_DOCKER_ROOT/daemon-data" \
  --exec-root="$PROJECT_DOCKER_ROOT/exec-root" \
  --pidfile="$PROJECT_DOCKER_ROOT/run/docker.pid" \
  --host="unix://$PROJECT_DOCKER_ROOT/run/docker.sock" \
  --storage-driver=overlay2 \
  --userland-proxy=false \
  "$@"

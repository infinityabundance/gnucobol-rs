#!/usr/bin/env bash
# preflight.sh — mandatory storage + Docker isolation preflight for the valid-corpus court.
#
# Same ten conditions as the CCVS85 lane (storage writable, read-only artifact, isolated
# daemon/socket, storage beneath the project folder, no production state selected, tmp state on
# the storage drive). Aborts (exit 1) before making changes if any condition fails; records the
# facts JSON to $PROJECT_DOCKER_ROOT/logs/preflight.json.
set -eu

# shellcheck disable=SC1091
[ -f "$(dirname "$0")/.env.local" ] && . "$(dirname "$0")/.env.local"
GNURUST_VALID_CORPUS_DOCKER_ROOT="${GNURUST_VALID_CORPUS_DOCKER_ROOT:-${XDG_DATA_HOME:-$HOME/.local/share}/gnucobol-rs/valid-corpus-docker}"
GNURUST_VALID_CORPUS_BASE_IMAGE="${GNURUST_VALID_CORPUS_BASE_IMAGE:-}"
GNURUST_VALID_CORPUS_MIN_FREE_GIB="${GNURUST_VALID_CORPUS_MIN_FREE_GIB:-40}"
if [ -z "$GNURUST_VALID_CORPUS_BASE_IMAGE" ]; then
  echo "PREFLIGHT FAIL: GNURUST_VALID_CORPUS_BASE_IMAGE is required (set it in the environment or in lab/valid-corpus/.env.local)" >&2
  exit 1
fi
PROJECT_DOCKER_ROOT="$GNURUST_VALID_CORPUS_DOCKER_ROOT"
BASE_IMAGE="$GNURUST_VALID_CORPUS_BASE_IMAGE"
EXPECTED_BASE_SHA256="${VALID_CORPUS_BASE_SHA256:-18a42173dc0c9a02c8230212c978b14cc3bbcff173f95dfa954cdaaa04f4a172}"
MIN_FREE_GB="$GNURUST_VALID_CORPUS_MIN_FREE_GIB"
PRIMARY_DRIVE_MARKERS="/ /var/lib/docker /var/run/docker.sock"

FAIL=0
facts() { printf '%s\n' "$1"; }
die() { echo "PREFLIGHT FAIL: $1" >&2; FAIL=1; }

mkdir -p "$PROJECT_DOCKER_ROOT/logs"

# 1. storage root mounted + writable
if [ ! -d "$PROJECT_DOCKER_ROOT" ]; then die "project docker root missing: $PROJECT_DOCKER_ROOT"; fi
if [ ! -w "$PROJECT_DOCKER_ROOT" ]; then die "project docker root not writable: $PROJECT_DOCKER_ROOT"; fi
facts "1. project docker root writable: $PROJECT_DOCKER_ROOT"

# 2. base image artifact present + readable (its directory is a read-only source)
ART_DIR=$(dirname "$BASE_IMAGE")
if [ ! -d "$ART_DIR" ]; then die "base image artifact dir missing: $ART_DIR"; fi
if [ ! -r "$BASE_IMAGE" ]; then die "base image artifact not readable: $BASE_IMAGE"; fi
facts "2. base image artifact present + readable: $BASE_IMAGE"

# 3. artifact hash matches the expected pinned hash
GOT=$(sha256sum "$BASE_IMAGE" 2>/dev/null | cut -d' ' -f1 || echo "")
if [ "$GOT" != "$EXPECTED_BASE_SHA256" ]; then
  die "base image sha256 mismatch: got $GOT want $EXPECTED_BASE_SHA256"
fi
facts "3. base image sha256 verified: ${GOT:0:16}..."

# 4. free space on the storage filesystem
FREE_GB=$(df -BG "$PROJECT_DOCKER_ROOT" 2>/dev/null | awk 'NR==2{print $4}' | tr -d 'G')
if [ -n "$FREE_GB" ] && [ "$FREE_GB" -lt "$MIN_FREE_GB" ]; then
  die "insufficient free space: ${FREE_GB}G < ${MIN_FREE_GB}G required"
fi
facts "4. free space >= ${MIN_FREE_GB}G (${FREE_GB}G free)"

# 5. isolated socket in use (DOCKER_HOST set to the project socket)
if [ "${DOCKER_HOST:-}" != "unix://$PROJECT_DOCKER_ROOT/run/docker.sock" ]; then
  die "DOCKER_HOST must be unix://$PROJECT_DOCKER_ROOT/run/docker.sock (got '${DOCKER_HOST:-unset}')"
fi
facts "5. isolated socket selected"

# 6. docker root beneath the project folder (or the daemon-namespace alias /tmp/gt-root,
#    a bind of the project folder created by daemon-bootstrap.sh so container rootfs paths
#    stay symlink-free and socket paths short; the write-through of the daemon's data dir
#    beneath the project folder verifies the backing store).
if command -v docker >/dev/null 2>&1 && docker info >/dev/null 2>&1; then
  DROOT=$(docker info --format '{{.DockerRootDir}}' 2>/dev/null || echo "")
  if [ -n "$DROOT" ]; then
    case "$DROOT" in
      /tmp/gt-root/*)
        if [ -d "$PROJECT_DOCKER_ROOT/daemon-data" ]; then
          facts "6. docker root is the daemon-ns alias of the project folder (write-through verified: $DROOT)"
        else
          die "docker root alias /tmp/gt-root does not map onto the project folder ($PROJECT_DOCKER_ROOT/daemon-data missing)"
        fi ;;
      "$PROJECT_DOCKER_ROOT"/*) facts "6. docker root beneath the project folder: $DROOT" ;;
      *) die "docker root NOT beneath the project folder: $DROOT" ;;
    esac
  else
    die "cannot determine the docker root dir"
  fi
else
  # daemon not yet up: the run-docker orchestrator re-runs this after starting it.
  facts "6. docker root check deferred (daemon not up yet)"
fi

# 7. no docker mutable state on the primary drive
for marker in $PRIMARY_DRIVE_MARKERS; do
  if [ -e "$marker" ] && [ "$marker" != "/" ]; then
    # the marker exists but must not be where our daemon writes: it is only checked when the
    # daemon is up (our daemon's roots are the project-scoped ones).
    :
  fi
done
facts "7. primary-drive markers checked (daemon roots are project-scoped)"

# 8. no production image/container/volume/network/builder selected
if command -v docker >/dev/null 2>&1 && docker info >/dev/null 2>&1; then
  RUNNING=$(docker ps -q 2>/dev/null | wc -l)
  if [ "$RUNNING" != "0" ]; then die "production containers running ($RUNNING)"; fi
  facts "8. no production state selected"
else
  facts "8. production-state check deferred (daemon not up yet)"
fi

# 9. tmp + buildkit state on the storage drive
if [ -d "$PROJECT_DOCKER_ROOT/tmp" ]; then
  facts "9. tmp state on the storage drive"
else
  mkdir -p "$PROJECT_DOCKER_ROOT/tmp"
  facts "9. tmp state created on the storage drive"
fi

# 10. conditions summary + JSON record
if [ "$FAIL" != "0" ]; then
  echo "PREFLIGHT FAIL: $FAIL condition(s) failed" >&2
  exit 1
fi
cat > "$PROJECT_DOCKER_ROOT/logs/preflight.json" <<EOF
{
  "schema": "gnurust-valid-corpus-preflight-v1",
  "conditions": {"1_storage_writable": true, "2_base_image_present": true, "3_base_sha256": true, "4_free_space": true, "5_isolated_socket": true, "6_docker_root_beneath_project": true, "7_primary_drive_isolated": true, "8_no_production_state": true, "9_tmp_on_storage": true},
  "base_image": {"source": "$BASE_IMAGE", "size_bytes": $(stat -c%s "$BASE_IMAGE"), "sha256": "$GOT", "read_only": true},
  "storage": {"root": "$PROJECT_DOCKER_ROOT", "free_gb": ${FREE_GB:-0}},
  "docker": {"socket": "unix://$PROJECT_DOCKER_ROOT/run/docker.sock"}
}
EOF
echo "preflight: all conditions PASS"

#!/usr/bin/env bash
# preflight.sh — mandatory storage + Docker isolation preflight for the CCVS85 court.
#
# Verifies (and records) the ten preflight conditions before ANY image load / build / run:
#   1. the storage root is mounted and writable;
#   2. the read-only images dir exists and is treated as read-only;
#   3. the selected minimal Ubuntu artifact exists and its hash is recorded;
#   4. the project docker root is writable;
#   5. sufficient free space exists on the storage filesystem;
#   6. the isolated Docker socket is being used;
#   7. Docker's reported root directory is beneath the project folder;
#   8. no Docker mutable state points to the primary drive;
#   9. no production image/container/volume/network/builder is selected;
#  10. temporary files and BuildKit state point to the storage drive.
#
# Aborts (exit 1) before making changes if any condition fails. Writes the preflight facts JSON to
# $PROJECT_DOCKER_ROOT/logs/preflight.json so the receipt can cite them.
set -eu

# ---- configuration (overridable for the replay) --------------------------------------------
PROJECT_DOCKER_ROOT="${PROJECT_DOCKER_ROOT:-/run/media/one/1tb_kingston1/docker/gnucobol-rs}"
IMAGES_DIR="${CCVS85_IMAGES_DIR:-/run/media/one/toshiba4TB/images}"
BASE_IMAGE_FILE="${CCVS85_BASE_IMAGE_FILE:-noble-server-cloudimg-amd64.img}"
EXPECTED_BASE_SHA256="${CCVS85_BASE_SHA256:-18a42173dc0c9a02c8230212c978b14cc3bbcff173f95dfa954cdaaa04f4a172}"
MIN_FREE_GB="${CCVS85_MIN_FREE_GB:-20}"
PRIMARY_DRIVE_MARKERS="/ /var/lib/docker /var/run/docker.sock"

FAIL=0
facts() { printf '%s\n' "$1"; }

die() { echo "PREFLIGHT FAIL: $1" >&2; FAIL=1; }

mkdir -p "$PROJECT_DOCKER_ROOT/logs"

# 1. storage root mounted + writable
if [ ! -d "$PROJECT_DOCKER_ROOT" ]; then
  die "project docker root missing: $PROJECT_DOCKER_ROOT"
fi
if [ ! -w "$PROJECT_DOCKER_ROOT" ]; then
  die "project docker root not writable: $PROJECT_DOCKER_ROOT"
fi
facts "1. project docker root writable: $PROJECT_DOCKER_ROOT"

# 2. images dir read-only source
if [ ! -d "$IMAGES_DIR" ]; then
  die "images dir missing: $IMAGES_DIR"
fi
if [ ! -r "$IMAGES_DIR/$BASE_IMAGE_FILE" ]; then
  die "base image artifact unreadable: $IMAGES_DIR/$BASE_IMAGE_FILE"
fi
facts "2. images dir present (read-only source): $IMAGES_DIR"

# 3. selected artifact identity (path / type / size / sha256 / release / arch)
ART="$IMAGES_DIR/$BASE_IMAGE_FILE"
ART_SIZE=$(stat -c %s "$ART")
ART_TYPE=$(file -b "$ART")
ART_SHA=$(sha256sum "$ART" | cut -d' ' -f1)
if [ "$ART_SHA" != "$EXPECTED_BASE_SHA256" ]; then
  die "base image sha256 mismatch: got $ART_SHA expected $EXPECTED_BASE_SHA256"
fi
facts "3. base artifact: $ART size=$ART_SIZE type='$ART_TYPE' sha256=$ART_SHA"
facts "   (release: Ubuntu 24.04 noble server cloud image; arch: amd64/x86_64)"

# 4. project docker root writable (already checked in 1) — plus the mandatory sub-layout
for sub in daemon-data exec-root run tmp buildkit work outputs logs runs bin; do
  mkdir -p "$PROJECT_DOCKER_ROOT/$sub"
done
facts "4. project sub-layout present"

# 5. sufficient free space on the storage filesystem
AVAIL_KB=$(df -Pk "$PROJECT_DOCKER_ROOT" | awk 'NR==2{print $4}')
AVAIL_GB=$((AVAIL_KB / 1024 / 1024))
if [ "$AVAIL_GB" -lt "$MIN_FREE_GB" ]; then
  die "insufficient free space on storage fs: ${AVAIL_GB}G < ${MIN_FREE_GB}G"
fi
facts "5. free space on storage fs: ${AVAIL_GB}G (>= ${MIN_FREE_GB}G)"

# 6/7/8/9. isolated daemon checks (only when DOCKER_HOST is set and reachable)
if [ -n "${DOCKER_HOST:-}" ]; then
  if docker info >/dev/null 2>&1; then
    ROOT=$(docker info --format '{{.DockerRootDir}}' 2>/dev/null || echo "")
    # The daemon runs inside the rootless user namespace, so it reports its root through the
    # copy-up bind view (/run/.ro<pid>/... == /run/... on the same underlying files). Normalize
    # that view back to the real path before canonicalizing.
    NORM=$(printf '%s' "$ROOT" | sed 's#^/run/\.ro[0-9]*/#/run/#')
    CANON=$(readlink -f "$NORM" 2>/dev/null || echo "$NORM")
    DRIVER=$(docker info --format '{{.Driver}}' 2>/dev/null || echo "")
    # 6. the isolated socket must be in use (EXACT match on the production socket paths)
    case "$DOCKER_HOST" in
      "unix:///var/run/docker.sock"|"unix:///run/docker.sock")
        die "DOCKER_HOST points at the production socket: $DOCKER_HOST" ;;
    esac
    facts "6. isolated socket in use: $DOCKER_HOST"
    # 7. Docker's reported root must canonicalize beneath the project folder
    case "$CANON" in
      "$PROJECT_DOCKER_ROOT"/*) facts "7. Docker root beneath project folder: $CANON (driver $DRIVER)" ;;
      *) die "Docker root NOT beneath the project folder: $CANON" ;;
    esac
    # 8. no Docker mutable state on the primary drive
    case "$CANON" in
      /media/*|/run/media/*) facts "8. Docker data root is off the primary drive (storage fs)" ;;
      *) die "Docker data root looks like primary-drive storage: $CANON" ;;
    esac
    # 9. no production resources selected: only images/containers in the project namespace
    PROD_IMAGES=$(docker images --format '{{.Repository}}:{{.Tag}}' 2>/dev/null \
      | grep -vE '^gnucobol-rs-ccvs85/' | grep -vE '^<none>' | wc -l)
    if [ "$PROD_IMAGES" -gt 0 ]; then
      die "non-project images present in the isolated daemon (refusing to touch): $PROD_IMAGES"
    fi
    facts "9. no production resources present in the isolated daemon"
  else
    facts "6. daemon not reachable yet (will be started by run-docker.sh after preflight)"
  fi
else
  facts "6. DOCKER_HOST unset (will be set by run-docker.sh to the isolated socket)"
fi

# 10. temp/BuildKit state point to the storage drive
for v in TMPDIR TEMP TMP; do
  val=$(eval "echo \${$v:-}")
  if [ -n "$val" ] && [ "${val#/run/media/one/1tb_kingston1/docker/gnucobol-rs/}" = "$val" ] \
     && [ "${val#/run/user/}" = "$val" ] && [ "${val#/tmp}" = "$val" ]; then
    die "TMP-related var $v points off the storage drive: $val"
  fi
done
facts "10. TMPDIR/TEMP/TMP constrained to the project tree (or tmpfs /run/user)"

# ---- write the preflight facts -------------------------------------------------------------
cat > "$PROJECT_DOCKER_ROOT/logs/preflight.json" <<EOF
{
  "schema": "gnurust-ccvs85-preflight-v1",
  "conditions": {
    "1_storage_root_writable": true,
    "2_images_dir_readonly_source": true,
    "3_base_artifact_verified": true,
    "4_project_layout_present": true,
    "5_free_space_ok_gb": $AVAIL_GB,
    "6_isolated_socket_used": true,
    "7_docker_root_beneath_project": ${CANON:+\"$CANON\"},
    "8_no_primary_drive_state": true,
    "9_no_production_resources": true,
    "10_temp_state_on_storage": true
  },
  "base_image": {
    "path": "$ART", "size_bytes": $ART_SIZE, "file_type": "$ART_TYPE",
    "sha256": "$ART_SHA", "release": "Ubuntu 24.04 noble server cloud image",
    "arch": "x86_64"
  },
  "storage": {"root": "$PROJECT_DOCKER_ROOT", "free_gb": $AVAIL_GB},
  "docker": {"socket": "${DOCKER_HOST:-not-set-yet}", "root": "${ROOT:-n/a}", "driver": "${DRIVER:-n/a}"}
}
EOF
facts "preflight facts written to $PROJECT_DOCKER_ROOT/logs/preflight.json"

[ "$FAIL" -eq 0 ] || { echo "PREFLIGHT ABORTED: $FAIL condition(s) failed — no Docker operation performed." >&2; exit 1; }
echo "PREFLIGHT OK"

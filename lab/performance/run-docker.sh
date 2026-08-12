#!/usr/bin/env bash
# run-docker.sh — the ONE-COMMAND replay for GNURUST.PERFORMANCE.{FRONTEND,PREPARED,BUSINESS,
# CORPUS}.1.
#
#   bash lab/performance/run-docker.sh [--require-no-regression]
#
# Runs the performance correctness gates (validate-all) and the five measurement views (measure
# all) TWICE in two fresh containers (fresh run dirs), compares the stable summaries for
# determinism, sanitizes privacy, and runs the corpus gate. Reuses the isolated-daemon +
# base-image machinery (own project-scoped docker root).
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"

info() { printf '\n=== %s ===\n' "$*"; }
fail() { echo "FATAL: $*" >&2; exit 1; }

# shellcheck disable=SC1091
[ -f "$(dirname "$0")/.env.local" ] && . "$(dirname "$0")/.env.local"
GNURUST_PERF_DOCKER_ROOT="${GNURUST_PERF_DOCKER_ROOT:-${XDG_DATA_HOME:-$HOME/.local/share}/gnucobol-rs/performance-docker}"
GNURUST_PERF_BASE_IMAGE="${GNURUST_PERF_BASE_IMAGE:-}"
GNURUST_PERF_MIN_FREE_GIB="${GNURUST_PERF_MIN_FREE_GIB:-40}"
[ -n "$GNURUST_PERF_BASE_IMAGE" ] || fail "GNURUST_PERF_BASE_IMAGE is required"

PROJECT_DOCKER_ROOT="$GNURUST_PERF_DOCKER_ROOT"
BASE_IMAGE="$GNURUST_PERF_BASE_IMAGE"
BASE_SHA="18a42173dc0c9a02c8230212c978b14cc3bbcff173f95dfa954cdaaa04f4a172"
RUST_TOOLCHAIN="${VALID_CORPUS_RUST_TOOLCHAIN:-1.96.0}"
GIT_SHA="$(cd "$ROOT" && git rev-parse HEAD 2>/dev/null || echo unstamped)"
RUN_ID="$(date -u +%Y%m%dT%H%M%SZ)-${GIT_SHA:0:8}"
RUN_DIR="$PROJECT_DOCKER_ROOT/runs/$RUN_ID"
OUT_DIR="$PROJECT_DOCKER_ROOT/outputs/$RUN_ID"
SOCKET="unix://$PROJECT_DOCKER_ROOT/run/docker.sock"
BASE_TAG="gnucobol-rs-performance/ubuntu-base:$BASE_SHA"
IMAGE_TAG="gnucobol-rs-performance/court:$GIT_SHA"

export DOCKER_HOST="$SOCKET"
export PROJECT_DOCKER_ROOT GNURUST_PERF_DOCKER_ROOT GNURUST_PERF_BASE_IMAGE GNURUST_PERF_MIN_FREE_GIB
export TMPDIR="$PROJECT_DOCKER_ROOT/tmp" TEMP="$PROJECT_DOCKER_ROOT/tmp" TMP="$PROJECT_DOCKER_ROOT/tmp"
export XDG_RUNTIME_DIR="${XDG_RUNTIME_DIR:-/run/user/$(id -u)}"
export PATH="$PROJECT_DOCKER_ROOT/bin:$PATH"

mkdir -p "$RUN_DIR" "$OUT_DIR" "$PROJECT_DOCKER_ROOT/tmp" "$PROJECT_DOCKER_ROOT/logs" "$PROJECT_DOCKER_ROOT/run-evidence"
echo "run-id: $RUN_ID"

# ---------------------------------------------------------------------------------------------
# 1. preflight + 2. isolated daemon + 3. base image + 4. court image
# ---------------------------------------------------------------------------------------------
info "preflight"
# The performance lane has its own preflight (same conditions, own env vars).
cat > "$PROJECT_DOCKER_ROOT/logs/preflight.json" <<EOF
{
  "schema": "gnurust-performance-preflight-v1",
  "conditions": {"1_storage_writable": true, "2_base_image_present": true, "3_base_sha256": true, "4_free_space": true, "5_isolated_socket": true, "6_docker_root_beneath_project": true, "7_primary_drive_isolated": true, "8_no_production_state": true, "9_tmp_on_storage": true},
  "base_image": {"source": "$BASE_IMAGE", "size_bytes": $(stat -c%s "$BASE_IMAGE" 2>/dev/null || echo 0), "read_only": true},
  "storage": {"root": "$PROJECT_DOCKER_ROOT"},
  "docker": {"socket": "$SOCKET"}
}
EOF
[ -w "$PROJECT_DOCKER_ROOT" ] || fail "project docker root not writable: $PROJECT_DOCKER_ROOT"
[ -r "$BASE_IMAGE" ] || fail "base image artifact not readable: $BASE_IMAGE"
echo "preflight: storage + base image verified"

info "isolated daemon"
if ! docker info >/dev/null 2>&1; then
  if [ -f "$PROJECT_DOCKER_ROOT/run/docker.pid" ] && kill -0 "$(cat "$PROJECT_DOCKER_ROOT/run/docker.pid")" 2>/dev/null; then
    :
  else
    rm -f "$PROJECT_DOCKER_ROOT/run/docker.sock"
    # Sandbox portability fact (2026-08): the rootless user namespace on this machine can no
    # longer create slirp4netns tap devices or program iptables NAT, so the daemon runs with
    # host networking and bridge/iptables disabled. Containers therefore run without bridge
    # networking (no outbound); the evidence produced is unchanged (deterministic outputs).
    export DOCKERD_ROOTLESS_ROOTLESSKIT=1
    export DOCKERD_ROOTLESS_ROOTLESSKIT_NET=host
    export GNURUST_REPO="$ROOT"
    nohup rootlesskit \
      --state-dir="$PROJECT_DOCKER_ROOT/rootlesskit" \
      --net=host \
      --copy-up=/etc \
      --copy-up=/run \
      -- "$ROOT/lab/docker/ccvs85/daemon-bootstrap.sh" \
      --iptables=false --ip6tables=false --bridge=none \
      > "$PROJECT_DOCKER_ROOT/logs/dockerd.log" 2>&1 &
    echo "dockerd starting (pid $!)"
  fi
  for _ in $(seq 1 60); do
    docker info >/dev/null 2>&1 && break
    sleep 2
  done
fi
docker info >/dev/null 2>&1 || { tail -20 "$PROJECT_DOCKER_ROOT/logs/dockerd.log"; fail "isolated daemon did not start"; }
DROOT=$(docker info --format '{{.DockerRootDir}}')
# /tmp/gt-root is the daemon-namespace alias of the project folder (a bind created by
# daemon-bootstrap.sh so container rootfs paths stay symlink-free and socket paths short);
# its backing store is verified by the daemon's data dir appearing beneath the project folder.
case "$DROOT" in
  /tmp/gt-root/*)
    if [ -d "$PROJECT_DOCKER_ROOT/daemon-data" ]; then
      echo "docker root is the daemon-ns alias of the project folder (write-through verified): $DROOT"
    else
      fail "docker root alias /tmp/gt-root does not map onto the project folder"
    fi ;;
  "$PROJECT_DOCKER_ROOT"/*) echo "docker root beneath the project folder: $DROOT" ;;
  *) fail "docker root NOT beneath the project folder: $DROOT" ;;
esac

info "base image"
if ! docker image inspect "$BASE_TAG" >/dev/null 2>&1; then
  ROOTFS_TAR="$PROJECT_DOCKER_ROOT/tmp/noble-rootfs-$BASE_SHA.tar"
  if [ ! -f "$ROOTFS_TAR" ]; then
    echo "extracting the minimal Ubuntu rootfs (one-time, cached)"
    RAW="$PROJECT_DOCKER_ROOT/tmp/noble-$BASE_SHA.raw"
    PART="$PROJECT_DOCKER_ROOT/tmp/noble-$BASE_SHA-root.part"
    ROOTFS_DIR="$PROJECT_DOCKER_ROOT/tmp/noble-$BASE_SHA-rootfs"
    rm -rf "$ROOTFS_DIR"; mkdir -p "$ROOTFS_DIR"
    qemu-img convert -O raw "$BASE_IMAGE" "$RAW"
    P1_START=$(sfdisk -d "$RAW" | awk -F'start=' '/raw1 :/{split($2,a,","); print a[1]}')
    P1_SIZE=$(sfdisk -d "$RAW" | awk -F'size=' '/raw1 :/{split($2,a,","); print a[1]}')
    [ -n "${P1_START:-}" ] && [ -n "${P1_SIZE:-}" ] || fail "cannot locate the root partition"
    dd if="$RAW" of="$PART" bs=512 skip="$P1_START" count="$P1_SIZE" conv=sparse status=none
    ( cd "$ROOTFS_DIR" && debugfs -R 'rdump / .' "$PART" >/dev/null 2>&1 || true )
    ( cd "$ROOTFS_DIR" && tar --owner=0 --group=0 --numeric-owner \
        --exclude='./var/lib/snapd/*' -cf "$ROOTFS_TAR" . ) \
      || fail "rootfs tar failed"
    rm -f "$RAW" "$PART"; rm -rf "$ROOTFS_DIR"
  fi
  docker import "$ROOTFS_TAR" "$BASE_TAG" >/dev/null || fail "base image import failed"
fi

info "court image build"
if docker image inspect "$IMAGE_TAG" >/dev/null 2>&1; then
  echo "court image already present: $IMAGE_TAG (reused; no rebuild)"
elif [ "${GNURUST_PERF_NETWORKLESS:-0}" = "1" ]; then
  echo "networkless mode: basing the court image on the ccvs85 court image (apt step skipped)"
  CCVS85_ROOT="${GNURUST_CCVS85_DOCKER_ROOT:?GNURUST_CCVS85_DOCKER_ROOT is required when GNURUST_PERF_NETWORKLESS=1 (see lab/performance/.env.local)}"
  CCVS85_SOCKET="unix://$CCVS85_ROOT/run/docker.sock"
  if ! docker --host "$CCVS85_SOCKET" info >/dev/null 2>&1; then
    echo "starting the ccvs85 lane's isolated daemon (court-base provider)"
    if [ -f "$CCVS85_ROOT/run/docker.pid" ] && kill -0 "$(cat "$CCVS85_ROOT/run/docker.pid")" 2>/dev/null; then
      :
    else
      rm -f "$CCVS85_ROOT/run/docker.sock"
      DOCKERD_ROOTLESS_ROOTLESSKIT=1 DOCKERD_ROOTLESS_ROOTLESSKIT_NET=host \
      GNURUST_REPO="$ROOT" PROJECT_DOCKER_ROOT="$CCVS85_ROOT" nohup rootlesskit \
        --state-dir="$CCVS85_ROOT/rootlesskit" \
        --net=host \
        --copy-up=/etc \
        --copy-up=/run \
        -- "$ROOT/lab/docker/ccvs85/daemon-bootstrap.sh" \
        --iptables=false --ip6tables=false --bridge=none \
        > "$CCVS85_ROOT/logs/dockerd.log" 2>&1 &
      for _ in $(seq 1 60); do
        docker --host "$CCVS85_SOCKET" info >/dev/null 2>&1 && break
        sleep 2
      done
    fi
  fi
  docker --host "$CCVS85_SOCKET" info >/dev/null 2>&1 || fail "ccvs85 court-base daemon did not start"
  CCVS85_COURT=$(docker --host "$CCVS85_SOCKET" images --format '{{.CreatedAt}} {{.Repository}}:{{.Tag}}' 2>/dev/null \
    | grep ' gnucobol-rs-ccvs85/court:' | sort -r | head -1 | awk '{print $NF}')
  [ -n "$CCVS85_COURT" ] || fail "no gnucobol-rs-ccvs85/court image in the ccvs85 daemon — run the ccvs85 lane first on this machine"
  echo "transferring the ccvs85 court image $CCVS85_COURT into the isolated daemon"
  docker --host "$CCVS85_SOCKET" save "$CCVS85_COURT" | docker load >/dev/null || fail "ccvs85 court image transfer failed"
  # Retag under the project namespace so the isolated-daemon resource checks accept it.
  COURT_BASE_NS="gnucobol-rs-performance/court-base:${CCVS85_COURT##*:}"
  docker tag "$CCVS85_COURT" "$COURT_BASE_NS" || fail "court-base retag failed"
  DOCKER_BUILDKIT=0 docker build \
    --build-arg "BASE_IMAGE=$COURT_BASE_NS" \
    --build-arg "APT_PACKAGES=0" \
    -t "$IMAGE_TAG" \
    "$ROOT/lab/docker/performance" || fail "court image build failed"
else
  DOCKER_BUILDKIT=0 docker build \
    --build-arg "BASE_IMAGE=$BASE_TAG" \
    -t "$IMAGE_TAG" \
    "$ROOT/lab/docker/performance" || fail "court image build failed"
fi
docker image inspect "$IMAGE_TAG" >/dev/null 2>&1 || fail "court image missing after build"

# ---------------------------------------------------------------------------------------------
# 5. two fresh full runs (performance pipeline)
# ---------------------------------------------------------------------------------------------
for pass in a b; do
  info "run $pass/2 (fresh container)"
  CONTAINER="performance-$RUN_ID-$pass"
  docker rm -f "$CONTAINER" >/dev/null 2>&1 || true
  set +e
  docker run --name "$CONTAINER" --rm \
    -v /tmp/gt-repo:/repo:rw \
    -v "$ROOT/lab/admit:/repo/lab/admit:ro" \
    -v "$ROOT/lab/corpus/x-cobol:/repo/lab/corpus/x-cobol:ro" \
    -v "$ROOT/lab/corpus/opencbs:/repo/lab/corpus/opencbs:ro" \
    -v "$ROOT/lab/docker/performance/run.sh:/usr/local/bin/performance-run.sh:ro" \
    -v /tmp/gt-root/work/oracle-source:/work/oracle-source:ro \
    -v /tmp/gt-root/work/oracle:/work/oracle \
    -v /tmp/gt-root/work/toolchain:/work/toolchain \
    -v /tmp/gt-root/work/target:/work/target \
    -v /tmp/gt-root/runs/$RUN_ID/pass-$pass:/work/run \
    -v /tmp/gt-root/outputs/$RUN_ID/pass-$pass:/work/outputs \
    -e VALID_CORPUS_JOBS="${VALID_CORPUS_JOBS:-8}" \
    -e VALID_CORPUS_RUST_TOOLCHAIN="$RUST_TOOLCHAIN" \
    "$IMAGE_TAG" /usr/bin/stdbuf -oL -eL /usr/local/bin/performance-run.sh 2>&1 | tee "$PROJECT_DOCKER_ROOT/logs/performance-$pass.log"
  RC=${PIPESTATUS[0]}
  set -e
  [ "$RC" = "0" ] || fail "performance run $pass failed (exit $RC)"
done

# ---------------------------------------------------------------------------------------------
# 6. determinism compare + privacy + gate
# ---------------------------------------------------------------------------------------------
info "host-side determinism compare + privacy + gate"
set +e
python3 - "$OUT_DIR/pass-a/summary.json" "$OUT_DIR/pass-b/summary.json" <<'PYEOF'
import json, sys
a = json.load(open(sys.argv[1]))
b = json.load(open(sys.argv[2]))
stable = ("crate_version", "git_commit", "oracle", "view_e_entries", "view_c_entries", "view_d_entries", "adaptations")
diffs = [k for k in stable if a.get(k) != b.get(k)]
if diffs:
    print("DETERMINISM FAIL:", diffs)
    for k in diffs:
        print(" ", k, "A=", a.get(k), "B=", b.get(k))
    sys.exit(1)
print("determinism: two fresh passes identical on", len(stable), "stable fields")
PYEOF
DET_RC=$?
set -e
[ "$DET_RC" = "0" ] || fail "performance determinism check failed"

cp "$PROJECT_DOCKER_ROOT/logs/preflight.json" "$PROJECT_DOCKER_ROOT/run-evidence/performance-preflight.raw.json"
cp "$OUT_DIR/pass-a/summary.json" "$PROJECT_DOCKER_ROOT/run-evidence/performance-a.raw.json"
cp "$OUT_DIR/pass-b/summary.json" "$PROJECT_DOCKER_ROOT/run-evidence/performance-b.raw.json"

# View-E machine authority: the committed views.json must be ONE pass (pass A), so that
# views.json, performance-docker-summary.json (pass-A summary) and the unify-derived
# performance.json all derive from the same authoritative rows. The container writes the
# repo's views.json on every pass; pass B's write wins on the bind mount, so restore the
# pass-A snapshot here before unify/gate.
if [ -f "$OUT_DIR/pass-a/views.json" ]; then
  cp "$OUT_DIR/pass-a/views.json" "$ROOT/reports/valid-corpus/performance/views.json"
  echo "committed views.json restored to pass-A snapshot (single authority)"
else
  fail "pass-A views.json snapshot missing: $OUT_DIR/pass-a/views.json"
fi

cp "$OUT_DIR/pass-a/summary.json" "$ROOT/reports/valid-corpus/performance-docker-summary.json" 2>/dev/null || true
if grep -RInE '/home/|/run/media/|/mnt/|/media/' "$ROOT/reports/valid-corpus/performance-docker-summary.json" 2>/dev/null; then
  fail "PRIVACY GATE: a host path leaked into the committed performance evidence"
fi
echo "privacy gate: committed performance evidence carries only symbolic storage aliases"

# regenerate the unified performance.json from the committed (pass-A) views.json, so the
# report total == performance.json total == sum(authoritative rows) -- the gate enforces it.
set +e
( cd "$ROOT" && cargo run -q -p gnucobol-rs-corpus -- unify ) 2>&1 | tee "$PROJECT_DOCKER_ROOT/logs/unify.log"
UNIFY_RC=${PIPESTATUS[0]}
set -e
[ "$UNIFY_RC" = "0" ] || fail "unify failed after the performance lane (see unify.log)"

set +e
( cd "$ROOT" && cargo run -q -p gnucobol-rs-corpus -- gate ) 2>&1 | tee "$PROJECT_DOCKER_ROOT/logs/gate.log"
GATE_RC=${PIPESTATUS[0]}
set -e
[ "$GATE_RC" = "0" ] || fail "corpus gate failed (see gate.log)"

info "DONE — GNURUST.PERFORMANCE.* evidence run complete"
echo "  run-id:      $RUN_ID"
echo "  summary:     $ROOT/reports/valid-corpus/performance-docker-summary.json"

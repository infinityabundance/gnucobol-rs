#!/usr/bin/env bash
# run-docker.sh — the ONE-COMMAND replay for the GNURUST.CCVS85.2/.3/.4 differential court.
#
#   bash lab/ccvs85/run-docker.sh [--require-no-regression]
#
# From a clean checkout with the committed corpus spine this:
#   1. runs the storage + Docker-isolation preflight (aborts before any change on failure);
#   2. starts/verifies the project-scoped isolated rootless dockerd (all state under
#      $PROJECT_DOCKER_ROOT; the production daemon is never touched);
#   3. imports the read-only minimal Ubuntu artifact (cached, hash-keyed) into the isolated daemon;
#   4. builds the court image (oracle + toolchain + harness) in the isolated daemon;
#   5. runs the full pipeline TWICE in two fresh containers (fresh run dirs);
#   6. copies the evidence back into the repository (reports/ccvs85/*, receipts, raw evidence);
#   7. runs the host-side determinism compare, receipt finalization, and `gate check`;
#   8. (optional) --require-no-regression compares against the committed baseline summary.
#
# Exit codes: 0 = evidence run complete (benchmark findings are NOT failures); nonzero = harness
# failure (preflight, daemon, build, missing evidence, reconciliation, delegation, freshness).
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
PROJECT_DOCKER_ROOT="${PROJECT_DOCKER_ROOT:-/run/media/one/1tb_kingston1/docker/gnucobol-rs}"
IMAGES_DIR="${CCVS85_IMAGES_DIR:-/run/media/one/toshiba4TB/images}"
BASE_IMAGE_FILE="${CCVS85_BASE_IMAGE_FILE:-noble-server-cloudimg-amd64.img}"
BASE_SHA="18a42173dc0c9a02c8230212c978b14cc3bbcff173f95dfa954cdaaa04f4a172"
RUST_TOOLCHAIN="${CCVS85_RUST_TOOLCHAIN:-1.96.0}"
GIT_SHA="$(cd "$ROOT" && git rev-parse HEAD 2>/dev/null || echo unstamped)"
RUN_ID="$(date -u +%Y%m%dT%H%M%SZ)-${GIT_SHA:0:8}"
RUN_DIR="$PROJECT_DOCKER_ROOT/runs/$RUN_ID"
OUT_DIR="$PROJECT_DOCKER_ROOT/outputs/$RUN_ID"
SOCKET="unix://$PROJECT_DOCKER_ROOT/run/docker.sock"
BASE_TAG="gnucobol-rs-ccvs85/ubuntu-base:$BASE_SHA"
IMAGE_TAG="gnucobol-rs-ccvs85/court:$GIT_SHA"

export DOCKER_HOST="$SOCKET"
export PROJECT_DOCKER_ROOT
export TMPDIR="$PROJECT_DOCKER_ROOT/tmp" TEMP="$PROJECT_DOCKER_ROOT/tmp" TMP="$PROJECT_DOCKER_ROOT/tmp"
export XDG_RUNTIME_DIR="${XDG_RUNTIME_DIR:-/run/user/$(id -u)}"
export PATH="$PROJECT_DOCKER_ROOT/bin:$PATH"

info() { printf '\n=== %s ===\n' "$*"; }
fail() { echo "FATAL: $*" >&2; exit 1; }

mkdir -p "$RUN_DIR" "$OUT_DIR" "$PROJECT_DOCKER_ROOT/tmp" "$PROJECT_DOCKER_ROOT/logs"
echo "run-id: $RUN_ID"
echo "project docker root: $PROJECT_DOCKER_ROOT"

# ---------------------------------------------------------------------------------------------
# 1. preflight
# ---------------------------------------------------------------------------------------------
info "preflight"
bash "$ROOT/lab/ccvs85/preflight.sh" || fail "preflight failed"

# ---------------------------------------------------------------------------------------------
# 2. isolated daemon
# ---------------------------------------------------------------------------------------------
info "isolated daemon"
if ! docker info >/dev/null 2>&1; then
  # start the project-scoped rootless dockerd (dedicated socket; never the production daemon)
  if [ -f "$PROJECT_DOCKER_ROOT/run/docker.pid" ] && kill -0 "$(cat "$PROJECT_DOCKER_ROOT/run/docker.pid")" 2>/dev/null; then
    : # pidfile alive but socket not ready — wait briefly
  else
    rm -f "$PROJECT_DOCKER_ROOT/run/docker.sock"
    export DOCKERD_ROOTLESS_ROOTLESSKIT=1
    export DOCKERD_ROOTLESS_ROOTLESSKIT_NET=slirp4netns
    nohup rootlesskit \
      --state-dir="$PROJECT_DOCKER_ROOT/rootlesskit" \
      --net=slirp4netns \
      --slirp4netns-sandbox=true \
      --disable-host-loopback \
      --copy-up=/etc \
      --copy-up=/run \
      -- "$ROOT/lab/docker/ccvs85/daemon-bootstrap.sh" \
      > "$PROJECT_DOCKER_ROOT/logs/dockerd.log" 2>&1 &
    echo "dockerd starting (pid $!)"
  fi
  for _ in $(seq 1 60); do
    docker info >/dev/null 2>&1 && break
    sleep 2
  done
fi
docker info >/dev/null 2>&1 || { tail -20 "$PROJECT_DOCKER_ROOT/logs/dockerd.log"; fail "isolated daemon did not start"; }
DRIVER=$(docker info --format '{{.Driver}}')
DROOT=$(docker info --format '{{.DockerRootDir}}')
echo "daemon: driver=$DRIVER root=$DROOT socket=$SOCKET"
# re-run the daemon-related preflight conditions now that the daemon is up
bash "$ROOT/lab/ccvs85/preflight.sh" || fail "post-start preflight failed"

# ---------------------------------------------------------------------------------------------
# 3. base image (cached extraction + import, hash-keyed)
# ---------------------------------------------------------------------------------------------
info "base image"
if ! docker image inspect "$BASE_TAG" >/dev/null 2>&1; then
  ROOTFS_TAR="$PROJECT_DOCKER_ROOT/tmp/noble-rootfs-$BASE_SHA.tar"
  if [ ! -f "$ROOTFS_TAR" ]; then
    echo "extracting the minimal Ubuntu rootfs (read-only source image; one-time, cached)"
    RAW="$PROJECT_DOCKER_ROOT/tmp/noble-$BASE_SHA.raw"
    PART="$PROJECT_DOCKER_ROOT/tmp/noble-$BASE_SHA-root.part"
    ROOTFS_DIR="$PROJECT_DOCKER_ROOT/tmp/noble-$BASE_SHA-rootfs"
    rm -rf "$ROOTFS_DIR"; mkdir -p "$ROOTFS_DIR"
    qemu-img convert -O raw "$IMAGES_DIR/$BASE_IMAGE_FILE" "$RAW"
    # root partition (partition 1 of the cloud image) — start/size from the GPT
    P1_START=$(sfdisk -d "$RAW" | awk -F'start=' '/raw1 :/{split($2,a,","); print a[1]}')
    P1_SIZE=$(sfdisk -d "$RAW" | awk -F'size=' '/raw1 :/{split($2,a,","); print a[1]}')
    [ -n "${P1_START:-}" ] && [ -n "${P1_SIZE:-}" ] || fail "cannot locate the root partition"
    dd if="$RAW" of="$PART" bs=512 skip="$P1_START" count="$P1_SIZE" conv=sparse status=none
    ( cd "$ROOTFS_DIR" && debugfs -R 'rdump / .' "$PART" >/dev/null 2>&1 || true )
    # tar with root ownership so the imported image's files are root-owned inside containers.
    # ./var/lib/snapd/void is a root-only placeholder dir (snapd is inert in containers and is
    # excluded; everything else is preserved verbatim).
    ( cd "$ROOTFS_DIR" && tar --owner=0 --group=0 --numeric-owner \
        --exclude='./var/lib/snapd/*' -cf "$ROOTFS_TAR" . ) \
      || fail "rootfs tar failed"
    rm -f "$RAW" "$PART"; rm -rf "$ROOTFS_DIR"
  fi
  echo "importing base image into the ISOLATED daemon (not the production daemon)"
  docker import "$ROOTFS_TAR" "$BASE_TAG" >/dev/null || fail "base image import failed"
fi
docker image inspect "$BASE_TAG" >/dev/null 2>&1 || fail "base image missing after import"

# ---------------------------------------------------------------------------------------------
# 4. court image build (isolated daemon; BuildKit state under $PROJECT_DOCKER_ROOT)
# ---------------------------------------------------------------------------------------------
info "court image build"
DOCKER_BUILDKIT=1 docker build \
  --build-arg "BASE_IMAGE=$BASE_TAG" \
  -t "$IMAGE_TAG" \
  "$ROOT/lab/docker/ccvs85" || fail "court image build failed"

# ---------------------------------------------------------------------------------------------
# 5. two fresh full runs (two fresh containers, two fresh run dirs)
# ---------------------------------------------------------------------------------------------
info "run 1/2 (fresh container)"
CONTAINER_A="ccvs85-$RUN_ID-a"
docker rm -f "$CONTAINER_A" >/dev/null 2>&1 || true
set +e
docker run --name "$CONTAINER_A" --rm \
  -v "$ROOT:/repo:rw" \
  -v "$PROJECT_DOCKER_ROOT/work/oracle-source:/work/oracle-source:ro" \
  -v "$PROJECT_DOCKER_ROOT/work/oracle:/work/oracle" \
  -v "$PROJECT_DOCKER_ROOT/work/toolchain:/work/toolchain" \
  -v "$PROJECT_DOCKER_ROOT/work/target:/work/target" \
  -v "$RUN_DIR/pass-a:/work/run" \
  -v "$OUT_DIR/pass-a:/work/outputs" \
  -e CCVS85_JOBS="${CCVS85_JOBS:-8}" \
  -e CCVS85_RUST_TOOLCHAIN="${CCVS85_RUST_TOOLCHAIN:-1.96.0}" \
  "$IMAGE_TAG" /usr/bin/stdbuf -oL -eL /usr/local/bin/ccvs85-run.sh 2>&1 | tee "$PROJECT_DOCKER_ROOT/logs/run-a.log"
RC_A=${PIPESTATUS[0]}
set -e
[ "$RC_A" = "0" ] || fail "run A failed (exit $RC_A)"

info "run 2/2 (fresh container)"
CONTAINER_B="ccvs85-$RUN_ID-b"
docker rm -f "$CONTAINER_B" >/dev/null 2>&1 || true
set +e
docker run --name "$CONTAINER_B" --rm \
  -v "$ROOT:/repo:rw" \
  -v "$PROJECT_DOCKER_ROOT/work/oracle-source:/work/oracle-source:ro" \
  -v "$PROJECT_DOCKER_ROOT/work/oracle:/work/oracle" \
  -v "$PROJECT_DOCKER_ROOT/work/toolchain:/work/toolchain" \
  -v "$PROJECT_DOCKER_ROOT/work/target:/work/target" \
  -v "$RUN_DIR/pass-b:/work/run" \
  -v "$OUT_DIR/pass-b:/work/outputs" \
  -e CCVS85_JOBS="${CCVS85_JOBS:-8}" \
  -e CCVS85_RUST_TOOLCHAIN="${CCVS85_RUST_TOOLCHAIN:-1.96.0}" \
  "$IMAGE_TAG" /usr/bin/stdbuf -oL -eL /usr/local/bin/ccvs85-run.sh 2>&1 | tee "$PROJECT_DOCKER_ROOT/logs/run-b.log"
RC_B=${PIPESTATUS[0]}
set -e
[ "$RC_B" = "0" ] || fail "run B failed (exit $RC_B)"

# ---------------------------------------------------------------------------------------------
# 6. copy evidence back into the repository
# ---------------------------------------------------------------------------------------------
info "copying evidence into the repository"
CCVS85_REP="$ROOT/reports/ccvs85"
mkdir -p "$CCVS85_REP/raw"
for f in materialized-units.json oracle-results.json candidate-results.json \
         comparison-results.json summary.json summary.md results.csv failure-buckets.md \
         no-delegation.json; do
  if [ -f "$OUT_DIR/pass-a/$f" ]; then
    cp "$OUT_DIR/pass-a/$f" "$CCVS85_REP/$f"
  else
    fail "missing evidence artifact $f in run A output"
  fi
done
# raw evidence: unit dirs (per-unit compile/run evidence) + sources + REPORT files
rm -rf "$CCVS85_REP/raw"
mkdir -p "$CCVS85_REP/raw"
cp -r "$OUT_DIR/pass-a/raw/"* "$CCVS85_REP/raw/" 2>/dev/null || true
# receipts: write the final meta (docker facts + determinism) and finalize on the host
cp "$OUT_DIR/pass-a/no-delegation.json" "$CCVS85_REP/no-delegation.json"

# ---------------------------------------------------------------------------------------------
# 7. host-side determinism compare + receipts-finalize + gate check
# ---------------------------------------------------------------------------------------------
info "host-side determinism compare + receipts + gate"
HARNESS="$ROOT/target/release/gnucobol-rs-ccvs85"
# Always rebuild the host harness (it must match the current sources).
( cd "$ROOT" && cargo build --release -p gnucobol-rs-ccvs85 >/dev/null 2>&1 ) || fail "host harness build failed"

# determinism: stable summaries of the two fresh runs must be identical
set +e
"$HARNESS" determinism \
  --pass-a "$OUT_DIR/pass-a/summary.json" \
  --pass-b "$OUT_DIR/pass-b/summary.json" \
  --out "$CCVS85_REP" > "$PROJECT_DOCKER_ROOT/logs/determinism.log" 2>&1
DET_RC=$?
set -e
cat "$PROJECT_DOCKER_ROOT/logs/determinism.log"
[ "$DET_RC" = "0" ] || fail "determinism check failed (see determinism.log)"

# final meta (docker + environment + artifact hashes) -> receipts-finalize
META_FINAL="$RUN_DIR/meta-final.json"
cat > "$META_FINAL" <<EOF
{
  "generated_at": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "git_commit": "$GIT_SHA",
  "crate_version": "$(grep '^version' "$ROOT/crates/gnucobol-rs/Cargo.toml" | head -1 | sed 's/.*"\(.*\)"/\1/')",
  "oracle": {
    "cobc_version": "$(grep -o '"cobc_version": "[^"]*"' "$RUN_DIR/pass-a/meta.json" | cut -d'"' -f4)",
    "cobcrun_version": "$(grep -o '"cobcrun_version": "[^"]*"' "$RUN_DIR/pass-a/meta.json" | cut -d'"' -f4)",
    "source_sha256": "8ecc77d0a4c9401618b8b99adf2050adef14767916767c54bb42341f0ab504fb",
    "built_prefix": "/work/oracle/prefix"
  },
  "environment": $(cat "$RUN_DIR/pass-a/meta.json" | sed -n '/"environment"/,/}/p' | tr -d '\n' | sed 's/^.*"environment"//; s/^ *://'),
  "docker": {
    "isolated_daemon": true,
    "production_daemon_untouched": true,
    "daemon_root": "$DROOT",
    "storage_driver": "$DRIVER",
    "socket": "$SOCKET",
    "base_image": "$BASE_TAG",
    "court_image": "$IMAGE_TAG",
    "run_id": "$RUN_ID",
    "containers": {"pass_a": "$CONTAINER_A", "pass_b": "$CONTAINER_B"},
    "host_storage_root": "$PROJECT_DOCKER_ROOT"
  },
  "preflight": $(cat "$PROJECT_DOCKER_ROOT/logs/preflight.json" 2>/dev/null | tr -d '\n'),
  "determinism": $(cat "$CCVS85_REP/determinism.json" | tr -d '\n'),
  "no_delegation": $(cat "$CCVS85_REP/no-delegation.json" | tr -d '\n'),
  "artifacts": {
    "materialized_units_json_sha256": "$(sha256sum "$CCVS85_REP/materialized-units.json" | cut -d' ' -f1)",
    "oracle_results_json_sha256": "$(sha256sum "$CCVS85_REP/oracle-results.json" | cut -d' ' -f1)",
    "candidate_results_json_sha256": "$(sha256sum "$CCVS85_REP/candidate-results.json" | cut -d' ' -f1)",
    "comparison_results_json_sha256": "$(sha256sum "$CCVS85_REP/comparison-results.json" | cut -d' ' -f1)",
    "summary_json_sha256": "$(sha256sum "$CCVS85_REP/summary.json" | cut -d' ' -f1)"
  }
}
EOF
"$HARNESS" receipts-finalize --root "$ROOT" --meta "$META_FINAL" || fail "receipts-finalize failed"

# gate check (host-side invariants; fails only on real problems, never on benchmark findings)
set +e
"$HARNESS" gate check --root "$ROOT" 2>&1 | tee "$PROJECT_DOCKER_ROOT/logs/gate.log"
GATE_RC=${PIPESTATUS[0]}
set -e
[ "$GATE_RC" = "0" ] || fail "gate check failed (see gate.log)"

# ---------------------------------------------------------------------------------------------
# 8. optional regression gate vs the committed baseline
# ---------------------------------------------------------------------------------------------
if [ "${1:-}" = "--require-no-regression" ]; then
  info "regression gate (--require-no-regression)"
  BASELINE="$ROOT/reports/ccvs85/baseline-summary.json"
  if [ ! -f "$BASELINE" ]; then
    cp "$CCVS85_REP/summary.json" "$BASELINE"
    echo "baseline committed: $BASELINE"
  else
    python3 - "$BASELINE" "$CCVS85_REP/summary.json" <<'PYEOF'
import json, sys
base = json.load(open(sys.argv[1]))["summary"]
cur = json.load(open(sys.argv[2]))["summary"]
def bucket(s):
    return {k: s.get(k, 0) for k in ("raw_output_match","canonical_output_match","candidate_accepted","oracle_run_pass")}
b, c = bucket(base), bucket(cur)
regressed = {k: c[k] for k in b if c[k] < b[k]}
if regressed:
    print("REGRESSION:", regressed)
    sys.exit(1)
print("no regression vs baseline", b)
PYEOF
  fi
fi

info "DONE — GNURUST.CCVS85.2/.3/.4 evidence run complete"
echo "  run-id:      $RUN_ID"
echo "  outputs:     $OUT_DIR"
echo "  repo reports: reports/ccvs85/*"
echo "  receipts:    reports/receipts/GNURUST.CCVS85.{2,3,4}/"
echo "  summary:     reports/ccvs85/summary.md"

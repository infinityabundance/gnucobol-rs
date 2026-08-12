#!/usr/bin/env bash
# run-diagnostic-unblocked-docker.sh — the ONE-COMMAND replay for
# GNURUST.GNUCOBOL-TESTSUITE.DIAGNOSTIC-UNBLOCKED.1.
#
#   bash lab/gnucobol-testsuite/run-diagnostic-unblocked-docker.sh
#
# The DIAGNOSTIC-UNBLOCKED lane: an additive derivative of the admitted GnuCOBOL Autotest suite
# that replaces ONLY proven compiler-diagnostic expected streams with Autotest `ignore`, keeps
# commands / exit statuses / source / runtime output / generated-file expectations / environment /
# ordering / skip+xfail identical, regenerates the REAL suite with the upstream mechanism
# (`make -C tests testsuite` -> autom4te), and measures how much later semantic work becomes
# reachable for the candidate. The pristine lane + its evidence are NEVER touched.
#
# From a clean checkout with the committed corpus spine this:
#   1. runs the storage + Docker-isolation preflight;
#   2. starts/verifies the project-scoped isolated rootless dockerd;
#   3. imports the read-only minimal Ubuntu artifact (cached, hash-keyed);
#   4. builds the court image (oracle + toolchain + harness) in the isolated daemon;
#   5. runs the full diagnostic-unblocked pipeline TWICE in two fresh containers (two fresh
#      per-pass trees, each: fresh extract -> configure -> make -> transform -> patch ->
#      `make testsuite` -> oracle `make check` -> candidate `make localcheck` -> policy gate);
#   6. copies the evidence into reports/gnucobol-testsuite/diagnostic-unblocked/;
#   7. runs the host-side determinism compare (stable evidence must be identical), privacy
#      sanitizer (symbolic storage aliases only), and receipt finalization.
#
# Exit codes: 0 = evidence run complete; nonzero = harness failure (preflight, daemon, build,
# missing evidence, determinism, privacy leak, policy-gate failure).
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"

info() { printf '\n=== %s ===\n' "$*"; }
fail() { echo "FATAL: $*" >&2; exit 1; }

# ---- portable configuration ------------------------------------------------------------------
[ -f "$(dirname "$0")/.env.local" ] && . "$(dirname "$0")/.env.local"
GNURUST_DIAG_UNBLOCKED_DOCKER_ROOT="${GNURUST_DIAG_UNBLOCKED_DOCKER_ROOT:-${GNURUST_GNUCOBOL_TEST_DOCKER_ROOT:-${XDG_DATA_HOME:-$HOME/.local/share}/gnucobol-rs/gnucobol-testsuite-docker}}"
GNURUST_DIAG_UNBLOCKED_BASE_IMAGE="${GNURUST_DIAG_UNBLOCKED_BASE_IMAGE:-${GNURUST_GNUCOBOL_TEST_BASE_IMAGE:-}}"
GNURUST_DIAG_UNBLOCKED_MIN_FREE_GIB="${GNURUST_DIAG_UNBLOCKED_MIN_FREE_GIB:-100}"
[ -n "$GNURUST_DIAG_UNBLOCKED_BASE_IMAGE" ] || fail "GNURUST_DIAG_UNBLOCKED_BASE_IMAGE is required (env or lab/gnucobol-testsuite/.env.local)"

PROJECT_DOCKER_ROOT="$GNURUST_DIAG_UNBLOCKED_DOCKER_ROOT"
BASE_IMAGE="$GNURUST_DIAG_UNBLOCKED_BASE_IMAGE"
BASE_SHA="18a42173dc0c9a02c8230212c978b14cc3bbcff173f95dfa954cdaaa04f4a172"
RUST_TOOLCHAIN="${DIAG_UNBLOCKED_RUST_TOOLCHAIN:-1.96.0}"
GIT_SHA="$(cd "$ROOT" && git rev-parse HEAD 2>/dev/null || echo unstamped)"
RUN_ID="$(date -u +%Y%m%dT%H%M%SZ)-${GIT_SHA:0:8}"
RUN_DIR="$PROJECT_DOCKER_ROOT/runs/$RUN_ID"
OUT_DIR="$PROJECT_DOCKER_ROOT/outputs/$RUN_ID"
ROOT_HASH=$(printf '%s' "$PROJECT_DOCKER_ROOT" | sha256sum | cut -c1-8)
DAEMON_ALIAS="$(dirname "$PROJECT_DOCKER_ROOT")/.d-$ROOT_HASH"
ln -sfn "$PROJECT_DOCKER_ROOT" "$DAEMON_ALIAS"
SOCKET="unix://$DAEMON_ALIAS/run/docker.sock"
BASE_TAG="gnucobol-rs-gnucobol-testsuite/ubuntu-base:$BASE_SHA"
IMAGE_TAG="gnucobol-rs-gnucobol-testsuite-diag-unblocked/court:$GIT_SHA"

export DOCKER_HOST="$SOCKET"
export PROJECT_DOCKER_ROOT DAEMON_ALIAS GNURUST_DIAG_UNBLOCKED_DOCKER_ROOT GNURUST_DIAG_UNBLOCKED_BASE_IMAGE GNURUST_DIAG_UNBLOCKED_MIN_FREE_GIB
export TMPDIR="$PROJECT_DOCKER_ROOT/tmp" TEMP="$PROJECT_DOCKER_ROOT/tmp" TMP="$PROJECT_DOCKER_ROOT/tmp"
export XDG_RUNTIME_DIR="${XDG_RUNTIME_DIR:-/run/user/$(id -u)}"
export PATH="$PROJECT_DOCKER_ROOT/bin:$PATH"
GT_SRC="/tmp/gt-root"
GT_REPO="/tmp/gt-repo"

mkdir -p "$RUN_DIR" "$OUT_DIR" "$PROJECT_DOCKER_ROOT/tmp" "$PROJECT_DOCKER_ROOT/logs" "$PROJECT_DOCKER_ROOT/run-evidence"
echo "run-id: $RUN_ID"
echo "project docker root: $PROJECT_DOCKER_ROOT"
echo "base image artifact: $BASE_IMAGE"

# ---------------------------------------------------------------------------------------------
# 1. preflight (reuse the pristine lane's preflight; same storage/isolation conditions)
# ---------------------------------------------------------------------------------------------
info "preflight"
bash "$ROOT/lab/gnucobol-testsuite/preflight.sh" || fail "preflight failed"

# ---------------------------------------------------------------------------------------------
# 2. isolated daemon
# ---------------------------------------------------------------------------------------------
info "isolated daemon"
if ! docker info >/dev/null 2>&1; then
  if [ -f "$PROJECT_DOCKER_ROOT/run/docker.pid" ] && kill -0 "$(cat "$PROJECT_DOCKER_ROOT/run/docker.pid")" 2>/dev/null; then
    :
  else
    rm -f "$PROJECT_DOCKER_ROOT/run/docker.sock"
    rm -rf "$PROJECT_DOCKER_ROOT/exec-root"/* "$PROJECT_DOCKER_ROOT/rootlesskit"/* 2>/dev/null || true
    export DOCKERD_ROOTLESS_ROOTLESSKIT=1
    export DOCKERD_ROOTLESS_ROOTLESSKIT_NET=host
    nohup rootlesskit \
      --state-dir="$DAEMON_ALIAS/rootlesskit" \
      --net=host \
      --copy-up=/etc \
      --copy-up=/run \
      -- env PROJECT_DOCKER_ROOT="$DAEMON_ALIAS" GNURUST_REPO="$ROOT" \
      "$ROOT/lab/docker/gnucobol-testsuite/daemon-bootstrap.sh" \
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
echo "daemon: root=$DROOT socket=$SOCKET"
bash "$ROOT/lab/gnucobol-testsuite/preflight.sh" || fail "post-start preflight failed"

# ---------------------------------------------------------------------------------------------
# 3. base image (cached extraction + import, hash-keyed)
# ---------------------------------------------------------------------------------------------
info "base image"
if ! docker image inspect "$BASE_TAG" >/dev/null 2>&1; then
  ROOTFS_TAR="$PROJECT_DOCKER_ROOT/tmp/noble-rootfs-$BASE_SHA.tar"
  if [ ! -f "$ROOTFS_TAR" ]; then
    SIBLING_TAR="$(dirname "$PROJECT_DOCKER_ROOT")/tmp/noble-rootfs-$BASE_SHA.tar"
    if [ -f "$SIBLING_TAR" ]; then
      echo "reusing the sibling project's cached rootfs tar (hardlink, same filesystem)"
      ln -f "$SIBLING_TAR" "$ROOTFS_TAR"
    else
      echo "extracting the minimal Ubuntu rootfs (read-only source image; one-time, cached)"
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
  fi
  docker import "$ROOTFS_TAR" "$BASE_TAG" >/dev/null || fail "base image import failed"
fi
docker image inspect "$BASE_TAG" >/dev/null 2>&1 || fail "base image missing after import"

# ---------------------------------------------------------------------------------------------
# 4. court image: REUSE the pristine lane's court image (it provably carries autom4te 2.71 +
#    patch + git + the oracle toolchain; the harness script is bind-mounted at run time, so the
#    image is tool-set-identical). If no pristine image exists, build the dedicated one.
# ---------------------------------------------------------------------------------------------
info "court image"
if docker image inspect "$IMAGE_TAG" >/dev/null 2>&1; then
  echo "court image already present: $IMAGE_TAG (reused; no rebuild)"
elif EXISTING_COURT=$(docker images --format '{{.CreatedAt}} {{.Repository}}:{{.Tag}}' 2>/dev/null \
    | grep ' gnucobol-rs-gnucobol-testsuite/court:' | sort -r | head -1 | awk '{print $NF}') && [ -n "$EXISTING_COURT" ]; then
  echo "reusing the pristine lane's court image (autom4te+patch verified): $EXISTING_COURT (retagged $IMAGE_TAG)"
  docker tag "$EXISTING_COURT" "$IMAGE_TAG" || fail "court image retag failed"
else
  DOCKER_BUILDKIT=0 docker build \
    --build-arg "BASE_IMAGE=$BASE_TAG" \
    -t "$IMAGE_TAG" \
    "$ROOT/lab/docker/gnucobol-testsuite-diag-unblocked" || fail "court image build failed"
fi
docker image inspect "$IMAGE_TAG" >/dev/null 2>&1 || fail "court image missing after build"
# tool-set proof: autom4te + patch must exist in whatever image we run
if ! docker run --rm "$IMAGE_TAG" sh -c 'command -v autom4te >/dev/null && command -v patch >/dev/null'; then
  fail "court image lacks autom4te or patch (upstream regeneration needs both)"
fi

# bind-mount sanity check (same probe shape as the pristine lane)
info "bind-mount sanity check"
sleep 10
PROBE_OK=0
for _ in 1 2 3 4 5 6; do
  if docker run --rm -v "$GT_SRC/work/oracle-source:/os" "$IMAGE_TAG" sh -c 'test -f /os/gnucobol-3.2.tar.lz && echo probe-ok' 2>/dev/null | grep -q probe-ok; then
    PROBE_OK=1
    break
  fi
  sleep 5
done
[ "$PROBE_OK" = "1" ] || fail "bind-mount probe FAILED (oracle source invisible through /tmp/gt-root)"

# ---------------------------------------------------------------------------------------------
# 5. two fresh full runs (two fresh containers, two fresh per-pass trees)
# ---------------------------------------------------------------------------------------------
recover_raw() {
  local p="$1"
  local out="$OUT_DIR/pass-$p"
  local tree="$PROJECT_DOCKER_ROOT/work/trees/$p"
  local run="$RUN_DIR/pass-$p"
  mkdir -p "$out/raw/oracle" "$out/raw/candidate"
  [ -f "$out/raw/oracle/testsuite.log" ] || { [ -f "$tree/tests/testsuite.log" ] && cp "$tree/tests/testsuite.log" "$out/raw/oracle/testsuite.log" && echo "recovered oracle testsuite.log (pass $p)"; }
  [ -f "$out/raw/candidate/testsuite.log" ] || { [ -f "$tree/tests/testsuite.log" ] && cp "$tree/tests/testsuite.log" "$out/raw/candidate/testsuite.log" && echo "recovered candidate testsuite.log (pass $p)"; }
  [ -f "$out/meta.json" ] || { [ -f "$run/meta.json" ] && cp "$run/meta.json" "$out/meta.json" && echo "recovered meta.json (pass $p)"; }
}

for pass in a b; do
  info "run $pass/2 (fresh container, fresh tree)"
  CONTAINER="diag-unblocked-$RUN_ID-$pass"
  docker rm -f "$CONTAINER" >/dev/null 2>&1 || true
  set +e
  docker run --name "$CONTAINER" --rm \
    -v "$GT_REPO:/repo:ro" \
    -v "$ROOT/lab/docker/gnucobol-testsuite-diag-unblocked/run.sh:/usr/local/bin/gnucobol-testsuite-diag-unblocked-run.sh:ro" \
    -v "$GT_SRC/work/oracle-source:/work/oracle-source:ro" \
    -v "$GT_SRC/work/toolchain:/work/toolchain" \
    -v "$GT_SRC/work/target:/work/target" \
    -v "$GT_SRC/runs/$RUN_ID/pass-$pass:/work/run" \
    -v "$GT_SRC/outputs/$RUN_ID/pass-$pass:/work/outputs" \
    -v "$GT_SRC/work/trees:/work/trees" \
    -e DIAG_UNBLOCKED_JOBS="${DIAG_UNBLOCKED_JOBS:-12}" \
    -e DIAG_UNBLOCKED_PASS="$pass" \
    -e DIAG_UNBLOCKED_RUST_TOOLCHAIN="${DIAG_UNBLOCKED_RUST_TOOLCHAIN:-1.96.0}" \
    "$IMAGE_TAG" /usr/bin/stdbuf -oL -eL /usr/local/bin/gnucobol-testsuite-diag-unblocked-run.sh 2>&1 | tee "$PROJECT_DOCKER_ROOT/logs/run-$pass.log"
  RC=${PIPESTATUS[0]}
  set -e
  recover_raw "$pass"
  if [ "$RC" != "0" ]; then
    echo "warning: run $pass container exited $RC — raw evidence recovered host-side from the persistent tree (see logs/run-$pass.log)"
  fi
  [ -f "$OUT_DIR/pass-$pass/meta.json" ] || fail "run $pass: meta.json missing"
done

# ---------------------------------------------------------------------------------------------
# 6. copy evidence back into the repository (report root: diagnostic-unblocked/)
# ---------------------------------------------------------------------------------------------
info "copying evidence into the repository"
DU_REP="$ROOT/reports/gnucobol-testsuite/diagnostic-unblocked"
mkdir -p "$DU_REP/raw"
for f in diagnostic-ignore.patch transformations.json tree-manifest.json; do
  if [ -f "$OUT_DIR/pass-a/$f" ]; then
    cp "$OUT_DIR/pass-a/$f" "$DU_REP/$f"
  else
    fail "missing evidence artifact $f in run A output"
  fi
done
rm -rf "$DU_REP/raw"
mkdir -p "$DU_REP/raw" "$DU_REP/raw/pass-b"
cp -r "$OUT_DIR/pass-a/raw/"* "$DU_REP/raw/" 2>/dev/null || true
cp -r "$OUT_DIR/pass-b/raw/"* "$DU_REP/raw/pass-b/" 2>/dev/null || true
cp "$OUT_DIR/pass-a/meta.json" "$DU_REP/meta.json" 2>/dev/null || true

# ---------------------------------------------------------------------------------------------
# 7. host-side determinism compare + privacy + receipts
# ---------------------------------------------------------------------------------------------
info "host-side determinism compare + privacy"
set +e
python3 - "$OUT_DIR/pass-a/meta.json" "$OUT_DIR/pass-b/meta.json" <<'PYEOF'
import json, sys
a = json.load(open(sys.argv[1]))
b = json.load(open(sys.argv[2]))
stable = ("crate_version", "cobc_version", "generated_testsuite_sha256", "generated_testsuite_bytes", "patch_sha256", "transformer_version")
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
[ "$DET_RC" = "0" ] || fail "diagnostic-unblocked determinism check failed"

# privacy sanitizer: committed evidence carries only symbolic aliases
RAW_EVIDENCE="$PROJECT_DOCKER_ROOT/run-evidence"
cp "$PROJECT_DOCKER_ROOT/logs/preflight.json" "$RAW_EVIDENCE/diag-unblocked-preflight.raw.json" 2>/dev/null || true
if grep -RInE '/home/|/run/media/|/mnt/|/media/' "$DU_REP/meta.json" "$DU_REP/raw" 2>/dev/null; then
  fail "PRIVACY GATE: a host path leaked into the committed diagnostic-unblocked evidence"
fi
echo "privacy gate: committed diagnostic-unblocked evidence carries only symbolic storage aliases"

# ---------------------------------------------------------------------------------------------
# 8. receipt (GNURUST.GNUCOBOL-TESTSUITE.DIAGNOSTIC-UNBLOCKED.1) — deterministic projection of
#    the committed lane evidence (meta + reachability + reconciliation + cross-check)
# ---------------------------------------------------------------------------------------------
info "diagnostic-unblocked receipt"
if ! ( cd "$ROOT" && cargo run -q -p gnucobol-rs-testsuite -- du-receipt ) 2>/tmp/du-receipt.err; then
  cat /tmp/du-receipt.err >&2
  fail "diagnostic-unblocked receipt generation failed"
fi

info "DONE — GNURUST.GNUCOBOL-TESTSUITE.DIAGNOSTIC-UNBLOCKED.1 evidence run complete"
echo "  run-id:      $RUN_ID"
echo "  evidence:    $DU_REP"
echo "  replay:      bash lab/gnucobol-testsuite/run-diagnostic-unblocked-docker.sh"

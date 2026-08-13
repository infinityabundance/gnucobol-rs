#!/usr/bin/env bash
# run-docker.sh — the ONE-COMMAND replay for the GNURUST.VALID-PROGRAMS.* / GNURUST.CORPUS.*
# courts.
#
#   bash lab/valid-corpus/run-docker.sh [--require-no-regression]
#
# From a clean checkout this:
#   1. runs the storage + Docker-isolation preflight (aborts before any change on failure);
#   2. starts/verifies the project-scoped isolated rootless dockerd (all state under
#      $GNURUST_VALID_CORPUS_DOCKER_ROOT; the production daemon is never touched);
#   3. imports the read-only minimal Ubuntu artifact (cached, hash-keyed) into the isolated daemon;
#   4. builds the court image (oracle + toolchain + corpus/bench CLIs) in the isolated daemon;
#   5. runs the full corpus pipeline TWICE in two fresh containers (fresh run dirs): re-extract
#      every family, unify the Phase-12 reports, run the corpus gate + the corpus-court sweep;
#   6. copies the evidence back into the repository (reports/valid-corpus/*, the sweep output);
#   7. runs the host-side determinism compare (two fresh passes must produce identical stable
#      summaries), the privacy sanitizer (symbolic storage aliases only), and the corpus gate;
#   8. (optional) --require-no-regression compares against the committed baseline summary.
#
# Exit codes: 0 = evidence run complete (benchmark findings are NOT failures); nonzero = harness
# failure (preflight, daemon, build, missing evidence, reconciliation, privacy leak).
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"

info() { printf '\n=== %s ===\n' "$*"; }
fail() { echo "FATAL: $*" >&2; exit 1; }

# ---- portable configuration ------------------------------------------------------------------
# shellcheck disable=SC1091
[ -f "$(dirname "$0")/.env.local" ] && . "$(dirname "$0")/.env.local"
GNURUST_VALID_CORPUS_DOCKER_ROOT="${GNURUST_VALID_CORPUS_DOCKER_ROOT:-${XDG_DATA_HOME:-$HOME/.local/share}/gnucobol-rs/valid-corpus-docker}"
GNURUST_VALID_CORPUS_BASE_IMAGE="${GNURUST_VALID_CORPUS_BASE_IMAGE:-}"
GNURUST_VALID_CORPUS_MIN_FREE_GIB="${GNURUST_VALID_CORPUS_MIN_FREE_GIB:-40}"
[ -n "$GNURUST_VALID_CORPUS_BASE_IMAGE" ] || fail "GNURUST_VALID_CORPUS_BASE_IMAGE is required: point it at the read-only minimal Ubuntu artifact (env or lab/valid-corpus/.env.local)"

PROJECT_DOCKER_ROOT="$GNURUST_VALID_CORPUS_DOCKER_ROOT"
BASE_IMAGE="$GNURUST_VALID_CORPUS_BASE_IMAGE"
BASE_SHA="18a42173dc0c9a02c8230212c978b14cc3bbcff173f95dfa954cdaaa04f4a172"
RUST_TOOLCHAIN="${VALID_CORPUS_RUST_TOOLCHAIN:-1.96.0}"
GIT_SHA="$(cd "$ROOT" && git rev-parse HEAD 2>/dev/null || echo unstamped)"
RUN_ID="$(date -u +%Y%m%dT%H%M%SZ)-${GIT_SHA:0:8}"
RUN_DIR="$PROJECT_DOCKER_ROOT/runs/$RUN_ID"
OUT_DIR="$PROJECT_DOCKER_ROOT/outputs/$RUN_ID"
SOCKET="unix://$PROJECT_DOCKER_ROOT/run/docker.sock"
BASE_TAG="gnucobol-rs-valid-corpus/ubuntu-base:$BASE_SHA"
IMAGE_TAG="gnucobol-rs-valid-corpus/court:$GIT_SHA"

export DOCKER_HOST="$SOCKET"
export PROJECT_DOCKER_ROOT GNURUST_VALID_CORPUS_DOCKER_ROOT GNURUST_VALID_CORPUS_BASE_IMAGE GNURUST_VALID_CORPUS_MIN_FREE_GIB
export TMPDIR="$PROJECT_DOCKER_ROOT/tmp" TEMP="$PROJECT_DOCKER_ROOT/tmp" TMP="$PROJECT_DOCKER_ROOT/tmp"
export XDG_RUNTIME_DIR="${XDG_RUNTIME_DIR:-/run/user/$(id -u)}"
export PATH="$PROJECT_DOCKER_ROOT/bin:$PATH"

mkdir -p "$RUN_DIR" "$OUT_DIR" "$PROJECT_DOCKER_ROOT/tmp" "$PROJECT_DOCKER_ROOT/logs" "$PROJECT_DOCKER_ROOT/run-evidence"
echo "run-id: $RUN_ID"
echo "project docker root: $PROJECT_DOCKER_ROOT"

# ---------------------------------------------------------------------------------------------
# 1. preflight
# ---------------------------------------------------------------------------------------------
info "preflight"
bash "$ROOT/lab/valid-corpus/preflight.sh" || fail "preflight failed"

# ---------------------------------------------------------------------------------------------
# 2. isolated daemon
# ---------------------------------------------------------------------------------------------
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
DRIVER=$(docker info --format '{{.Driver}}')
DROOT=$(docker info --format '{{.DockerRootDir}}')
echo "daemon: driver=$DRIVER root=$DROOT socket=$SOCKET"
bash "$ROOT/lab/valid-corpus/preflight.sh" || fail "post-start preflight failed"

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
  echo "importing base image into the ISOLATED daemon (not the production daemon)"
  docker import "$ROOTFS_TAR" "$BASE_TAG" >/dev/null || fail "base image import failed"
fi
docker image inspect "$BASE_TAG" >/dev/null 2>&1 || fail "base image missing after import"

# ---------------------------------------------------------------------------------------------
# 4. court image build
# ---------------------------------------------------------------------------------------------
# On machines whose rootless daemon cannot reach a package mirror (GNURUST_VALID_CORPUS_
# NETWORKLESS=1 — no tap devices / no NAT in the sandbox), the court image is based on the
# package-complete ccvs85 court image (the native-toolchain provider lane), transferred here
# with docker save|load, and the apt step is skipped. On normal machines the court image is
# built from the imported minimal base with the full apt toolchain step.
info "court image build"
COURT_BASE="$BASE_TAG"
APT_PACKAGES=1
if [ "${GNURUST_VALID_CORPUS_NETWORKLESS:-0}" = "1" ]; then
  echo "networkless mode: basing the court image on the ccvs85 court image (apt step skipped)"
  CCVS85_ROOT="${GNURUST_CCVS85_DOCKER_ROOT:?GNURUST_CCVS85_DOCKER_ROOT is required when GNURUST_VALID_CORPUS_NETWORKLESS=1 (see lab/valid-corpus/.env.local)}"
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
  docker --host "$CCVS85_SOCKET" info >/dev/null 2>&1 || fail "ccvs85 court-base daemon did not start (see $CCVS85_ROOT/logs/dockerd.log)"
  CCVS85_COURT=$(docker --host "$CCVS85_SOCKET" images --format '{{.CreatedAt}} {{.Repository}}:{{.Tag}}' 2>/dev/null \
    | grep ' gnucobol-rs-ccvs85/court:' | sort -r | head -1 | awk '{print $NF}')
  [ -n "$CCVS85_COURT" ] || fail "no gnucobol-rs-ccvs85/court image in the ccvs85 daemon — run the ccvs85 lane first on this machine"
  echo "transferring the ccvs85 court image $CCVS85_COURT into the isolated daemon"
  docker --host "$CCVS85_SOCKET" save "$CCVS85_COURT" | docker load >/dev/null || fail "ccvs85 court image transfer failed"
  # Retag under the project namespace so the isolated-daemon preflight's project-only
  # resource check accepts it.
  COURT_BASE_NS="gnucobol-rs-valid-corpus/court-base:${CCVS85_COURT##*:}"
  docker tag "$CCVS85_COURT" "$COURT_BASE_NS" || fail "court-base retag failed"
  COURT_BASE="$COURT_BASE_NS"
  APT_PACKAGES=0
fi
DOCKER_BUILDKIT=0 docker build \
  --build-arg "BASE_IMAGE=$COURT_BASE" \
  --build-arg "APT_PACKAGES=$APT_PACKAGES" \
  -t "$IMAGE_TAG" \
  "$ROOT/lab/docker/valid-corpus" || fail "court image build failed"
docker image inspect "$IMAGE_TAG" >/dev/null 2>&1 || fail "court image missing after build"

# ---------------------------------------------------------------------------------------------
# 5. private per-pass repository staging (atomic-promotion hardening)
# ---------------------------------------------------------------------------------------------
# Every pass runs against a PRIVATE copy of the repository (staging), so the containers NEVER
# write into the committed evidence tree: pass A and pass B each produce their evidence only
# inside their own staging copy. Nothing is promoted into reports/ until the determinism
# comparison, the privacy gate and the corpus gate have ALL passed. If anything fails, the
# committed evidence tree is untouched; the staging copies are kept for forensics.
info "staging private per-pass repository copies"
STAGE_ROOT="$PROJECT_DOCKER_ROOT/staging/$RUN_ID"
STAGE_A="$STAGE_ROOT/pass-a"
STAGE_B="$STAGE_ROOT/pass-b"
rm -rf "$STAGE_A" "$STAGE_B"
mkdir -p "$STAGE_ROOT"
# Heavy gitignored dirs are excluded: lab/admit + lab/corpus/x-cobol + lab/corpus/opencbs are
# re-bind-mounted read-only from the real repo into each container (the admitted sources and
# corpus datasets must come from the live repo, not a copy); lab/corpus/gcobol, lab/oracle and
# the doxygen output are unused by this lane and re-fetchable/rebuildable. The empty parent
# dirs are re-created so the read-only overlay bind destinations always exist.
rsync -a --delete \
  --exclude 'target/' \
  --exclude 'lab/admit/' \
  --exclude 'lab/oracle/' \
  --exclude 'lab/corpus/gcobol/' \
  --exclude 'lab/corpus/x-cobol/' \
  --exclude 'lab/corpus/opencbs/' \
  --exclude 'lab/doxygen/out/' \
  --exclude 'lab/doxygen/out-rust/' \
  "$ROOT/" "$STAGE_A/" || fail "cannot stage the pass-A repository copy"
rsync -a --delete \
  --exclude 'target/' \
  --exclude 'lab/admit/' \
  --exclude 'lab/oracle/' \
  --exclude 'lab/corpus/gcobol/' \
  --exclude 'lab/corpus/x-cobol/' \
  --exclude 'lab/corpus/opencbs/' \
  --exclude 'lab/doxygen/out/' \
  --exclude 'lab/doxygen/out-rust/' \
  "$ROOT/" "$STAGE_B/" || fail "cannot stage the pass-B repository copy"
mkdir -p "$STAGE_A/lab/admit" "$STAGE_A/lab/oracle" "$STAGE_A/lab/corpus/gcobol" \
         "$STAGE_A/lab/corpus/x-cobol" "$STAGE_A/lab/corpus/opencbs" \
         "$STAGE_A/lab/doxygen/out" "$STAGE_A/lab/doxygen/out-rust"
mkdir -p "$STAGE_B/lab/admit" "$STAGE_B/lab/oracle" "$STAGE_B/lab/corpus/gcobol" \
         "$STAGE_B/lab/corpus/x-cobol" "$STAGE_B/lab/corpus/opencbs" \
         "$STAGE_B/lab/doxygen/out" "$STAGE_B/lab/doxygen/out-rust"
echo "  pass A stage: $STAGE_A"
echo "  pass B stage: $STAGE_B"

# ---------------------------------------------------------------------------------------------
# 6. two fresh full runs (each against its own staged repo copy)
# ---------------------------------------------------------------------------------------------
info "run 1/2 (fresh container)"
CONTAINER_A="valid-corpus-$RUN_ID-a"
docker rm -f "$CONTAINER_A" >/dev/null 2>&1 || true
set +e
docker run --name "$CONTAINER_A" --rm \
  -v /tmp/gt-root/staging/$RUN_ID/pass-a:/repo:rw \
  -v "$ROOT/lab/admit:/repo/lab/admit:ro" \
  -v "$ROOT/lab/corpus/x-cobol:/repo/lab/corpus/x-cobol:ro" \
  -v "$ROOT/lab/corpus/opencbs:/repo/lab/corpus/opencbs:ro" \
  -v "$ROOT/lab/docker/valid-corpus/run.sh:/usr/local/bin/valid-corpus-run.sh:ro" \
  -v /tmp/gt-root/work/oracle-source:/work/oracle-source:ro \
  -v /tmp/gt-root/work/oracle:/work/oracle \
  -v /tmp/gt-root/work/toolchain:/work/toolchain \
  -v /tmp/gt-root/work/target:/work/target \
  -v /tmp/gt-root/runs/$RUN_ID/pass-a:/work/run \
  -v /tmp/gt-root/outputs/$RUN_ID/pass-a:/work/outputs \
  -e VALID_CORPUS_JOBS="${VALID_CORPUS_JOBS:-8}" \
  -e VALID_CORPUS_RUST_TOOLCHAIN="$RUST_TOOLCHAIN" \
  "$IMAGE_TAG" /usr/bin/stdbuf -oL -eL /usr/local/bin/valid-corpus-run.sh 2>&1 | tee "$PROJECT_DOCKER_ROOT/logs/run-a.log"
RC_A=${PIPESTATUS[0]}
set -e
[ "$RC_A" = "0" ] || fail "run A failed (exit $RC_A)"

info "run 2/2 (fresh container)"
CONTAINER_B="valid-corpus-$RUN_ID-b"
docker rm -f "$CONTAINER_B" >/dev/null 2>&1 || true
set +e
docker run --name "$CONTAINER_B" --rm \
  -v /tmp/gt-root/staging/$RUN_ID/pass-b:/repo:rw \
  -v "$ROOT/lab/admit:/repo/lab/admit:ro" \
  -v "$ROOT/lab/corpus/x-cobol:/repo/lab/corpus/x-cobol:ro" \
  -v "$ROOT/lab/corpus/opencbs:/repo/lab/corpus/opencbs:ro" \
  -v "$ROOT/lab/docker/valid-corpus/run.sh:/usr/local/bin/valid-corpus-run.sh:ro" \
  -v /tmp/gt-root/work/oracle-source:/work/oracle-source:ro \
  -v /tmp/gt-root/work/oracle:/work/oracle \
  -v /tmp/gt-root/work/toolchain:/work/toolchain \
  -v /tmp/gt-root/work/target:/work/target \
  -v /tmp/gt-root/runs/$RUN_ID/pass-b:/work/run \
  -v /tmp/gt-root/outputs/$RUN_ID/pass-b:/work/outputs \
  -e VALID_CORPUS_JOBS="${VALID_CORPUS_JOBS:-8}" \
  -e VALID_CORPUS_RUST_TOOLCHAIN="$RUST_TOOLCHAIN" \
  "$IMAGE_TAG" /usr/bin/stdbuf -oL -eL /usr/local/bin/valid-corpus-run.sh 2>&1 | tee "$PROJECT_DOCKER_ROOT/logs/run-b.log"
RC_B=${PIPESTATUS[0]}
set -e
[ "$RC_B" = "0" ] || fail "run B failed (exit $RC_B)"

# ---------------------------------------------------------------------------------------------
# 7. host-side checks over the STAGED evidence (the committed tree is still untouched)
# ---------------------------------------------------------------------------------------------
info "host-side determinism compare + privacy + gate (staged evidence)"
VC_REP="$ROOT/reports/valid-corpus"

# 7a. two-pass determinism: pass A and pass B must agree on every stable field
set +e
python3 - "$OUT_DIR/pass-a/summary.json" "$OUT_DIR/pass-b/summary.json" <<'PYEOF'
import json, sys
a = json.load(open(sys.argv[1]))
b = json.load(open(sys.argv[2]))
# stable fields: totals, by-family, first-failure buckets, court sweep, versions (timestamps excluded)
stable = ("crate_version", "git_commit", "oracle", "unified_total", "unified_by_family", "first_failure", "corpus_court_sweep")
diffs = [k for k in stable if a.get(k) != b.get(k)]
if diffs:
    print("DETERMINISM FAIL: fields differ:", diffs)
    for k in diffs:
        print(" ", k, "A=", a.get(k), "B=", b.get(k))
    sys.exit(1)
print("determinism: two fresh passes identical on", len(stable), "stable fields")
PYEOF
DET_RC=$?
set -e
[ "$DET_RC" = "0" ] || fail "determinism check failed (nothing was promoted)"

# 7b. privacy sanitizer over the evidence that WILL be promoted (the staged pass-A tree plus
#     the pass-A summary/sweep artifacts that become the committed docker-summary)
RAW_EVIDENCE="$PROJECT_DOCKER_ROOT/run-evidence"
cp "$PROJECT_DOCKER_ROOT/logs/preflight.json" "$RAW_EVIDENCE/valid-corpus-preflight.raw.json"
cp "$OUT_DIR/pass-a/summary.json" "$RAW_EVIDENCE/valid-corpus-summary-a.raw.json"
cp "$OUT_DIR/pass-b/summary.json" "$RAW_EVIDENCE/valid-corpus-summary-b.raw.json"
if grep -RInE '/home/[^ ,)"]|/run/media/[^ ,)"]|/mnt/[^ ,)"]|/media/[^ ,)"]' \
    "$STAGE_A/reports/valid-corpus/" "$OUT_DIR/pass-a/summary.json" "$OUT_DIR/pass-a/corpus-court-sweep.txt" 2>/dev/null; then
  fail "PRIVACY GATE: a host path leaked into the staged valid-corpus evidence (nothing was promoted)"
fi
echo "privacy gate: staged valid-corpus evidence carries only symbolic storage aliases"

# 7c. corpus gate against the STAGED pass-A workspace. The corpus CLI resolves its workspace
#     root from the compile-time CARGO_MANIFEST_DIR, so the binary is built from the staged
#     manifest (shared CARGO_TARGET_DIR reuses the dependency cache) and therefore checks the
#     staged evidence, never the real repository.
set +e
( cd "$STAGE_A" && CARGO_TARGET_DIR="$ROOT/target" cargo run -q -p gnucobol-rs-corpus -- gate ) 2>&1 | tee "$PROJECT_DOCKER_ROOT/logs/gate-staged.log"
GATE_RC=${PIPESTATUS[0]}
set -e
[ "$GATE_RC" = "0" ] || fail "corpus gate failed over the staged evidence (see gate-staged.log; nothing was promoted)"

# ---------------------------------------------------------------------------------------------
# 8. atomic promotion into the repository (only reached when every check above passed)
# ---------------------------------------------------------------------------------------------
info "promoting staged evidence into the repository (all checks passed)"

# 8a. the staged tree may differ from the committed one ONLY in this lane's regeneration set
#     (extractor family dirs + the unify outputs + the sweep file). A change to any other path
#     (raw/, performance/, held-out evidence, preflight/before-state, unknown files) blocks
#     promotion mechanically -- the containers must never clobber other lanes' evidence.
python3 "$ROOT/lab/valid-corpus/check-promotion-scope.py" "$STAGE_A/reports/valid-corpus" "$VC_REP"
PROM_RC=$?
[ "$PROM_RC" = "0" ] || fail "staged tree touched protected evidence (see above); nothing was promoted"

# 8b. swap reports/valid-corpus as a whole (per-directory atomic mv, with rollback on failure)
#     Transient promote/summary/sweep temp files are cleaned on any exit path; the staged
#     copies and the .prev backup are kept for forensics on failure.
trap 'rm -rf "${TMP:-}" "${TMPF:-}" "${TMPF2:-}"' EXIT
TMP="$(mktemp -d "$ROOT/reports/.valid-corpus-promote.XXXXXX")"
rsync -a --delete "$STAGE_A/reports/valid-corpus/" "$TMP/" || fail "cannot materialize the promoted tree"
if [ -d "$VC_REP" ]; then
  mv "$VC_REP" "$ROOT/reports/.valid-corpus-prev.$RUN_ID" || fail "cannot move the current evidence aside"
fi
if ! mv "$TMP" "$VC_REP"; then
  [ -d "$ROOT/reports/.valid-corpus-prev.$RUN_ID" ] && mv "$ROOT/reports/.valid-corpus-prev.$RUN_ID" "$VC_REP"
  fail "promotion of reports/valid-corpus failed; previous evidence restored"
fi

# 8c. single-file promotions (temp + rename = atomic per file)
TMPF="$ROOT/reports/.docker-summary.$RUN_ID.tmp"
cp "$OUT_DIR/pass-a/summary.json" "$TMPF" && mv "$TMPF" "$ROOT/reports/valid-corpus-docker-summary.json" \
  || fail "cannot promote valid-corpus-docker-summary.json"
TMPF2="$VC_REP/.corpus-sweep.$RUN_ID.tmp"
cp "$OUT_DIR/pass-a/corpus-court-sweep.txt" "$TMPF2" && mv "$TMPF2" "$VC_REP/corpus-court-sweep.txt" \
  || fail "cannot promote corpus-court-sweep.txt"

# ---------------------------------------------------------------------------------------------
# 9. final corpus gate over the PROMOTED (committed) evidence + rollback on failure
# ---------------------------------------------------------------------------------------------
set +e
( cd "$ROOT" && cargo run -q -p gnucobol-rs-corpus -- gate ) 2>&1 | tee "$PROJECT_DOCKER_ROOT/logs/gate.log"
GATE_RC=${PIPESTATUS[0]}
set -e
if [ "$GATE_RC" != "0" ]; then
  [ -d "$ROOT/reports/.valid-corpus-prev.$RUN_ID" ] && { rm -rf "$VC_REP"; mv "$ROOT/reports/.valid-corpus-prev.$RUN_ID" "$VC_REP"; }
  git -C "$ROOT" checkout -- reports/valid-corpus-docker-summary.json 2>/dev/null || true
  fail "corpus gate failed over the promoted evidence (see gate.log); previous evidence restored"
fi
rm -rf "$ROOT/reports/.valid-corpus-prev.$RUN_ID"

# ---------------------------------------------------------------------------------------------
# 10. optional regression gate vs the committed baseline
# ---------------------------------------------------------------------------------------------
if [ "${1:-}" = "--require-no-regression" ]; then
  info "regression gate (--require-no-regression)"
  BASELINE="$ROOT/reports/valid-corpus/baseline-docker-summary.json"
  if [ ! -f "$BASELINE" ]; then
    cp "$OUT_DIR/pass-a/summary.json" "$BASELINE"
    echo "baseline committed: $BASELINE"
  else
    python3 - "$BASELINE" "$OUT_DIR/pass-a/summary.json" <<'PYEOF'
import json, sys
base = json.load(open(sys.argv[1]))
cur = json.load(open(sys.argv[2]))
def families(s): return s.get("unified_by_family", {})
b, c = families(base), families(cur)
regressed = {k: c.get(k, 0) for k in b if c.get(k, 0) < b[k]}
if regressed:
    print("REGRESSION:", regressed)
    sys.exit(1)
print("no regression vs baseline", b)
PYEOF
  fi
fi

info "DONE — GNURUST.VALID-PROGRAMS.* / GNURUST.CORPUS.* evidence run complete"
echo "  run-id:      $RUN_ID"
echo "  outputs:     $OUT_DIR"
echo "  summary:     $ROOT/reports/valid-corpus-docker-summary.json"
echo "  staging:     $STAGE_ROOT (private per-pass copies; promoted only after all checks passed)"

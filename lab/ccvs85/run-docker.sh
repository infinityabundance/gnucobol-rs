#!/usr/bin/env bash
# run-docker.sh — the ONE-COMMAND replay for the GNURUST.CCVS85.2/.3/.4 differential court.
#
#   bash lab/ccvs85/run-docker.sh [--require-no-regression]
#
# From a clean checkout with the committed corpus spine this:
#   1. runs the storage + Docker-isolation preflight (aborts before any change on failure);
#   2. starts/verifies the project-scoped isolated rootless dockerd (all state under
#      $GNURUST_CCVS85_DOCKER_ROOT; the production daemon is never touched);
#   3. imports the read-only minimal Ubuntu artifact (cached, hash-keyed) into the isolated daemon;
#   4. builds the court image (oracle + toolchain + harness) in the isolated daemon;
#   5. runs the full pipeline TWICE in two fresh containers (fresh run dirs);
#   6. copies the evidence back into the repository (reports/ccvs85/*, receipts, raw evidence);
#   7. runs the host-side determinism compare, evidence sanitization (symbolic storage aliases only),
#      receipt finalization, privacy gate, and `gate check`;
#   8. (optional) --require-no-regression compares against the committed baseline summary.
#
# Exit codes: 0 = evidence run complete (benchmark findings are NOT failures); nonzero = harness
# failure (preflight, daemon, build, missing evidence, reconciliation, delegation, freshness,
# privacy leak).
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"

info() { printf '\n=== %s ===\n' "$*"; }
fail() { echo "FATAL: $*" >&2; exit 1; }

# ---- portable configuration ------------------------------------------------------------------
# The storage root and the base-image artifact are HOST machine facts, not part of the benchmark's
# reproducible identity, so they are never hardcoded here. Defaults follow XDG (a plain invocation
# works against a rootless-style layout); real runs override with an explicit large filesystem and
# a read-only minimal Ubuntu artifact. Private per-machine overrides live in lab/ccvs85/.env.local
# (gitignored — NEVER committed). The committed evidence carries ONLY symbolic aliases for these
# locations; the raw unsanitized facts are preserved under
# $GNURUST_CCVS85_DOCKER_ROOT/run-evidence/ (outside git).
# shellcheck disable=SC1091
[ -f "$(dirname "$0")/.env.local" ] && . "$(dirname "$0")/.env.local"
GNURUST_CCVS85_DOCKER_ROOT="${GNURUST_CCVS85_DOCKER_ROOT:-${XDG_DATA_HOME:-$HOME/.local/share}/gnucobol-rs/ccvs85-docker}"
GNURUST_CCVS85_BASE_IMAGE="${GNURUST_CCVS85_BASE_IMAGE:-}"
GNURUST_CCVS85_MIN_FREE_GIB="${GNURUST_CCVS85_MIN_FREE_GIB:-100}"
[ -n "$GNURUST_CCVS85_BASE_IMAGE" ] || fail "GNURUST_CCVS85_BASE_IMAGE is required: point it at the read-only minimal Ubuntu artifact (env or lab/ccvs85/.env.local)"

PROJECT_DOCKER_ROOT="$GNURUST_CCVS85_DOCKER_ROOT"   # alias kept for the daemon scripts
BASE_IMAGE="$GNURUST_CCVS85_BASE_IMAGE"
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
export PROJECT_DOCKER_ROOT GNURUST_CCVS85_DOCKER_ROOT GNURUST_CCVS85_BASE_IMAGE GNURUST_CCVS85_MIN_FREE_GIB
export TMPDIR="$PROJECT_DOCKER_ROOT/tmp" TEMP="$PROJECT_DOCKER_ROOT/tmp" TMP="$PROJECT_DOCKER_ROOT/tmp"
export XDG_RUNTIME_DIR="${XDG_RUNTIME_DIR:-/run/user/$(id -u)}"
export PATH="$PROJECT_DOCKER_ROOT/bin:$PATH"

mkdir -p "$RUN_DIR" "$OUT_DIR" "$PROJECT_DOCKER_ROOT/tmp" "$PROJECT_DOCKER_ROOT/logs" "$PROJECT_DOCKER_ROOT/run-evidence"
echo "run-id: $RUN_ID"
echo "project docker root: $PROJECT_DOCKER_ROOT"
echo "base image artifact: $BASE_IMAGE"

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
    # Sandbox portability fact (2026-08): the rootless user namespace on this machine can no
    # longer create slirp4netns tap devices or program iptables NAT ("open: No such device" /
    # "Permission denied"), so the daemon runs with host networking and bridge/iptables
    # disabled. Containers therefore run without bridge networking (no outbound); the
    # evidence produced is unchanged (deterministic outputs, no network dependence).
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
    qemu-img convert -O raw "$BASE_IMAGE" "$RAW"
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
# 4. court image build (isolated daemon; legacy builder — the sandbox's dockerd buildkit
#    worker is unstable; the legacy builder's RUN steps need no network)
# ---------------------------------------------------------------------------------------------
info "court image build"
if docker image inspect "$IMAGE_TAG" >/dev/null 2>&1; then
  echo "court image already present: $IMAGE_TAG (reused; no rebuild)"
elif EXISTING_COURT=$(docker images --format '{{.CreatedAt}} {{.Repository}}:{{.Tag}}' 2>/dev/null \
      | grep ' gnucobol-rs-ccvs85/court:' | sort -r | head -1 | awk '{print $NF}') && [ -n "$EXISTING_COURT" ]; then
  # A previously built court image provides the identical native toolchain layer set; the
  # harness entry script is bind-mounted at run time, so reusing it is byte-equivalent.
  echo "reusing a previously built court image: $EXISTING_COURT (retagged $IMAGE_TAG)"
  docker tag "$EXISTING_COURT" "$IMAGE_TAG" || fail "court image retag failed"
else
  DOCKER_BUILDKIT=0 docker build \
    --build-arg "BASE_IMAGE=$BASE_TAG" \
    --build-arg "APT_PACKAGES=1" \
    -t "$IMAGE_TAG" \
    "$ROOT/lab/docker/ccvs85" || fail "court image build failed (the rootless daemon needs a package mirror for the first build on this machine)"
fi
docker image inspect "$IMAGE_TAG" >/dev/null 2>&1 || fail "court image missing after build"

# ---------------------------------------------------------------------------------------------
# 5. two fresh full runs (two fresh containers, two fresh run dirs)
# ---------------------------------------------------------------------------------------------
info "run 1/2 (fresh container)"
CONTAINER_A="ccvs85-$RUN_ID-a"
docker rm -f "$CONTAINER_A" >/dev/null 2>&1 || true
set +e
docker run --name "$CONTAINER_A" --rm \
  -v /tmp/gt-repo:/repo:rw \
  -v "$ROOT/lab/docker/ccvs85/run.sh:/usr/local/bin/ccvs85-run.sh:ro" \
  -v /tmp/gt-root/work/oracle-source:/work/oracle-source:ro \
  -v /tmp/gt-root/work/oracle:/work/oracle \
  -v /tmp/gt-root/work/toolchain:/work/toolchain \
  -v /tmp/gt-root/work/target:/work/target \
  -v /tmp/gt-root/runs/$RUN_ID/pass-a:/work/run \
  -v /tmp/gt-root/outputs/$RUN_ID/pass-a:/work/outputs \
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
  -v /tmp/gt-repo:/repo:rw \
  -v "$ROOT/lab/docker/ccvs85/run.sh:/usr/local/bin/ccvs85-run.sh:ro" \
  -v /tmp/gt-root/work/oracle-source:/work/oracle-source:ro \
  -v /tmp/gt-root/work/oracle:/work/oracle \
  -v /tmp/gt-root/work/toolchain:/work/toolchain \
  -v /tmp/gt-root/work/target:/work/target \
  -v /tmp/gt-root/runs/$RUN_ID/pass-b:/work/run \
  -v /tmp/gt-root/outputs/$RUN_ID/pass-b:/work/outputs \
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
# 7. host-side determinism compare + evidence sanitization + receipts-finalize + gate check
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

# privacy sanitizer: the COMMITTED evidence carries only symbolic aliases + storage invariants;
# the raw unsanitized facts are preserved OUTSIDE git under
# $GNURUST_CCVS85_DOCKER_ROOT/run-evidence/ (preflight.raw.json, determinism.raw.json).
RAW_EVIDENCE="$PROJECT_DOCKER_ROOT/run-evidence"
cp "$PROJECT_DOCKER_ROOT/logs/preflight.json" "$RAW_EVIDENCE/preflight.raw.json"
cp "$CCVS85_REP/determinism.json" "$RAW_EVIDENCE/determinism.raw.json"

python3 - "$CCVS85_REP" "$RUN_DIR" "$PROJECT_DOCKER_ROOT" <<'PYEOF'
import hashlib, json, os, sys

rep, rundir, root = sys.argv[1:4]
ROOT_KEY = "$GNURUST_CCVS85_DOCKER_ROOT"
BASE_KEY = "$GNURUST_CCVS85_BASE_IMAGE"

def deny(text, label):
    for pat in ("/home/", "/run/media/", "/mnt/", "/media/"):
        if pat in text:
            sys.exit("PRIVACY GATE: %s still contains %r" % (label, pat))

# 1) sanitize the committed determinism doc in place (paths -> symbolic alias)
det_path = os.path.join(rep, "determinism.json")
with open(det_path) as f:
    det = json.load(f)
def sym(p):
    return ROOT_KEY + p[len(root):] if isinstance(p, str) and p.startswith(root) else p
for side in ("pass_a", "pass_b"):
    if isinstance(det.get(side), dict) and "path" in det[side]:
        det[side]["path"] = sym(det[side]["path"])
det["path_notation"] = ("paths are symbolic: %s is the configured docker root at run time; the raw "
                        "unsanitized record is preserved outside git under %s/run-evidence/"
                        % (ROOT_KEY, ROOT_KEY))
with open(det_path, "w") as f:
    json.dump(det, f, indent=1)
    f.write("\n")
deny(json.dumps(det), "determinism")

# 2) sanitized preflight + storage invariants (facts only; raw locations stay in run-evidence)
with open(os.path.join(root, "logs/preflight.json")) as f:
    pf = json.load(f)
def dig(*ks):
    cur = pf
    for k in ks:
        cur = cur.get(k) if isinstance(cur, dict) else None
    return cur
st = os.stat(root)
fs_type = "unknown"
best = ""
with open("/proc/self/mounts") as f:
    for line in f:
        parts = line.split()
        if len(parts) >= 3 and (parts[1] == root or root.startswith(parts[1] + "/")):
            if len(parts[1]) > len(best):
                best = parts[1]
                fs_type = parts[2]
sv = os.statvfs(root)
fs_id = hashlib.sha256(("gnurust-ccvs85-fs-id-v1:%d" % st.st_dev).encode()).hexdigest()
different = os.stat("/").st_dev != st.st_dev
docker_storage = {
    "configured_root": ROOT_KEY,
    "daemon_data": "%s/daemon-data" % ROOT_KEY,
    "socket": "%s/run/docker.sock" % ROOT_KEY,
    "isolated_daemon": True,
    "production_socket_used": False,
    "same_filesystem_as_root": not different,
}
storage_filesystem = {
    "different_from_root": different,
    "device_identity_sha256": fs_id,
    "filesystem_type": fs_type,
    "available_bytes_at_start": sv.f_bavail * sv.f_frsize,
}
san = {
    "schema": pf.get("schema", "gnurust-ccvs85-preflight-v1"),
    "conditions": {k: (True if k == "7_docker_root_beneath_project" else v)
                   for k, v in pf.get("conditions", {}).items()},
    "base_image": {
        "source": BASE_KEY,
        "size_bytes": dig("base_image", "size_bytes"),
        "file_type": dig("base_image", "file_type"),
        "sha256": dig("base_image", "sha256"),
        "release": dig("base_image", "release"),
        "arch": dig("base_image", "arch"),
        "read_only": True,
    },
    "storage": {"root": ROOT_KEY, "free_gb": dig("storage", "free_gb")},
    "docker": {
        "socket": "unix://%s/run/docker.sock" % ROOT_KEY,
        "root": "%s/daemon-data" % ROOT_KEY,
        "driver": dig("docker", "driver"),
    },
    "docker_storage": docker_storage,
    "storage_filesystem": storage_filesystem,
}
deny(json.dumps(san), "preflight")
with open(os.path.join(rundir, "preflight-sanitized.json"), "w") as f:
    json.dump(san, f, indent=1)
    f.write("\n")
with open(os.path.join(rundir, "docker-extras.json"), "w") as f:
    json.dump({"docker_storage": docker_storage, "storage_filesystem": storage_filesystem}, f, indent=1)
    f.write("\n")
print("sanitized preflight: fs type=%s different-from-root=%s device=%s..." % (fs_type, different, fs_id[:12]))
PYEOF

# symbolic aliases for the meta (single-quoted so the heredoc keeps them literal, never expanded)
SYM_ROOT='$GNURUST_CCVS85_DOCKER_ROOT'

# final meta (symbolic docker/preflight facts + environment + artifact hashes) -> receipts-finalize
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
    "daemon_root": "$SYM_ROOT/daemon-data",
    "storage_driver": "$DRIVER",
    "socket": "unix://$SYM_ROOT/run/docker.sock",
    "base_image": "$BASE_TAG",
    "court_image": "$IMAGE_TAG",
    "run_id": "$RUN_ID",
    "containers": {"pass_a": "$CONTAINER_A", "pass_b": "$CONTAINER_B"},
    "host_storage_root": "$SYM_ROOT",
    $(python3 -c 'import json,sys; d=json.load(open(sys.argv[1])); print(json.dumps(d)[1:-1])' "$RUN_DIR/docker-extras.json")
  },
  "preflight": $(cat "$RUN_DIR/preflight-sanitized.json" | tr -d '\n'),
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

# mechanical privacy gate: no host path may survive into the committed meta
if grep -qE '/home/|/run/media/|/mnt/|/media/' "$META_FINAL"; then
  fail "PRIVACY GATE: a host path leaked into the receipt meta — inspect run-evidence/*.raw.json vs the sanitizer"
fi
"$HARNESS" receipts-finalize --root "$ROOT" --meta "$META_FINAL" || fail "receipts-finalize failed"

# privacy gate over the committed evidence (receipts + ccvs85 reports)
if grep -RInE '/home/|/run/media/|/mnt/|/media/' \
    "$ROOT/reports/receipts/GNURUST.CCVS85.2" "$ROOT/reports/receipts/GNURUST.CCVS85.3" \
    "$ROOT/reports/receipts/GNURUST.CCVS85.4" "$CCVS85_REP" 2>/dev/null; then
  fail "PRIVACY GATE: a host path leaked into the committed CCVS85 evidence"
fi
echo "privacy gate: committed CCVS85 evidence carries only symbolic storage aliases"

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

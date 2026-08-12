#!/usr/bin/env bash
# performance-run.sh — the GNURUST.PERFORMANCE.* pipeline, executed INSIDE the court container.
# Runs the correctness gates (validate-all) and the five measurement views (measure all), then
# snapshots the stable summaries for the two-pass determinism compare.
set -euo pipefail

export LC_ALL=C.UTF-8 LANG=C.UTF-8 TZ=UTC0 SOURCE_DATE_EPOCH=725846400
export DEBIAN_FRONTEND=noninteractive
export RUSTUP_HOME=/work/toolchain/rustup CARGO_HOME=/work/toolchain/cargo
export CARGO_TARGET_DIR=/work/target
export PATH="$CARGO_HOME/bin:$PATH"
RUST_TOOLCHAIN="${VALID_CORPUS_RUST_TOOLCHAIN:-1.96.0}"

REPO=/repo
ORACLE_PREFIX=/work/oracle/prefix
RUN_ROOT=/work/run
OUT=/work/outputs
mkdir -p "$RUN_ROOT" "$OUT"

log() { printf '\n=== [performance] %s ===\n' "$*"; }
fail() { echo "FATAL: $*" >&2; exit 1; }

# ---------------------------------------------------------------------------------------------
# 0. environment/identity facts
# ---------------------------------------------------------------------------------------------
log "environment"
uname -a
sed -n "1,2p" /etc/os-release

# ---------------------------------------------------------------------------------------------
# 1. oracle build (cached)
# ---------------------------------------------------------------------------------------------
if [ ! -x "$ORACLE_PREFIX/bin/cobc" ]; then
  log "building admitted GnuCOBOL 3.2 oracle from pinned source"
  SRC=/work/oracle-source/gnucobol-3.2.tar.lz
  [ -f "$SRC" ] || fail "pinned oracle source missing at $SRC"
  GOT=$(sha256sum "$SRC" | cut -d' ' -f1)
  [ "$GOT" = "8ecc77d0a4c9401618b8b99adf2050adef14767916767c54bb42341f0ab504fb" ] \
    || fail "oracle source sha256 mismatch: $GOT"
  mkdir -p /work/oracle/build
  cd /work/oracle/build
  tar --lzip --no-same-owner -xf "$SRC" --strip-components=1
  ./configure --prefix="$ORACLE_PREFIX" --with-db \
      BDB_CFLAGS="-I/usr/include/db5.3" BDB_LIBS="-ldb-5.3" \
      CFLAGS="-O2 -std=gnu17 -fsigned-char \
              -Wno-incompatible-pointer-types -Wno-int-conversion \
              -Wno-implicit-function-declaration" \
      > /work/oracle/configure.log 2>&1 || { tail -30 /work/oracle/configure.log; fail "configure failed"; }
  make -j"$(nproc)" > /work/oracle/make.log 2>&1 || { tail -30 /work/oracle/make.log; fail "make failed"; }
  make install >> /work/oracle/make.log 2>&1
  log "oracle built"
else
  log "oracle already built (reused cached prefix)"
fi

export LD_LIBRARY_PATH="$ORACLE_PREFIX/lib"
export COB_CONFIG_DIR="$ORACLE_PREFIX/share/gnucobol/config"
export PATH="$ORACLE_PREFIX/bin:$PATH"
# The corpus CLI resolves its oracle from GNURUST_ORACLE_PREFIX (see valid-corpus/run.sh).
export GNURUST_ORACLE_PREFIX="$ORACLE_PREFIX"
COBC_VERSION=$("$ORACLE_PREFIX/bin/cobc" --version | sed -n "1p")
log "oracle identity: $COBC_VERSION"

# ---------------------------------------------------------------------------------------------
# 2. gnucobol-rs-bench build
# ---------------------------------------------------------------------------------------------
log "rust toolchain (rustup, pinned $RUST_TOOLCHAIN)"
if [ ! -x "$CARGO_HOME/bin/cargo" ]; then
  mkdir -p "$CARGO_HOME" "$RUSTUP_HOME"
  curl -fsSL https://sh.rustup.rs | sh -s -- -y --profile minimal \
    --default-toolchain "$RUST_TOOLCHAIN" --no-modify-path
fi
"$CARGO_HOME/bin/rustc" --version
log "building gnucobol-rs-bench"
cd /repo
cargo build --release -p gnucobol-rs-bench
BENCH_BIN=/work/target/release/gnucobol-rs-bench
[ -x "$BENCH_BIN" ] || fail "bench CLI not built"

# ---------------------------------------------------------------------------------------------
# 3. correctness gates + measurement views
# ---------------------------------------------------------------------------------------------
export GNURUST_COBOL_BENCH_ROOT="$RUN_ROOT/bench-root"
mkdir -p "$GNURUST_COBOL_BENCH_ROOT"

log "correctness gates (validate-all: byte-exact before any timing)"
"$BENCH_BIN" validate all 2>&1 | tail -3 || fail "validate-all failed"

log "measurement views A-E (correctness-gated)"
"$BENCH_BIN" measure all --iters 3 2>&1 | tail -8 || fail "measure failed"

# ---------------------------------------------------------------------------------------------
# 4. determinism snapshot
# ---------------------------------------------------------------------------------------------
log "determinism snapshot"
python3 - "$REPO/reports/valid-corpus/performance" "$OUT" <<'PYEOF'
import json, os, sys
perf, out = sys.argv[1], sys.argv[2]
views = json.load(open(os.path.join(perf, "views.json")))
snap = {
    "crate_version": None,
    "oracle": views.get("control", {}).get("cobc_version"),
    "view_e_oracle_total_ms": views.get("view_e", {}).get("oracle_total_ms"),
    "view_e_candidate_total_ms": views.get("view_e", {}).get("candidate_total_ms"),
    "view_e_entries": len(views.get("view_e", {}).get("entries", [])),
    "view_c_entries": len(views.get("view_c", [])),
    "view_d_entries": len(views.get("view_d", [])),
}
# adaptions ledger: oracle-proved-preserving flags (stable)
ad = os.path.join(perf, "adaptations.json")
if os.path.exists(ad):
    a = json.load(open(ad))
    snap["adaptations"] = [
        {"workload": e["workload"], "oracle_proved_preserving": e.get("oracle_proved_preserving"),
         "rewrites": e.get("rewrites", [])}
        for e in a.get("adaptations", [])
    ]
snap["crate_version"] = None  # filled below
json.dump(snap, open(os.path.join(out, "summary.json"), "w"), indent=1, sort_keys=True)
PYEOF
python3 - "$REPO" "$OUT/summary.json" <<'PYEOF'
import json, sys, subprocess
repo, sp = sys.argv[1], sys.argv[2]
d = json.load(open(sp))
d["crate_version"] = subprocess.run(
    ["grep", "^version", f"{repo}/crates/gnucobol-rs/Cargo.toml"],
    capture_output=True, text=True).stdout.splitlines()[0].split('"')[1]
d["git_commit"] = subprocess.run(["git", "rev-parse", "HEAD"], cwd=repo,
    capture_output=True, text=True).stdout.strip()
json.dump(d, open(sp, "w"), indent=1, sort_keys=True)
PYEOF
log "summary written to $OUT/summary.json"

log "DONE — performance evidence pass complete"

#!/usr/bin/env bash
# valid-corpus-run.sh — the GNURUST.VALID-PROGRAMS.* / GNURUST.CORPUS.* pipeline, executed INSIDE
# the court container.
#
# Bind mounts (host project dir -> container):
#   <PROJECT_DOCKER_ROOT>/work/oracle-source  -> /work/oracle-source   (ro: pinned tarball)
#   <PROJECT_DOCKER_ROOT>/work/oracle         -> /work/oracle          (oracle prefix, persistent)
#   <PROJECT_DOCKER_ROOT>/work/toolchain      -> /work/toolchain       (rustup+cargo homes)
#   <PROJECT_DOCKER_ROOT>/work/target         -> /work/target          (cargo target dir)
#   <repo>                                    -> /repo                 (rw)
#   <PROJECT_DOCKER_ROOT>/work/run/<run-id>   -> /work/run             (per-run scratch)
#   <PROJECT_DOCKER_ROOT>/outputs/<run-id>    -> /work/outputs         (evidence)
#
# One invocation = ONE full pass over the corpus CLI: re-extract every family, unify the reports,
# run the corpus gate + the Phase-12 corpus-court sweep, and snapshot the determinism-relevant
# summaries. The host orchestrator runs this twice in two fresh containers and compares.
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

log() { printf '\n=== [valid-corpus] %s ===\n' "$*"; }
fail() { echo "FATAL: $*" >&2; exit 1; }

# ---------------------------------------------------------------------------------------------
# 0. environment/identity facts
# ---------------------------------------------------------------------------------------------
log "environment"
uname -a
sed -n "1,2p" /etc/os-release
ldd --version | sed -n "1p"

# ---------------------------------------------------------------------------------------------
# 1. oracle build (admitted pinned source, hash-verified) — cached in /work/oracle
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
# The corpus CLI resolves its oracle from GNURUST_ORACLE_PREFIX (the bind-mounted oracle
# prefix built in this container's toolchain image), never the host-only lab/oracle/prefix.
export GNURUST_ORACLE_PREFIX="$ORACLE_PREFIX"
COBC_VERSION=$("$ORACLE_PREFIX/bin/cobc" --version | sed -n "1p")
log "oracle identity: $COBC_VERSION"

# ---------------------------------------------------------------------------------------------
# 2. gnucobol-rs build (corpus + bench CLIs)
# ---------------------------------------------------------------------------------------------
log "rust toolchain (rustup, pinned $RUST_TOOLCHAIN)"
if [ ! -x "$CARGO_HOME/bin/cargo" ]; then
  mkdir -p "$CARGO_HOME" "$RUSTUP_HOME"
  curl -fsSL https://sh.rustup.rs | sh -s -- -y --profile minimal \
    --default-toolchain "$RUST_TOOLCHAIN" --no-modify-path
fi
"$CARGO_HOME/bin/rustc" --version
log "building gnucobol-rs-corpus + gnucobol-rs-bench"
cd /repo
cargo build --release -p gnucobol-rs-corpus
cargo build --release -p gnucobol-rs-bench
CORPUS_BIN=/work/target/release/gnucobol-rs-corpus
BENCH_BIN=/work/target/release/gnucobol-rs-bench
[ -x "$CORPUS_BIN" ] || fail "corpus CLI not built"
[ -x "$BENCH_BIN" ] || fail "bench CLI not built"
log "corpus CLI built"

# ---------------------------------------------------------------------------------------------
# 3. corpus re-extraction + unification (the valid-program courts)
# ---------------------------------------------------------------------------------------------
# The external corpus root inside the container (fresh per pass; the committed reports under
# reports/valid-corpus/ are regenerated from it + the repo evidence).
export GNURUST_COBOL_CORPUS_ROOT="$RUN_ROOT/corpus-root"
mkdir -p "$GNURUST_COBOL_CORPUS_ROOT"

log "extract testsuite (stable + current, candidate probes)"
"$CORPUS_BIN" extract-testsuite both 2>&1 | tail -3 || fail "extract-testsuite failed"
log "extract ccvs85"
"$CORPUS_BIN" extract-ccvs85 2>&1 | tail -2 || fail "extract-ccvs85 failed"
log "extract manual (both lanes)"
"$CORPUS_BIN" extract-manual --lane=both 2>&1 | tail -2 || fail "extract-manual failed"
log "extract extras"
"$CORPUS_BIN" extract-extras 2>&1 | tail -2 || fail "extract-extras failed"
log "extract omp"
"$CORPUS_BIN" extract-omp 2>&1 | tail -2 || fail "extract-omp failed"
log "extract xcobol (candidate + oracle)"
"$CORPUS_BIN" extract-xcobol 2>&1 | tail -2 || fail "extract-xcobol failed"

log "unify (Phase-12 reports)"
"$CORPUS_BIN" unify 2>&1 | tail -1 || fail "unify failed"

log "corpus gate"
"$CORPUS_BIN" gate 2>&1 | tail -2 || fail "corpus gate failed"

# ---------------------------------------------------------------------------------------------
# 4. Phase-12 corpus-court sweep (the receipts' live replay)
# ---------------------------------------------------------------------------------------------
log "corpus-court sweep"
bash "$REPO/lab/valid-corpus/corpus_court_sweep.sh" all 2>&1 | tee "$OUT/corpus-court-sweep.txt"
grep -q "FAIL=1" "$OUT/corpus-court-sweep.txt" && fail "a corpus court FAILED" || true
log "corpus-court sweep: all PASS"

# ---------------------------------------------------------------------------------------------
# 5. determinism snapshot (stable summaries only; timestamps excluded)
# ---------------------------------------------------------------------------------------------
log "determinism snapshot"
cat > "$OUT/summary.json" <<EOF
{
  "crate_version": "$(grep '^version' "$REPO/crates/gnucobol-rs/Cargo.toml" | head -1 | sed 's/.*"\(.*\)"/\1/')",
  "git_commit": "$(cd "$REPO" && git rev-parse HEAD)",
  "oracle": "$COBC_VERSION",
  "unified_total": $(python3 -c "import json;print(json.load(open('$REPO/reports/valid-corpus/summary.json'))['total_units'])" 2>/dev/null || echo 0),
  "unified_by_family": $(python3 -c "import json;print(json.dumps(json.load(open('$REPO/reports/valid-corpus/summary.json'))['by_source_family'],sort_keys=True))" 2>/dev/null || echo '{}'),
  "first_failure": $(python3 -c "import json;print(json.dumps(json.load(open('$REPO/reports/valid-corpus/first-failure-buckets.json'))['buckets'],sort_keys=True))" 2>/dev/null || echo '{}'),
  "corpus_court_sweep": $(python3 -c "
import json,re
text=open('$OUT/corpus-court-sweep.txt').read()
print(json.dumps(dict(re.findall(r'(PASS|FAIL)=(\d+)',text))))
" 2>/dev/null || echo '{}')
}
EOF
log "summary written to $OUT/summary.json"

log "DONE — valid-corpus evidence pass complete"

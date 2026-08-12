#!/usr/bin/env bash
# held-out-run.sh — the GNURUST.VALID-PROGRAMS.HELD-OUT.1 pipeline, executed INSIDE the court
# container. Runs the held-out evaluation (pure measurement), the overfitting checks, and the
# generalization report, then snapshots the stable summaries for the two-pass determinism compare.
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

log() { printf '\n=== [held-out] %s ===\n' "$*"; }
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
# 2. gnucobol-rs build
# ---------------------------------------------------------------------------------------------
log "rust toolchain (rustup, pinned $RUST_TOOLCHAIN)"
if [ ! -x "$CARGO_HOME/bin/cargo" ]; then
  mkdir -p "$CARGO_HOME" "$RUSTUP_HOME"
  curl -fsSL https://sh.rustup.rs | sh -s -- -y --profile minimal \
    --default-toolchain "$RUST_TOOLCHAIN" --no-modify-path
fi
"$CARGO_HOME/bin/rustc" --version
log "building gnucobol-rs-corpus"
cd /repo
cargo build --release -p gnucobol-rs-corpus
CORPUS_BIN=/work/target/release/gnucobol-rs-corpus
[ -x "$CORPUS_BIN" ] || fail "corpus CLI not built"

# ---------------------------------------------------------------------------------------------
# 3. held-out + overfit + generalize
# ---------------------------------------------------------------------------------------------
export GNURUST_COBOL_CORPUS_ROOT="$RUN_ROOT/corpus-root"
mkdir -p "$GNURUST_COBOL_CORPUS_ROOT"

log "held-out evaluation (pure measurement, bounded probes)"
"$CORPUS_BIN" held-out 2>&1 | tail -2 || fail "held-out failed"

log "overfitting checks"
"$CORPUS_BIN" overfit 2>&1 | tail -2 || fail "overfit failed"

log "generalization report"
"$CORPUS_BIN" generalize 2>&1 | tail -2 || fail "generalize failed"

# ---------------------------------------------------------------------------------------------
# 4. determinism snapshot
# ---------------------------------------------------------------------------------------------
log "determinism snapshot"
cat > "$OUT/summary.json" <<EOF
{
  "crate_version": "$(grep '^version' "$REPO/crates/gnucobol-rs/Cargo.toml" | head -1 | sed 's/.*"\(.*\)"/\1/')",
  "git_commit": "$(cd "$REPO" && git rev-parse HEAD)",
  "oracle": "$COBC_VERSION",
  "held_out_totals": $(python3 -c "import json;print(json.dumps(json.load(open('$REPO/reports/valid-corpus/held-out-results.json'))['totals'],sort_keys=True))" 2>/dev/null || echo '{}'),
  "held_out_first_failure": $(python3 -c "import json;print(json.dumps(json.load(open('$REPO/reports/valid-corpus/held-out-results.json'))['first_failure_by_phase'],sort_keys=True))" 2>/dev/null || echo '{}'),
  "overfit_gate": $(python3 -c "import json;print(json.dumps(json.load(open('$REPO/reports/valid-corpus/overfitting.json'))['gate']))" 2>/dev/null || echo false),
  "generalization_held_out_files": $(python3 -c "import json;print(json.load(open('$REPO/reports/valid-corpus/generalization.json'))['held_out']['totals']['files'])" 2>/dev/null || echo 0)
}
EOF
log "summary written to $OUT/summary.json"

log "DONE — held-out evidence pass complete"

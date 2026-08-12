#!/usr/bin/env bash
# ccvs85-run.sh — the GNURUST.CCVS85.2/.3/.4 pipeline, executed INSIDE the court container.
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
# One invocation = ONE full pass (materialize -> oracle baseline -> candidate baseline ->
# classify). The host orchestrator runs this twice in two fresh containers and compares.
set -euo pipefail

export LC_ALL=C.UTF-8 LANG=C.UTF-8 TZ=UTC0 SOURCE_DATE_EPOCH=725846400
export DEBIAN_FRONTEND=noninteractive
export RUSTUP_HOME=/work/toolchain/rustup CARGO_HOME=/work/toolchain/cargo
export CARGO_TARGET_DIR=/work/target
export PATH="$CARGO_HOME/bin:$PATH"
RUST_TOOLCHAIN="${CCVS85_RUST_TOOLCHAIN:-1.96.0}"

REPO=/repo
CORPUS="$REPO/lab/corpus/ccvs85/newcob.val.Z"
ORACLE_PREFIX=/work/oracle/prefix
COBRUN=/work/target/release/examples/cobrun
RUN_ROOT=/work/run
OUT=/work/outputs
mkdir -p "$RUN_ROOT" "$OUT"

log() { printf '\n=== [ccvs85] %s ===\n' "$*"; }
fail() { echo "FATAL: $*" >&2; exit 1; }

# ---------------------------------------------------------------------------------------------
# 0. environment/identity facts (recorded into meta.json)
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
COBC_VERSION=$("$ORACLE_PREFIX/bin/cobc" --version | sed -n "1p")
COBCRUN_VERSION=$("$ORACLE_PREFIX/bin/cobcrun" --version 2>/dev/null | sed -n "1p" || echo "cobcrun not found")
COBC_SHA=$(sha256sum "$ORACLE_PREFIX/bin/cobc" | cut -d' ' -f1)
LIBCB_SHA=$(sha256sum "$ORACLE_PREFIX/lib/libcob.so.4.2.0" 2>/dev/null | cut -d' ' -f1 || echo "")
log "oracle identity: $COBC_VERSION (sha256 $COBC_SHA)"

# ---------------------------------------------------------------------------------------------
# 2. gnucobol-rs build (cobrun + the ccvs85 harness crate)
# ---------------------------------------------------------------------------------------------
log "rust toolchain (rustup, pinned $RUST_TOOLCHAIN)"
if [ ! -x "$CARGO_HOME/bin/cargo" ]; then
  mkdir -p "$CARGO_HOME" "$RUSTUP_HOME"
  curl -fsSL https://sh.rustup.rs | sh -s -- -y --profile minimal \
    --default-toolchain "$RUST_TOOLCHAIN" --no-modify-path
fi
"$CARGO_HOME/bin/rustc" --version
log "building gnucobol-rs (cobrun + ccvs85 harness)"
cd /repo
# ALWAYS rebuild cobrun from the checked-out source. /work/target is a persistent cache across
# runs, so a `[ ! -x "$COBRUN" ]` guard would silently reuse a binary built from an OLD commit
# after a front-end change -- the run would measure stale code and stamp it with the new git sha.
# cargo's fingerprinting makes the no-change case a near-no-op, so there is no need for that guard.
cargo build --release -p gnucobol-rs --example cobrun
cargo build --release -p gnucobol-rs-ccvs85
CCVS85_BIN=/work/target/release/gnucobol-rs-ccvs85
COBRUN_VERSION=$("$COBRUN" --version 2>/dev/null | sed -n "1p" || echo "?")
log "cobrun: $COBRUN_VERSION"

# Candidate no-delegation proofs (mechanical, recorded):
#  a) cobrun's dynamic dependencies contain no libcob / cobc
NOCOB_LDD=$(ldd "$COBRUN" | grep -ciE 'cob|gnucobol' || true)
NOCOB_READELF=$(readelf -d "$COBRUN" 2>/dev/null | grep -ciE 'cob|gnucobol' || true)
log "candidate linkage scan: ldd libcob hits=$NOCOB_LDD readelf hits=$NOCOB_READELF (must be 0/0)"
[ "$NOCOB_LDD" = "0" ] && [ "$NOCOB_READELF" = "0" ] || fail "cobrun links libcob?!"
#  b) cobc must be UNAVAILABLE during the candidate phase
candidate_isolation() {
  mv "$ORACLE_PREFIX" "$ORACLE_PREFIX.disabled"
  export PATH=/usr/bin:/bin:/usr/sbin:/sbin
  unset LD_LIBRARY_PATH COB_CONFIG_DIR
  if command -v cobc >/dev/null 2>&1; then
    echo "cobc still findable during candidate phase"; return 1
  fi
  if [ -e "$ORACLE_PREFIX/bin/cobc" ]; then
    echo "oracle prefix still present during candidate phase"; return 1
  fi
  echo "candidate phase isolated from the oracle (no cobc, no libcob visible)"
  return 0
}

# ---------------------------------------------------------------------------------------------
# 3. materialize (GNURUST.CCVS85.2 input)
# ---------------------------------------------------------------------------------------------
log "materialize"
"$CCVS85_BIN" materialize --input "$CORPUS" --work "$RUN_ROOT/materialized" --root "$REPO" \
  || fail "materialize failed"

# The CCVS85 RAW-DATA harness control file (ASSIGN name XXXXX062) is an indexed file the site
# provides; the modules OPEN it I-O at startup. This court seeds it with an EMPTY starter (key
# X(6), the RAW-DATA-KEY layout) created by the admitted oracle itself, so modules run their own
# literal-expectation tests standalone instead of aborting on OPEN (status 35).
log "raw-data starter"
if [ ! -f "$RUN_ROOT/materialized/data/XXXXX062" ]; then
  mkdir -p /tmp/rawdata-mk
  cat > /tmp/rawdata-mk/mkstarter.cob <<'EOF'
       IDENTIFICATION DIVISION.
       PROGRAM-ID. MKSTARTER.
       ENVIRONMENT DIVISION.
       INPUT-OUTPUT SECTION.
       FILE-CONTROL.
           SELECT RAW-DATA ASSIGN TO "XXXXX062"
               ORGANIZATION IS INDEXED
               ACCESS MODE IS RANDOM
               RECORD KEY IS RAW-DATA-KEY.
       DATA DIVISION.
       FILE SECTION.
       FD RAW-DATA.
       01 RAW-DATA-SATZ.
           05 RAW-DATA-KEY PIC X(6).
       PROCEDURE DIVISION.
           OPEN OUTPUT RAW-DATA.
           CLOSE RAW-DATA.
           STOP RUN.
EOF
  "$ORACLE_PREFIX/bin/cobc" -x -free -o /tmp/rawdata-mk/mkstarter /tmp/rawdata-mk/mkstarter.cob \
    || fail "mkstarter compile failed"
  ( cd /tmp/rawdata-mk && LD_LIBRARY_PATH="$ORACLE_PREFIX/lib" ./mkstarter ) \
    || fail "mkstarter run failed"
  mkdir -p "$RUN_ROOT/materialized/data"
  cp /tmp/rawdata-mk/XXXXX062 "$RUN_ROOT/materialized/data/XXXXX062"
  log "raw-data starter created ($(stat -c%s "$RUN_ROOT/materialized/data/XXXXX062") bytes)"
else
  log "raw-data starter already present"
fi

# ---------------------------------------------------------------------------------------------
# 4. oracle baseline (GNURUST.CCVS85.2)
# ---------------------------------------------------------------------------------------------
log "oracle baseline (compile + run, pinned GnuCOBOL 3.2)"
export LD_LIBRARY_PATH="$ORACLE_PREFIX/lib"
export COB_CONFIG_DIR="$ORACLE_PREFIX/share/gnucobol/config"
"$CCVS85_BIN" oracle-run --work "$RUN_ROOT/materialized" --prefix "$ORACLE_PREFIX" --jobs "${CCVS85_JOBS:-8}" \
  || fail "oracle-run failed"

# ---------------------------------------------------------------------------------------------
# 5. candidate baseline (GNURUST.CCVS85.3) — oracle isolated away
# ---------------------------------------------------------------------------------------------
log "candidate baseline (cobrun) with the oracle disabled"
ISOLATION_NOTE=$(candidate_isolation) || fail "candidate isolation check failed: $ISOLATION_NOTE"
"$CCVS85_BIN" candidate-run --work "$RUN_ROOT/materialized" --cobrun "$COBRUN" --jobs "${CCVS85_JOBS:-8}" \
  || fail "candidate-run failed"
mv "$ORACLE_PREFIX.disabled" "$ORACLE_PREFIX"
export LD_LIBRARY_PATH="$ORACLE_PREFIX/lib"

cat > "$RUN_ROOT/no-delegation.json" <<EOF
{
  "schema": "gnurust-ccvs85-no-delegation-v1",
  "candidate_phase_isolated": true,
  "candidate_phase_note": "$ISOLATION_NOTE",
  "cobrun_links_no_libcob": true,
  "cobrun_ldd_libcob_hits": $NOCOB_LDD,
  "cobrun_readelf_libcob_hits": $NOCOB_READELF,
  "cobrun_version": "$COBRUN_VERSION",
  "cobc_unavailable_during_candidate_phase": true,
  "candidate_binary_sha256": "$(sha256sum "$COBRUN" | cut -d' ' -f1)",
  "candidate_binary_path": "$COBRUN"
}
EOF

# ---------------------------------------------------------------------------------------------
# 6. classify (GNURUST.CCVS85.4) + evidence copy
# ---------------------------------------------------------------------------------------------
log "classify"
cat > "$RUN_ROOT/meta.json" <<EOF
{
  "generated_at": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "git_commit": "$(cd "$REPO" && git rev-parse HEAD 2>/dev/null || echo unstamped)",
  "crate_version": "$(grep '^version' "$REPO/crates/gnucobol-rs/Cargo.toml" | sed -n "1p" | sed 's/.*"\(.*\)"/\1/')",
  "oracle": {
    "cobc_version": "$COBC_VERSION",
    "cobcrun_version": "$COBCRUN_VERSION",
    "source_sha256": "8ecc77d0a4c9401618b8b99adf2050adef14767916767c54bb42341f0ab504fb",
    "built_prefix": "$ORACLE_PREFIX",
    "cobc_bin_sha256": "$COBC_SHA",
    "libcob_sha256": "$LIBCB_SHA"
  },
  "environment": {
    "LC_ALL": "$LC_ALL", "LANG": "$LANG", "TZ": "$TZ",
    "SOURCE_DATE_EPOCH": "$SOURCE_DATE_EPOCH",
    "uname": "$(uname -srm)",
    "libc": "$(ldd --version | sed -n "1p")"
  }
}
EOF
"$CCVS85_BIN" classify --work "$RUN_ROOT/materialized" --meta "$RUN_ROOT/meta.json" --out "$OUT" \
  || fail "classify failed"

# mirror the raw per-unit evidence (oracle compile/run + candidate run) into the output
mkdir -p "$OUT/raw"
cp -r "$RUN_ROOT/materialized"/u* "$OUT/raw/" 2>/dev/null || true
# also mirror the materialized source files for raw-evidence preservation
mkdir -p "$OUT/raw/sources"
cp -r "$RUN_ROOT/materialized"/*.cob "$OUT/raw/sources/" 2>/dev/null || true
cp -r "$RUN_ROOT/materialized"/copybooks "$OUT/raw/sources/" 2>/dev/null || true
cp -r "$RUN_ROOT/materialized"/data "$OUT/raw/sources/" 2>/dev/null || true
cp "$RUN_ROOT/no-delegation.json" "$OUT/no-delegation.json"

log "pass complete — evidence in $OUT"
ls -la "$OUT" | sed -n "1,20p"

#!/usr/bin/env bash
# diag-unblocked-run.sh — the DIAGNOSTIC-UNBLOCKED pipeline, executed INSIDE the court container.
#
# One invocation = ONE full pass. The host orchestrator runs this twice (pass a/b) in fresh
# containers and compares stable evidence.
#
# Pipeline per pass:
#   1. fresh-extract the admitted GnuCOBOL 3.2 tarball (sha256-verified) into a scratch tree;
#   2. build the tree with the standard court configuration (identical to the pristine lane);
#   3. run the diagnostic-unblocked TRANSFORMER (host-crate build) against the fresh
#      tests/testsuite.src, producing diagnostic-ignore.patch + transformations.* + tree-manifest;
#   4. apply the patch to tests/testsuite.src;
#   5. regenerate the REAL Autotest `testsuite` with the UPSTREAM mechanism:
#        make -C tests testsuite   (autom4te --language=autotest ...)
#   6. run the regenerated suite with the ORACLE (make check) — proves the patch does not change
#      oracle outcomes (diagnostics the oracle itself produces are ignored, so oracle results must
#      match the pristine suite's oracle results);
#   7. run the regenerated suite with the CANDIDATE (make localcheck, COBC=cobc-rs, isolated PATH)
#      — measures how far the candidate gets when diagnostic wording no longer gates groups;
#   8. run the Phase-4 patch-policy gate on the generated patch (independent of the transformer);
#   9. stage per-pass evidence for the host-side determinism compare + receipts.
#
# The lane visibly identifies itself as DIAGNOSTIC-UNBLOCKED and NEVER writes to the pristine
# suite reports (reports/gnucobol-testsuite/* stays untouched; everything lands under
# reports/gnucobol-testsuite/diagnostic-unblocked/).
set -euo pipefail

export LC_ALL=C.UTF-8 LANG=C.UTF-8 TZ=UTC0 SOURCE_DATE_EPOCH=725846400
export TERM="${TERM:-xterm}"
export DEBIAN_FRONTEND=noninteractive
export RUSTUP_HOME=/work/toolchain/rustup CARGO_HOME=/work/toolchain/cargo
export CARGO_TARGET_DIR=/work/target
export PATH="$CARGO_HOME/bin:$PATH"
RUST_TOOLCHAIN="${DIAG_UNBLOCKED_RUST_TOOLCHAIN:-1.96.0}"
JOBS="${DIAG_UNBLOCKED_JOBS:-12}"
PASS="${DIAG_UNBLOCKED_PASS:-a}"

REPO=/repo
ORACLE_SRC=/work/oracle-source/gnucobol-3.2.tar.lz
GNUCOBOL_SRC_SHA256="8ecc77d0a4c9401618b8b99adf2050adef14767916767c54bb42341f0ab504fb"
TREES=/work/trees
TREE="$TREES/$PASS"
RUN_ROOT=/work/run
OUT=/work/outputs
mkdir -p "$RUN_ROOT" "$OUT" "$TREE"

log() { printf '\n=== [diag-unblocked] %s ===\n' "$*"; }
fail() { echo "FATAL: $*" >&2; exit 1; }

log "environment"
uname -a
echo "cpu: $(nproc) cores; pass: $PASS; jobs: $JOBS"

# ---------------------------------------------------------------------------------------------
# 1. oracle source identity (sha256-verified)
# ---------------------------------------------------------------------------------------------
log "oracle source identity"
SRC="$ORACLE_SRC"
for _ in 1 2 3 4 5 6; do
  [ -f "$SRC" ] && break
  echo "warning: $SRC not visible yet (bind flicker?); retrying…"
  sleep 5
done
[ -f "$SRC" ] || fail "pinned oracle source missing at $SRC"
GOT=$(sha256sum "$SRC" | cut -d' ' -f1)
[ "$GOT" = "$GNUCOBOL_SRC_SHA256" ] || fail "oracle source sha256 mismatch: $GOT"
echo "gnucobol-3.2.tar.lz sha256 verified: $GOT"

# ---------------------------------------------------------------------------------------------
# 2. rust toolchain (rustup, pinned)
# ---------------------------------------------------------------------------------------------
log "rust toolchain (rustup, pinned $RUST_TOOLCHAIN)"
if [ ! -x "$CARGO_HOME/bin/cargo" ]; then
  mkdir -p "$CARGO_HOME" "$RUSTUP_HOME"
  curl -fsSL https://sh.rustup.rs | sh -s -- -y --profile minimal \
    --default-toolchain "$RUST_TOOLCHAIN" --no-modify-path
fi
"$CARGO_HOME/bin/rustc" --version
"$CARGO_HOME/bin/cargo" --version

# ---------------------------------------------------------------------------------------------
# 3. fresh GnuCOBOL tree (extract + configure + make) — identical config to the pristine lane
# ---------------------------------------------------------------------------------------------
log "building fresh tree at $TREE"
mkdir -p "$TREE"
tar --lzip -xf "$ORACLE_SRC" --strip-components=1 -C "$TREE"
(
  cd "$TREE"
  ./configure --prefix=/work/oracle/prefix --with-db \
      BDB_CFLAGS="-I/usr/include/db5.3" BDB_LIBS="-ldb-5.3" \
      CFLAGS="-O2 -std=gnu17 -fsigned-char" \
      > "$RUN_ROOT/configure.log" 2>&1 \
    || { tail -30 "$RUN_ROOT/configure.log"; fail "configure failed"; }
  make -j"$(nproc)" > "$RUN_ROOT/make.log" 2>&1 \
    || { tail -30 "$RUN_ROOT/make.log"; fail "make failed"; }
)
log "tree built"
export LD_LIBRARY_PATH="$TREE/libcob/.libs"
COBC_VERSION=$("$TREE/cobc/cobc" --version | sed -n "1p")
echo "oracle identity (in-tree): $COBC_VERSION"

# ---------------------------------------------------------------------------------------------
# 4. diagnostic-unblocked transformer (fresh build of the corpus crate)
# ---------------------------------------------------------------------------------------------
log "building gnucobol-rs-corpus (transformer)"
cd "$REPO"
cargo build --release -p gnucobol-rs-corpus
CORPUS_BIN=/work/target/release/gnucobol-rs-corpus
[ -x "$CORPUS_BIN" ] || fail "corpus CLI not built"

log "running the diagnostic-unblocked transformer on the fresh suite source"
DU_REP="$RUN_ROOT/diag-unblocked"
mkdir -p "$DU_REP"
"$CORPUS_BIN" diag-unblocked transform "$TREE/tests/testsuite.src" "$DU_REP" \
  --revision=stable-3.2 2>&1 | tail -2

log "applying diagnostic-ignore.patch to tests/testsuite.src (upstream tree, scratch copy)"
cd "$TREE/tests/testsuite.src"
patch -p1 < "$DU_REP/diagnostic-ignore.patch" || fail "patch application failed"
echo "patch applied: $(grep -c '\[ignore\]' configuration.at 2>/dev/null || true) ignore in configuration.at"

# ---------------------------------------------------------------------------------------------
# 5. regenerate the REAL Autotest suite with the UPSTREAM mechanism
# ---------------------------------------------------------------------------------------------
log "regenerating the Autotest testsuite (make -C tests testsuite)"
(
  cd "$TREE/tests"
  make testsuite > "$RUN_ROOT/make-testsuite.log" 2>&1 \
    || { tail -20 "$RUN_ROOT/make-testsuite.log"; fail "make testsuite failed"; }
)
GENERATED="$TREE/tests/testsuite"
[ -f "$GENERATED" ] || fail "generated testsuite missing"
GENERATED_SHA=$(sha256sum "$GENERATED" | cut -d' ' -f1)
echo "generated testsuite sha256: $GENERATED_SHA"
echo "generated testsuite size:   $(stat -c%s "$GENERATED")"
echo "autom4te: $(autom4te --version 2>/dev/null | sed -n '1p')"
cp "$GENERATED" "$OUT/generated-testsuite" 2>/dev/null || true

# ---------------------------------------------------------------------------------------------
# 6. oracle run on the REGENERATED suite (make check) — oracle outcomes must match pristine
# ---------------------------------------------------------------------------------------------
log "oracle run on the regenerated suite (make check)"
mkdir -p "$OUT/raw/oracle"
set +e
( cd "$TREE" && timeout --kill-after=30s 5400 make check TESTSUITEFLAGS="--jobs=$JOBS" \
    > "$OUT/raw/oracle/make-check.stdout" 2>&1 )
ORACLE_RC=$?
set -e
echo "oracle make check exit: $ORACLE_RC"
cp "$TREE/tests/testsuite.log" "$OUT/raw/oracle/testsuite.log" 2>/dev/null || true
cp -r "$TREE/tests/testsuite.dir" "$OUT/raw/oracle/testsuite.dir" 2>/dev/null || true

# ---------------------------------------------------------------------------------------------
# 7. candidate run on the REGENERATED suite (make localcheck, COBC=cobc-rs, isolated PATH)
# ---------------------------------------------------------------------------------------------
log "building gnucobol-rs (cobrun + cobc-rs)"
cd "$REPO"
cargo build --release -p gnucobol-rs --example cobrun
cargo build --release -p cobc-rs
COBRUN=/work/target/release/examples/cobrun
[ -x "$COBRUN" ] || fail "cobrun not built"
[ -x "/work/target/release/cobc-rs" ] || fail "cobc-rs not built"

log "candidate run on the regenerated suite (make localcheck)"
CAND_BIN="$RUN_ROOT/candidate-bin"
mkdir -p "$CAND_BIN"
ln -sfn /work/target/release/cobc-rs "$CAND_BIN/cobc-rs"
ln -sfn /work/target/release/cobc-rs "$CAND_BIN/cobcrun-rs"
ln -sfn cobc-rs "$CAND_BIN/cobc"
ln -sfn cobcrun-rs "$CAND_BIN/cobcrun"
chmod +x "$CAND_BIN/cobc-rs" "$CAND_BIN/cobcrun-rs" "$CAND_BIN/cobc" "$CAND_BIN/cobcrun"
COBC_RS_SHA=$(sha256sum /work/target/release/cobc-rs | cut -d' ' -f1)

# isolation: the oracle must be unreachable during the candidate phase
export PATH="$CAND_BIN:/usr/bin:/bin:/usr/sbin:/sbin"
unset LD_LIBRARY_PATH COB_CONFIG_DIR COB_COPY_DIR COB_LIBRARY_PATH GNUCOBOL_CENSUS_FILE
command -v cobc | grep -q "$CAND_BIN" || fail "cobc resolves outside the candidate bin"
[ ! -e /work/oracle/prefix/bin/cobc ] || fail "oracle prefix present during candidate phase"

mkdir -p "$OUT/raw/candidate"
rm -f "$TREE/tests/testsuite.log"
set +e
( cd "$TREE/tests" && env COBC="$CAND_BIN/cobc" COBCRUN="$CAND_BIN/cobcrun" \
    timeout --kill-after=30s 3600 make localcheck TESTSUITEFLAGS="--jobs=$JOBS" \
    > "$OUT/raw/candidate/make-localcheck.stdout" 2>&1 )
CANDIDATE_RC=$?
set -e
echo "candidate make localcheck exit: $CANDIDATE_RC"
cp "$TREE/tests/testsuite.log" "$OUT/raw/candidate/testsuite.log" 2>/dev/null || true
cp -r "$TREE/tests/testsuite.dir" "$OUT/raw/candidate/testsuite.dir" 2>/dev/null || true

# ---------------------------------------------------------------------------------------------
# 8. Phase-4 patch-policy gate (independent verification inside the same container)
# ---------------------------------------------------------------------------------------------
log "diag-unblocked policy gate (independent of the transformer)"
"$CORPUS_BIN" diag-unblocked gate \
  "$DU_REP/diagnostic-ignore.patch" \
  "$DU_REP/pristine" "$DU_REP/transformed" "$DU_REP/transformations.json" \
  || fail "diag-unblocked policy gate failed"

# ---------------------------------------------------------------------------------------------
# 9. per-pass identity meta + evidence staging
# ---------------------------------------------------------------------------------------------
log "per-pass identity meta"
GIT_SHA=$(cd "$REPO" && git rev-parse HEAD 2>/dev/null || echo unstamped)
CRATE_VERSION=$(grep '^version' "$REPO/crates/gnucobol-rs/Cargo.toml" | head -1 | sed 's/.*"\(.*\)"/\1/')
cat > "$RUN_ROOT/meta.json" <<EOF
{
  "schema": "gnurust-diag-unblocked-pass-meta-v1",
  "pass": "$PASS",
  "git_commit": "$GIT_SHA",
  "crate_version": "$CRATE_VERSION",
  "cobc_version": "$COBC_VERSION",
  "configure": "./configure --prefix=/work/oracle/prefix --with-db BDB_CFLAGS=-I/usr/include/db5.3 BDB_LIBS=-ldb-5.3 CFLAGS=-O2 -std=gnu17 -fsigned-char",
  "generated_testsuite_sha256": "$GENERATED_SHA",
  "generated_testsuite_bytes": $(stat -c%s "$GENERATED"),
  "patch_sha256": $(sha256sum "$DU_REP/diagnostic-ignore.patch" | cut -d' ' -f1 | sed 's/.*/"&"/'),
  "transformer_version": "gnurust-diag-unblocked-transform-v1",
  "candidate_binary_sha256": "$COBC_RS_SHA",
  "environment": {
    "LC_ALL": "${LC_ALL:-}", "LANG": "${LANG:-}", "TZ": "${TZ:-}",
    "SOURCE_DATE_EPOCH": "${SOURCE_DATE_EPOCH:-}",
    "RUST_TOOLCHAIN": "$RUST_TOOLCHAIN",
    "jobs": "$JOBS", "nproc": "$(nproc)"
  }
}
EOF
cp "$RUN_ROOT/meta.json" "$OUT/meta.json"
cp "$DU_REP/diagnostic-ignore.patch" "$OUT/diagnostic-ignore.patch" 2>/dev/null || true
cp "$DU_REP/transformations.json" "$OUT/transformations.json" 2>/dev/null || true
cp "$DU_REP/tree-manifest.json" "$OUT/tree-manifest.json" 2>/dev/null || true

log "pass $PASS raw evidence complete — host-side compare + receipts run next"
echo "pass $PASS complete: regenerated-suite oracle + candidate + gate evidence staged"

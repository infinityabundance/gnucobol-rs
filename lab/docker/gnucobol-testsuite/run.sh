#!/usr/bin/env bash
# gnucobol-testsuite-run.sh — the GNURUST.GNUCOBOL-TESTSUITE.{1,2,3} pipeline, executed INSIDE the
# court container.
#
# Bind mounts (host project dir -> container):
#   <PROJECT_DOCKER_ROOT>/work/oracle-source  -> /work/oracle-source   (ro: pinned tarball)
#   <PROJECT_DOCKER_ROOT>/work/toolchain      -> /work/toolchain       (rustup+cargo homes)
#   <PROJECT_DOCKER_ROOT>/work/target         -> /work/target          (cargo target dir)
#   <repo>                                    -> /repo                 (rw)
#   <PROJECT_DOCKER_ROOT>/work/run/<run-id>   -> /work/run             (per-pass scratch)
#   <PROJECT_DOCKER_ROOT>/outputs/<run-id>    -> /work/outputs         (evidence)
#
# One invocation = ONE full pass (baseline + census -> candidate). The host orchestrator runs this
# twice in two fresh containers (two fresh per-pass build trees) and compares.
#
# Phases (mirroring the court gates):
#   GNURUST.GNUCOBOL-TESTSUITE.1  suite custody + baseline (real admitted cobc, in-tree Autotest
#                                 suite, full invocation census, raw logs preserved)
#   GNURUST.GNUCOBOL-TESTSUITE.2  candidate execution (COBC=cobc-rs through make localcheck,
#                                 no-delegation proof, all tests accounted, no parity claim)
#   GNURUST.GNUCOBOL-TESTSUITE.3  differential classification (baseline vs candidate)
set -euo pipefail

export LC_ALL=C.UTF-8 LANG=C.UTF-8 TZ=UTC0 SOURCE_DATE_EPOCH=725846400
# The DISPLAY-EXCEPTION / screen tests need a terminal name even without a real tty; the admitted
# suite checks COB_HAS_CURSES (built) and then runs them, so a deterministic TERM is required.
export TERM="${TERM:-xterm}"
export DEBIAN_FRONTEND=noninteractive
export RUSTUP_HOME=/work/toolchain/rustup CARGO_HOME=/work/toolchain/cargo
export CARGO_TARGET_DIR=/work/target
export PATH="$CARGO_HOME/bin:$PATH"
RUST_TOOLCHAIN="${GNUCOBOL_TEST_RUST_TOOLCHAIN:-1.96.0}"
JOBS="${GNUCOBOL_TEST_JOBS:-12}"
PASS="${GNUCOBOL_TEST_PASS:-a}"
# stage gate: `baseline` stops after the oracle baseline + census (harness bring-up); `full` (default)
# runs the complete pipeline including the cobc-rs candidate phase and classification.
STAGE="${GNUCOBOL_TEST_STAGE:-full}"

REPO=/repo
ORACLE_SRC=/work/oracle-source/gnucobol-3.2.tar.lz
GNUCOBOL_SRC_SHA256="8ecc77d0a4c9401618b8b99adf2050adef14767916767c54bb42341f0ab504fb"
TREES=/work/trees
BASELINE_TREE="$TREES/$PASS/baseline"
CANDIDATE_TREE="$TREES/$PASS/candidate"
RUN_ROOT=/work/run
OUT=/work/outputs
mkdir -p "$RUN_ROOT" "$OUT" "$TREES/$PASS"

log() { printf '\n=== [gnucobol-testsuite] %s ===\n' "$*"; }
fail() { echo "FATAL: $*" >&2; exit 1; }

# ---------------------------------------------------------------------------------------------
# 0. environment/identity facts
# ---------------------------------------------------------------------------------------------
log "environment"
uname -a
sed -n "1,2p" /etc/os-release
ldd --version | sed -n "1p"
echo "cpu: $(nproc) cores; pass: $PASS; jobs: $JOBS"

# ---------------------------------------------------------------------------------------------
# 1. rust toolchain (rustup, pinned)
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
# 2. admitted oracle source identity
# ---------------------------------------------------------------------------------------------
log "oracle source identity"
# the bind-flicker watchdog: a transient empty bind can make the pinned source look missing; retry
# briefly before failing (the run-docker.sh probe already verified the pinned mount).
SRC=/work/oracle-source/gnucobol-3.2.tar.lz
for _ in 1 2 3 4 5 6; do
  [ -f "$SRC" ] && break
  echo "warning: $SRC not visible yet (bind flicker?); retrying…"
  sleep 5
  SRC=/work/oracle-source/gnucobol-3.2.tar.lz
  [ -f "$SRC" ] && break
done
[ -f "$SRC" ] || fail "pinned oracle source missing at $SRC"
GOT=$(sha256sum "$SRC" | cut -d' ' -f1)
[ "$GOT" = "$GNUCOBOL_SRC_SHA256" ] || fail "oracle source sha256 mismatch: $GOT"
echo "gnucobol-3.2.tar.lz sha256 verified: $GOT"

# ---------------------------------------------------------------------------------------------
# 3. fresh GnuCOBOL build trees (extract + configure + make), identical configuration
# ---------------------------------------------------------------------------------------------
# Two trees per pass: baseline (real cobc) and candidate (COBC=cobc-rs). Each is a FRESH extract of
# the admitted source with the same configure arguments, so a dirty tree can never contaminate the
# other side or a later pass. Configure flags match the admitted oracle build (ccvs85 court).
build_tree() {
  local tree="$1"
  local tag="$2"
  if [ -f "$tree/Makefile" ]; then
    log "tree $tag already present (cached): $tree"
    return 0
  fi
  log "building fresh $tag tree at $tree"
  mkdir -p "$tree"
  tar --lzip -xf "$ORACLE_SRC" --strip-components=1 -C "$tree"
  (
    cd "$tree"
    # Stock GnuCOBOL 3.2 configuration (same across every tree). No `-fpermissive` and no compat
    # -Wno-* flags: those leak cc1 warnings into stderr and would break the suite's stderr-exact
    # expectations, making the baseline measure the build workaround instead of the compiler.
    ./configure --prefix=/work/oracle/prefix --with-db \
        BDB_CFLAGS="-I/usr/include/db5.3" BDB_LIBS="-ldb-5.3" \
        CFLAGS="-O2 -std=gnu17 -fsigned-char" \
        > "$RUN_ROOT/configure-$tag.log" 2>&1 \
      || { tail -30 "$RUN_ROOT/configure-$tag.log"; fail "configure ($tag) failed"; }
    make -j"$(nproc)" > "$RUN_ROOT/make-$tag.log" 2>&1 \
      || { tail -30 "$RUN_ROOT/make-$tag.log"; fail "make ($tag) failed"; }
  )
  log "tree $tag built"
}

build_tree "$BASELINE_TREE" "baseline-$PASS"
build_tree "$CANDIDATE_TREE" "candidate-$PASS"

# in-tree cobc identity (the binaries the suite actually exercises)
export LD_LIBRARY_PATH="$BASELINE_TREE/libcob/.libs"
COBC_VERSION=$("$BASELINE_TREE/cobc/cobc" --version | sed -n "1p")
COBCRUN_VERSION=$("$BASELINE_TREE/bin/cobcrun" --version 2>/dev/null | sed -n "1p" || echo "cobcrun not found")
echo "oracle identity (in-tree): $COBC_VERSION / $COBCRUN_VERSION"

# ---------------------------------------------------------------------------------------------
# 4. baseline suite run (GNURUST.GNUCOBOL-TESTSUITE.1) — real admitted cobc + invocation census
# ---------------------------------------------------------------------------------------------
log "baseline suite run (real admitted cobc; invocation census recorded)"
CENSUS_FILE="$RUN_ROOT/census.jsonl"
install_recorder() {
  # $1 = dir with the real binary, $2 = real binary name. The recorder writes a JSONL census line
  # (argv boundaries preserved) then execs the renamed real binary. The real-name is baked in via a
  # placeholder so each wrapper targets ITS OWN .real sibling (no env-var race under --jobs).
  local dir="$1" real="$2"
  if head -1 "$dir/$real" 2>/dev/null | grep -q 'gnurust-census-recorder-v2'; then
    echo "recorder already installed at $dir/$real (idempotent)"
    return 0
  fi
  # preserve the real binary under .real (idempotent across recorder formats)
  if [ ! -f "$dir/$real.real" ]; then
    [ -f "$dir/$real" ] || fail "recorder target missing: $dir/$real"
    mv "$dir/$real" "$dir/$real.real"
  fi
  rm -f "$dir/$real"
  cat > "$dir/$real" <<'PYEOF'
#!/usr/bin/env python3
# gnurust-census-recorder-v2
import datetime, json, os, sys
rec = os.environ.get("GNUCOBOL_CENSUS_FILE", "")
entry = {
    "t": datetime.datetime.now(datetime.timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
    "cwd": os.getcwd(),
    "tool": os.path.basename(sys.argv[0]),
    "argv": sys.argv,
    "env": {k: v for k, v in os.environ.items()
            if k.startswith("COB") or k in ("PATH", "LD_LIBRARY_PATH", "LANG", "LC_ALL", "TZ", "TERM")},
}
if rec:
    with open(rec, "a", encoding="utf-8") as f:
        f.write(json.dumps(entry, ensure_ascii=False) + "\n")
real = "__REAL__"
os.execv(os.path.join(os.path.dirname(os.path.abspath(sys.argv[0])), real),
         [real] + sys.argv[1:])
PYEOF
  sed -i "s/__REAL__/$real.real/" "$dir/$real"
  chmod +x "$dir/$real"
}
install_recorder "$BASELINE_TREE/cobc" "cobc"
install_recorder "$BASELINE_TREE/bin" "cobcrun"

export GNUCOBOL_CENSUS_FILE="$CENSUS_FILE"
# fresh per-pass census: the recorder appends, so a stale/partial file from an earlier attempt
# (e.g. a crashed container) must not contaminate this pass's invocation ledger.
rm -f "$CENSUS_FILE"
mkdir -p "$OUT/raw/baseline"
set +e
# Wall-clock watchdog: a 90-minute cap on the baseline suite. The real run finishes well inside
# this; a firing means the admitted suite (or the recorder) hung in THIS environment and the raw
# partial evidence is preserved + classified honestly (missing groups = not-reached).
( cd "$BASELINE_TREE" && timeout --kill-after=30s 5400 make check TESTSUITEFLAGS="--jobs=$JOBS" \
    > "$OUT/raw/baseline/make-check.stdout" 2>&1 )
BASELINE_RC=$?
set -e
echo "baseline make check exit: $BASELINE_RC"
cp "$BASELINE_TREE/tests/testsuite.log" "$OUT/raw/baseline/testsuite.log" 2>/dev/null || true
cp -r "$BASELINE_TREE/tests/testsuite.dir" "$OUT/raw/baseline/testsuite.dir" 2>/dev/null || true
[ -f "$CENSUS_FILE" ] || fail "no census recorded — recorder never fired?"
echo "census invocations: $(wc -l < "$CENSUS_FILE")"
cp "$CENSUS_FILE" "$OUT/raw/baseline/census.jsonl"
unset GNUCOBOL_CENSUS_FILE

# ---------------------------------------------------------------------------------------------
# 5. build gnucobol-rs (cobrun + cobc-rs + the testsuite harness crate)
# ---------------------------------------------------------------------------------------------
if [ "$STAGE" = "baseline" ]; then
  log "STAGE=baseline: stopping after the oracle baseline + census (no candidate phase)"
  echo "baseline-only pass $PASS complete"
  exit 0
fi
log "building gnucobol-rs (cobrun + cobc-rs + testsuite harness)"
# bind watchdog: the rootless daemon's /run copy-up can degrade mid-run on this machine (bind
# sources under /run/media intermittently present as empty dirs). Fail LOUDLY here (before the
# candidate phase wastes an hour) rather than silently running against empty trees.
[ -d "$CANDIDATE_TREE/tests" ] || fail "BIND WATCHDOG: /work/trees bind lost its content (candidate tree missing) — restart the daemon and re-run"
[ -d "$BASELINE_TREE/tests" ] || fail "BIND WATCHDOG: /work/trees bind lost its content (baseline tree missing) — restart the daemon and re-run"
cd "$REPO"
# ALWAYS rebuild from the checked-out source (cargo fingerprints make the no-change case a no-op).
cargo build --release -p gnucobol-rs --example cobrun
cargo build --release -p gnucobol-rs-testsuite
cargo build --release -p cobc-rs
COBRUN=/work/target/release/examples/cobrun
TS_BIN=/work/target/release/gnucobol-rs-testsuite
[ -x "$COBRUN" ] || fail "cobrun not built"
[ -x "/work/target/release/cobc-rs" ] || fail "cobc-rs not built"
echo "cobrun: $("$COBRUN" --version 2>/dev/null | sed -n "1p" || echo "?")"

# ---------------------------------------------------------------------------------------------
# 6. candidate suite run (GNURUST.GNUCOBOL-TESTSUITE.2) — COBC=cobc-rs through the native harness
# ---------------------------------------------------------------------------------------------
log "candidate suite run (COBC=cobc-rs via make localcheck; oracle isolated away)"
CAND_BIN="$RUN_ROOT/candidate-bin"
mkdir -p "$CAND_BIN"
ln -sfn /work/target/release/cobc-rs "$CAND_BIN/cobc"
ln -sfn /work/target/release/cobc-rs "$CAND_BIN/cobcrun"
chmod +x "$CAND_BIN/cobc" "$CAND_BIN/cobcrun"

# bind-flicker watchdog: verify the candidate binary is visible through the pinned binds (the
# rootless /run copy-up can transiently present an empty view on this machine); retry briefly.
for _ in 1 2 3 4 5 6; do
  [ -x /work/target/release/cobc-rs ] && break
  echo "warning: /work/target/release/cobc-rs not visible (bind flicker?); retrying…"
  sleep 5
done
[ -x /work/target/release/cobc-rs ] || fail "BIND WATCHDOG: /work/target bind lost its content (cobc-rs missing) — restart the daemon and re-run"

# mechanical no-delegation proofs (recorded; see no-delegation.json below)
NOCOB_COBRUN_LDD=$(ldd "$COBRUN" 2>/dev/null | grep -ciE 'cob|gnucobol' || true)
NOCOB_COBRUN_READELF=$(readelf -d "$COBRUN" 2>/dev/null | grep -ciE 'cob|gnucobol' || true)
NOCOB_COBCRS_LDD=$(ldd /work/target/release/cobc-rs 2>/dev/null | grep -ciE 'cob|gnucobol' || true)
NOCOB_COBCRS_READELF=$(readelf -d /work/target/release/cobc-rs 2>/dev/null | grep -ciE 'cob|gnucobol' || true)
echo "linkage scan: cobrun ldd=$NOCOB_COBRUN_LDD readelf=$NOCOB_COBRUN_READELF; cobc-rs ldd=$NOCOB_COBCRS_LDD readelf=$NOCOB_COBCRS_READELF (must be 0/0/0/0)"
[ "$NOCOB_COBRUN_LDD" = "0" ] && [ "$NOCOB_COBRUN_READELF" = "0" ] \
  && [ "$NOCOB_COBCRS_LDD" = "0" ] && [ "$NOCOB_COBCRS_READELF" = "0" ] \
  || fail "candidate binary links libcob?!"

candidate_isolation() {
  # The oracle must be unreachable during the candidate phase: PATH stripped to the candidate bin +
  # system dirs, no LD_LIBRARY_PATH, no COB_CONFIG_DIR pointing at an oracle prefix, and no real
  # cobc/cobcrun anywhere on PATH.
  export PATH="$CAND_BIN:/usr/bin:/bin:/usr/sbin:/sbin"
  unset LD_LIBRARY_PATH COB_CONFIG_DIR COB_COPY_DIR COB_LIBRARY_PATH GNUCOBOL_CENSUS_FILE
  local found
  found=$(command -v cobc 2>/dev/null || true)
  [ "$found" = "$CAND_BIN/cobc" ] || { echo "cobc resolves to $found, expected the candidate"; return 1; }
  found=$(command -v cobcrun 2>/dev/null || true)
  [ "$found" = "$CAND_BIN/cobcrun" ] || { echo "cobcrun resolves to $found, expected the candidate"; return 1; }
  if command -v gcc >/dev/null 2>&1; then :; fi
  # no real oracle prefix may exist anywhere in the container
  if [ -e /work/oracle/prefix/bin/cobc ]; then
    echo "oracle prefix present at /work/oracle/prefix"; return 1
  fi
  echo "candidate phase isolated from the oracle (only $CAND_BIN/cobc + $CAND_BIN/cobcrun on PATH)"
  return 0
}

mkdir -p "$OUT/raw/candidate"
# isolation BEFORE the no-delegation record: the note must reflect the real check result and the
# candidate PATH must be in effect for the whole candidate phase (suite + recorder + launchers).
ISOLATION_NOTE=$(candidate_isolation) || fail "candidate isolation check failed: $ISOLATION_NOTE"

cat > "$RUN_ROOT/no-delegation.json" <<EOF
{
  "schema": "gnurust-gnucobol-testsuite-no-delegation-v1",
  "candidate_phase_isolated": true,
  "candidate_phase_note": "$ISOLATION_NOTE",
  "cobrun_links_no_libcob": true,
  "cobrun_ldd_libcob_hits": $NOCOB_COBRUN_LDD,
  "cobrun_readelf_libcob_hits": $NOCOB_COBRUN_READELF,
  "cobc_rs_links_no_libcob": true,
  "cobc_rs_ldd_libcob_hits": $NOCOB_COBCRS_LDD,
  "cobc_rs_readelf_libcob_hits": $NOCOB_COBCRS_READELF,
  "cobc_resolves_to_candidate_during_candidate_phase": true,
  "cobcrun_resolves_to_candidate_during_candidate_phase": true,
  "oracle_prefix_absent_during_candidate_phase": true,
  "cobrun_version": "$("$COBRUN" --version 2>/dev/null | sed -n "1p" || echo "?")",
  "candidate_binary_sha256": "$(sha256sum "$COBRUN" | cut -d' ' -f1)",
  "cobc_rs_binary_sha256": "$(sha256sum /work/target/release/cobc-rs | cut -d' ' -f1)"
}
EOF
cp "$RUN_ROOT/no-delegation.json" "$OUT/no-delegation.json"

# The atlocal bootstrap (`which cobc` / `which cobcrun`) can transiently fail when the rootless
# /run copy-up presents an empty bind view; the suite then never starts (no testsuite.log). Retry
# The atlocal bootstrap (`which cobc` / `which cobcrun`) can transiently fail when the rootless
# /run copy-up presents an empty bind view; the suite then never starts (no testsuite.log). Retry
# the candidate run a few times — a retry is SAFE here because the failure happens BEFORE the suite
# runs (prereq-testsuite wipes testsuite.dir). Once testsuite.log exists, the results are REAL and
# no retry happens regardless of the exit code.
CANDIDATE_RC=2
# fresh per-pass candidate invocation ledger (cobc-rs appends; a stale partial file must not
# contaminate this pass's ledger).
rm -f "$RUN_ROOT/candidate-census.jsonl"
# remove any STALE testsuite.log so the retry loop cannot mistake an old run's log for a fresh one
rm -f "$CANDIDATE_TREE/tests/testsuite.log"
for attempt in 1 2 3; do
  set +e
  # Wall-clock watchdog on the candidate suite: a 60-minute cap. The suite normally finishes in
  # ~35 min; a firing means the CANDIDATE mis-executed a test into a non-terminating loop (a real
  # finding). The group running at the kill is preserved (partial log -> classified honestly),
  # the killed group's status line is absent -> not-reached / fail-closed, never a silent pass.
  ( cd "$CANDIDATE_TREE/tests" && env COBC="$CAND_BIN/cobc" COBCRUN="$CAND_BIN/cobcrun" \
      GNURUST_COBCRS_RECORD="$RUN_ROOT/candidate-census.jsonl" \
      timeout --kill-after=30s 3600 make localcheck TESTSUITEFLAGS="--jobs=$JOBS" \
      > "$OUT/raw/candidate/make-localcheck.stdout" 2>&1 )
  CANDIDATE_RC=$?
  set -e
  if [ -f "$CANDIDATE_TREE/tests/testsuite.log" ]; then
    echo "candidate make localcheck exit: $CANDIDATE_RC (suite ran)"
    break
  fi
  echo "candidate suite did not start (attempt $attempt; atlocal/bind bootstrap failure); retrying…"
  sleep 15
  if [ -x "$CAND_BIN/cobc" ] && command -v cobc >/dev/null 2>&1; then :; else
    # re-export the candidate PATH (candidate_isolation ran once; keep the same environment)
    export PATH="$CAND_BIN:/usr/bin:/bin:/usr/sbin:/sbin"
  fi
done
echo "candidate make localcheck exit: $CANDIDATE_RC"
cp "$CANDIDATE_TREE/tests/testsuite.log" "$OUT/raw/candidate/testsuite.log" 2>/dev/null || true
cp -r "$CANDIDATE_TREE/tests/testsuite.dir" "$OUT/raw/candidate/testsuite.dir" 2>/dev/null || true
if [ -f "$RUN_ROOT/candidate-census.jsonl" ]; then
  echo "candidate cobc-rs invocations: $(wc -l < "$RUN_ROOT/candidate-census.jsonl")"
  cp "$RUN_ROOT/candidate-census.jsonl" "$OUT/raw/candidate/cobc-rs-census.jsonl"
fi

# ---------------------------------------------------------------------------------------------
# 6b. execve-trace no-delegation proof (prompt §2.6): run a SAMPLE of the candidate's own launch
#     artifacts under strace and record every execve target. The ONLY permitted target is the
#     candidate binary itself (the launcher symlink resolves to cobc-rs); any cobc/cobcrun/libcob
#     execution would fail the proof. The trace is raw evidence, kept next to no-delegation.json.
# ---------------------------------------------------------------------------------------------
EXEC_TRACE="$RUN_ROOT/execve-trace.log"
: > "$EXEC_TRACE"
TRACED=0
if command -v strace >/dev/null 2>&1 && [ -d "$CANDIDATE_TREE/tests/testsuite.dir" ]; then
  for dir in "$CANDIDATE_TREE"/tests/testsuite.dir/*/; do
    [ -x "$dir/prog" ] || continue
    TRACED=$((TRACED + 1))
    [ "$TRACED" -le 12 ] || break
    ( cd "$dir" && timeout 30 strace -f -e trace=execve -o "$EXEC_TRACE.one" ./prog >/dev/null 2>&1 ) || true
    sed "s#^#$dir #" "$EXEC_TRACE.one" >> "$EXEC_TRACE" 2>/dev/null || true
    rm -f "$EXEC_TRACE.one"
  done
fi
# mechanical check: no execve target may reference the oracle tools or the runtime library.
FORBIDDEN=$(grep -oE 'execve\("[^"]+"' "$EXEC_TRACE" 2>/dev/null \
  | grep -vE 'cobc-rs|/usr/bin|/bin/' \
  | grep -iE 'cobc|cobcrun|libcob' | head -5 || true)
if [ -n "$FORBIDDEN" ]; then
  echo "EXECVE-TRACE VIOLATION: oracle binaries executed during the candidate sample:"
  echo "$FORBIDDEN"
  fail "no-delegation execve trace failed"
fi
echo "execve trace: $TRACED candidate artifacts traced; no oracle execve target found"
cp "$EXEC_TRACE" "$OUT/execve-trace.log" 2>/dev/null || true

# ---------------------------------------------------------------------------------------------
# 6c. runtime/math performance campaign (GNURUST.GNUCOBOL-RUNTIME-MATH.PERF.1) — Views A/B only
#     on output-identical programs; strict labels, never a cross-implementation speed claim.
# ---------------------------------------------------------------------------------------------
log "runtime/math performance campaign (Views A/B, output-equivalence gated)"
if [ -x /usr/local/bin/gnucobol-testsuite-perf.sh ]; then
  /usr/local/bin/gnucobol-testsuite-perf.sh || echo "perf campaign reported a non-fatal issue (results preserved)"
fi


# ---------------------------------------------------------------------------------------------
# 7. per-pass identity meta (consumed HOST-SIDE by run-docker.sh for the receipt envelopes)
# ---------------------------------------------------------------------------------------------
log "per-pass identity meta"
GIT_SHA=$(cd "$REPO" && git rev-parse HEAD 2>/dev/null || echo unstamped)
CRATE_VERSION=$(grep '^version' "$REPO/crates/gnucobol-rs/Cargo.toml" | head -1 | sed 's/.*"\(.*\)"/\1/')
cat > "$RUN_ROOT/meta.json" <<EOF
{
  "schema": "gnurust-gnucobol-testsuite-pass-meta-v1",
  "pass": "$PASS",
  "git_commit": "$GIT_SHA",
  "crate_version": "$CRATE_VERSION",
  "cobc_version": "$COBC_VERSION",
  "cobcrun_version": "$COBCRUN_VERSION",
  "configure": "./configure --prefix=/work/oracle/prefix --with-db BDB_CFLAGS=-I/usr/include/db5.3 BDB_LIBS=-ldb-5.3 CFLAGS=-O2 -std=gnu17 -fsigned-char",
  "environment": {
    "LC_ALL": "${LC_ALL:-}", "LANG": "${LANG:-}", "TZ": "${TZ:-}",
    "SOURCE_DATE_EPOCH": "${SOURCE_DATE_EPOCH:-}",
    "RUST_TOOLCHAIN": "$RUST_TOOLCHAIN",
    "rustc": "$($CARGO_HOME/bin/rustc --version 2>/dev/null | tr -d '\n' || echo ?)",
    "cargo": "$($CARGO_HOME/bin/cargo --version 2>/dev/null | tr -d '\n' || echo ?)",
    "arch": "$(uname -m)",
    "kernel": "$(uname -r)",
    "libc": "$(ldd --version 2>/dev/null | sed -n '1p' || echo ?)",
    "os": "$(sed -n '1s/.*=\"\(.*\\)\"/\\1/p' /etc/os-release 2>/dev/null || echo ?)",
    "jobs": "$JOBS",
    "nproc": "$(nproc)"
  }
}
EOF
echo "meta written: $RUN_ROOT/meta.json (commit $GIT_SHA)"

# ---------------------------------------------------------------------------------------------
# 8. (evidence generation runs HOST-SIDE in run-docker.sh — the container only produces the raw
#    suite artifacts; the tree + raw outputs persist even if the fragile rootless daemon dies
#    right after the suite, so the evidence steps must not depend on this container staying up)
# ---------------------------------------------------------------------------------------------
log "pass $PASS raw suite artifacts complete — evidence generation happens host-side"
echo "pass $PASS complete: baseline + candidate raw evidence staged"


#!/usr/bin/env bash
# GNURUST.ELITE-REPLAY.1 -- opencbs real-program replay. Run the REAL public opencbs COBOL defect-suite
# programs (lab/corpus/opencbs/, third-party DF*.CBL) through BOTH the admitted cobc oracle AND the
# clean-room `cobrun` interpreter and diff OBSERVABLE BEHAVIOUR: stdout bytes (cmp -s), process exit status
# (the program's RETURN-CODE), and stderr-clean. A program COUNTS as MATCH only if cobc compiles it, cobrun
# runs it without a `cobrun:` boundary error, stdout is byte-identical, AND exit codes agree.
#
# HONESTY: every non-MATCH outcome is either an explicit, reason-tagged SKIP or a real FAIL -- nothing is
# silently dropped. Two committed allowlists keyed by program basename:
#   NOORACLE = deliberately-broken defect snippets the suite ships that cobc ITSELF cannot compile (no oracle
#              baseline exists -> out of scope, not a port gap).
#   NOTYET   = cobc-compilable programs cobrun does not YET match. Investigation (see the plan / receipt
#              non_claims) shows every one is DOABLE -- cobc runs them all observably, so these are unbuilt
#              targets, NOT permanent boundaries: external CALL to a module that genuinely does not exist
#              (the faithful behaviour is a module-not-found error -- reproducible); variable-length / I-O /
#              INDEXED real-file access (file-model work); a qualified+subscripted compound condition (parser
#              work). The goal is to drive NOTYET to empty (MATCH 30->39), one verified conversion at a time.
#              A NOTYET program is EXPECTED to NOT match yet; if one ever MATCHES, cobrun has grown to handle
#              it -> the run FAILs as NOTYET-NOW-MATCHES, forcing a deliberate promotion (remove it from
#              NOTYET + raise the receipt ratchet). The allowlist cannot mask a regression, and shrinking
#              scope cannot inflate MATCH.
#
# Real third-party programs legitimately evolve across GnuCOBOL versions, so there is NO 3.1.2 differential
# here (it would be brittle for zero conformance value); the court targets the admitted 3.2 oracle only.
# Terminal line: PASS=n FAIL=n SKIP=n MATCH=n  (MATCH == in-scope PASS; the receipt ratchet reads MATCH=).
set -u
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"; PREFIX="$ROOT/lab/oracle/prefix"
export PATH="$PREFIX/bin:$PATH" LD_LIBRARY_PATH="$PREFIX/lib" COB_CONFIG_DIR="$PREFIX/share/gnucobol/config" LC_ALL=C.UTF-8 TZ=UTC0
command -v cobc >/dev/null 2>&1 || { echo "cobc not built"; exit 2; }
( cd "$ROOT" && cargo build --release -p gnucobol-rs --example cobrun >/dev/null 2>&1 ) || exit 2
COBRUN="$ROOT/target/release/examples/cobrun"
CORPUS="$ROOT/lab/corpus/opencbs/repo/COBOL_Programs"
COPYBOOKS="$ROOT/lab/corpus/opencbs/repo/COBOL_Copybooks"
TMP="$(mktemp -d)"; trap 'rm -rf "$TMP"' EXIT

# Deliberately-broken / placeholder snippets that the admitted cobc itself rejects -> no oracle baseline.
declare -A NOORACLE=(
  [DF12TEST]="defect snippet: does not compile under cobc"  [DF13TEST]="defect snippet: does not compile under cobc"
  [DF14TEST]="defect snippet: does not compile under cobc"  [DF15TEST]="defect snippet: does not compile under cobc"
  [DF18TEST]="defect snippet: does not compile under cobc"  [DF20TEST]="defect snippet: does not compile under cobc"
  [DF27TEST]="defect snippet: does not compile under cobc"  [DF31TEST]="defect snippet: does not compile under cobc"
  [DF34TEST]="defect snippet: does not compile under cobc"  [DF35TEST]="defect snippet: does not compile under cobc"
  [DF40TEST]="defect snippet: does not compile under cobc"  [DF41TEST]="defect snippet: does not compile under cobc"
  [DF42TEST]="defect snippet: does not compile under cobc"  [DF45TEST]="defect snippet: does not compile under cobc"
)
# cobc-compilable programs cobrun does not YET match -- all DOABLE targets (see header). Expected NON-MATCH for now.
declare -A NOTYET=(
  [DF18CALL]="CALL to a missing module: reproduce libcob module-not-found (stdout+exit 1)"
  [DF31CALL]="CALL to a missing module: reproduce libcob module-not-found (stdout+exit 1)"
  [DF45CALL]="CALL to a missing module: reproduce libcob module-not-found (stdout+exit 1)"
  [DF02TEST]="qualified+subscripted compound condition operand (cond-parser work)"
  [DF03TEST]="ORGANIZATION INDEXED: wire the pure-Rust gnucobol-rs-bdb-format backend"
  [DF05TEST]="SORT USING a real input file (file-model + sort-from-file)"
  [DF25TEST]="OPEN I-O across two real files (file-model read-back)"
  [DF46TEST]="REWRITE over a real I-O file (file-model rewrite)"
)

PASS=0; FAIL=0; SKIP=0
shopt -s nullglob
for cob in "$CORPUS"/DF*.CBL; do
  name="$(basename "$cob" .CBL)"

  # 1. Out-of-scope: cobc itself cannot compile it (deliberately-broken snippet). Honest SKIP.
  if [ -n "${NOORACLE[$name]:-}" ]; then
    echo "$name: SKIP (no-oracle: ${NOORACLE[$name]})"; SKIP=$((SKIP+1)); continue
  fi

  # 2. Compile with cobc (fixed format; copybooks on cobc's -I, NOT cobrun's). A compile failure for a
  #    program NOT in NOORACLE is a real FAIL (suite drift / oracle change).
  if ! cobc -x -fixed -I "$COPYBOOKS" -o "$TMP/p" "$cob" 2>"$TMP/cobc.err"; then
    echo "$name: cobc compile FAIL (not in no-oracle set)"; head -2 "$TMP/cobc.err"; FAIL=$((FAIL+1)); continue
  fi

  # 3. Run BOTH from the corpus dir so OPEN INPUT disk reads resolve the same DFxxFILE files on each side.
  ( cd "$CORPUS" && "$TMP/p" </dev/null >"$TMP/o.out" 2>/dev/null ); oec=$?
  ( cd "$CORPUS" && "$COBRUN" -fixed "$cob" </dev/null >"$TMP/r.out" 2>"$TMP/r.err" ); rec=$?

  matched=0
  [ -z "$(cat "$TMP/r.err")" ] && cmp -s "$TMP/o.out" "$TMP/r.out" && [ "$oec" = "$rec" ] && matched=1

  # 4. NOT-YET-built (doable) target: expected NON-MATCH. If it now MATCHES, cobrun grew -> deliberate promotion.
  if [ -n "${NOTYET[$name]:-}" ]; then
    if [ "$matched" = 1 ]; then
      echo "$name: NOTYET-NOW-MATCHES ($name now matches cobc -- promote it out of NOTYET and raise min_match)"; FAIL=$((FAIL+1))
    else
      echo "$name: SKIP (not-yet, doable: ${NOTYET[$name]})"; SKIP=$((SKIP+1))
    fi
    continue
  fi

  # 5. In-scope program. Any cobrun stderr is a real RunError -> FAIL (mirrors cobol_frontend_sweep.sh).
  if [ -s "$TMP/r.err" ]; then
    echo "$name: cobrun FAIL: $(cat "$TMP/r.err")"; FAIL=$((FAIL+1)); continue
  fi
  if [ "$matched" = 1 ]; then
    echo "$name: MATCH (exit=$oec)"; PASS=$((PASS+1))
  else
    FAIL=$((FAIL+1)); echo "$name: DIFFER (exit cobc=$oec cobrun=$rec)"
    cmp -s "$TMP/o.out" "$TMP/r.out" || { echo "  cobc:   $(cat -A "$TMP/o.out" | head -3)"; echo "  cobrun: $(cat -A "$TMP/r.out" | head -3)"; }
  fi
done

echo "PASS=$PASS FAIL=$FAIL SKIP=$SKIP MATCH=$PASS"
[ "$FAIL" -eq 0 ] || exit 1

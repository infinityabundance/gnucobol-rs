#!/usr/bin/env bash
# GNURUST.ELITE-REPLAY.2 -- the BROAD public-corpus differential. Run a large set of real public COBOL
# programs (the GCC-COBOL "gcobol" testsuite + the hand-authored front-end corpus) through BOTH the admitted
# cobc 3.2 oracle AND the clean-room `cobrun` interpreter, and diff OBSERVABLE BEHAVIOUR: stdout bytes
# (cmp -s) + process exit status. One command, one boring-green receipt.
#
# THE GREEN INVARIANT: cobrun is byte-identical to cobc wherever it runs, and FAILS CLOSED everywhere it
# cannot -- it never silently mis-runs. Every program lands in exactly one bucket:
#   MATCH      -- cobc compiled+ran it AND cobrun reproduced stdout+exit byte-for-byte.
#   no-oracle  -- cobc 3.2 itself cannot compile it (GCC-COBOL-dialect / deliberately-failing test) -> no
#                 oracle baseline, out of scope (SKIP).
#   boundary   -- cobrun fails CLOSED with a typed `cobrun:` error (a construct outside the sealed subset)
#                 -> honest SKIP, never a wrong answer.
#   DIVERGENCE -- cobrun ran clean (no `cobrun:`), but stdout/exit differ from cobc. This is the ONLY bad
#                 bucket: a silent wrong answer. The committed allowlist `lab/oracle/elite_replay2_known.txt`
#                 tracks each known divergence (one `path  # reason` per line) as a bug to drive down; an
#                 UN-listed divergence FAILs the sweep. The allowlist ratchets toward empty (like opencbs).
# Terminal line: PASS=n FAIL=n SKIP=n MATCH=n  (FAIL = un-allowlisted divergence; the receipt reads MATCH=).
set -u
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"; PREFIX="$ROOT/lab/oracle/prefix"
export PATH="$PREFIX/bin:$PATH" LD_LIBRARY_PATH="$PREFIX/lib" COB_CONFIG_DIR="$PREFIX/share/gnucobol/config" LC_ALL=C.UTF-8 TZ=UTC0
command -v cobc >/dev/null 2>&1 || { echo "cobc not built"; exit 2; }
( cd "$ROOT" && cargo build --release -p gnucobol-rs --example cobrun >/dev/null 2>&1 ) || exit 2
COBRUN="$ROOT/target/release/examples/cobrun"
KNOWN="$ROOT/lab/oracle/elite_replay2_known.txt"
TMP="$(mktemp -d)"; trap 'rm -rf "$TMP"' EXIT

# Known silent divergences (relative paths under lab/corpus), each tracked with a reason; ratchets to empty.
is_known() { [ -f "$KNOWN" ] && grep -qE "^$1([[:space:]]|#|$)" "$KNOWN"; }

PASS=0; FAIL=0; SKIP=0; NOORA=0; BND=0
shopt -s nullglob
# The broad public corpus: GCC-COBOL testsuite (free format) + the hand-authored front-end corpus (free).
mapfile -t PROGS < <(find "$ROOT/lab/corpus/gcobol" "$ROOT/lab/corpus/frontend" \( -iname '*.cob' -o -iname '*.cbl' \) | sort)
for f in "${PROGS[@]}"; do
  rel="${f#"$ROOT"/}"
  dir="$(dirname "$f")"; base="$(basename "$f")"
  # 1. cobc oracle (free format). No oracle baseline -> SKIP (out of scope).
  if ! cobc -x -free -o "$TMP/p" "$f" 2>/dev/null; then
    NOORA=$((NOORA+1)); SKIP=$((SKIP+1)); continue
  fi
  # 2. run BOTH from the program's own dir (so any relative copybook/data resolves the same), no stdin,
  #    bounded so a pathological loop cannot hang the sweep.
  ( cd "$dir" && timeout 15 "$TMP/p" </dev/null >"$TMP/o.out" 2>/dev/null ); oec=$?
  # The cobc-built oracle itself CRASHED (SIGSEGV/SIGABRT) or timed out -> no reliable baseline -> no-oracle.
  if [ "$oec" -ge 132 ]; then NOORA=$((NOORA+1)); SKIP=$((SKIP+1)); continue; fi
  ( cd "$dir" && timeout 15 "$COBRUN" "$dir/$base" </dev/null >"$TMP/r.out" 2>"$TMP/r.err" ); rec=$?
  # 3. cobrun failed CLOSED (typed boundary) -> honest SKIP (never a wrong answer).
  if grep -q "^cobrun:" "$TMP/r.err"; then
    BND=$((BND+1)); SKIP=$((SKIP+1)); continue
  fi
  # 4. MATCH iff stdout byte-identical AND exit status agrees.
  if cmp -s "$TMP/o.out" "$TMP/r.out" && [ "$oec" = "$rec" ]; then
    PASS=$((PASS+1)); continue
  fi
  # 5. Silent divergence: bad unless it is a committed, tracked known-divergence.
  if is_known "$rel"; then
    SKIP=$((SKIP+1)); echo "$rel: KNOWN-DIVERGENCE (tracked)"
  else
    FAIL=$((FAIL+1)); echo "$rel: DIVERGENCE (cobrun ran clean but output != cobc; exit oc=$oec rc=$rec)"
  fi
done
echo "corpus=$((${#PROGS[@]})) no-oracle=$NOORA boundary=$BND"
echo "PASS=$PASS FAIL=$FAIL SKIP=$SKIP MATCH=$PASS"
[ "$FAIL" -eq 0 ] || exit 1

#!/usr/bin/env bash
# Multi-record FD differential sweep (GNURUST.FILEIO.MULTI-RECORD-FD.1): for each fixture with SEVERAL
# 01-level record descriptions beneath one FD, run it through the CLEAN-ROOM gnucobol-rs front-end
# `cobrun` AND through the admitted cobc, and diff stdout byte-for-byte. The fixtures prove:
#   - WRITE of ANY declared FD record resolves to its owning file and emits the NAMED record's bytes;
#   - the FD records share ONE record area (GnuCOBOL union: a MOVE into one record is visible through
#     every other -- verified against the oracle);
#   - different-length records emit their own lengths; group records lay out independently;
#   - records under different FDs never cross-associate;
#   - the CCVS85 `WRITE DUMMY-RECORD AFTER ADVANCING` shape parses and runs (the sweep also asserts the
#     oracle's line-control file bytes: n x LF before the record + a final LF at close).
# PASS=n FAIL=n.
set -u
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"; PREFIX="$ROOT/lab/oracle/prefix"
export PATH="$PREFIX/bin:$PATH" LD_LIBRARY_PATH="$PREFIX/lib" COB_CONFIG_DIR="$PREFIX/share/gnucobol/config" LC_ALL=C.UTF-8 TZ=UTC0
command -v cobc >/dev/null 2>&1 || { echo "cobc not built"; exit 2; }
( cd "$ROOT" && cargo build --release -p gnucobol-rs --example cobrun >/dev/null 2>&1 ) || exit 2
COBRUN="$ROOT/target/release/examples/cobrun"
CORPUS="$ROOT/lab/corpus/frontend"
TMP="$(mktemp -d)"; trap 'rm -rf "$TMP"' EXIT

PASS=0; FAIL=0
cd "$TMP" || exit 2
for cob in "$CORPUS"/p186_multifd_*.cob "$CORPUS"/p187_multifd_*.cob "$CORPUS"/p188_multifd_*.cob \
          "$CORPUS"/p189_multifd_*.cob "$CORPUS"/p190_multifd_*.cob "$CORPUS"/p191_multifd_*.cob; do
  [ -f "$cob" ] || continue
  name="$(basename "$cob" .cob)"
  if ! cobc -free -x -o p "$cob" 2>"$TMP/cobc.err"; then
    echo "$name: COBC COMPILE FAIL"; FAIL=$((FAIL+1)); continue
  fi
  if ! ./p > oracle.out 2>oracle.err; then
    echo "$name: COBC RUN FAIL"; FAIL=$((FAIL+1)); continue
  fi
  if ! "$COBRUN" -free "$cob" > cobrun.out 2>cobrun.err; then
    echo "$name: COBRUN RUN FAIL"; FAIL=$((FAIL+1)); continue
  fi
  if cmp -s oracle.out cobrun.out; then
    echo "$name: IDENTICAL"
    PASS=$((PASS+1))
  else
    echo "$name: OUTPUT DIFFERS"
    diff oracle.out cobrun.out | head -5
    FAIL=$((FAIL+1))
  fi
done

# The advancing fixture's line-control bytes are the ORACLE-side file content (both stdouts are empty).
# Pin the documented model: AFTER ADVANCING n -> n x LF before the record; the file ends with a final LF
# at close (GnuCOBOL's pending-needs-nl). p191 writes 120-byte records: ADV 1 then ADV 2.
if [ -f "$TMP/p191.dat" ]; then
  SIZE=$(stat -c %s "$TMP/p191.dat")
  # 1 LF + 120 + 2 LF + 120 + 1 close LF = 244
  if [ "$SIZE" = "244" ]; then
    echo "p191 line-control bytes: PASS (244 = 1xLF + 120 + 2xLF + 120 + close LF)"
    PASS=$((PASS+1))
  else
    echo "p191 line-control bytes: FAIL (size $SIZE, expected 244)"
    FAIL=$((FAIL+1))
  fi
else
  echo "p191.dat missing -- advancing oracle bytes not asserted"
  FAIL=$((FAIL+1))
fi

echo "PASS=$PASS FAIL=$FAIL"
[ "$FAIL" = "0" ]

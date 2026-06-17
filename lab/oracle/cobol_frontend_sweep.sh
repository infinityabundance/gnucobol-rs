#!/usr/bin/env bash
# COBOL FRONT-END sweep (the "Not a COBOL compiler" negative -> positive): for a corpus of small COBOL
# programs, run each through the CLEAN-ROOM gnucobol-rs front-end `cobrun` (parse + execute on the
# ported runtime -- no cobc, no libcob linked) AND through the admitted cobc, and diff stdout. A
# byte-identical result for every program is the proof: gnucobol-rs parses and runs these COBOL
# programs to cobc-identical output. (Sealed subset: WORKING-STORAGE 01 elementary items with
# PIC/VALUE; MOVE / ADD / SUBTRACT / MULTIPLY / DIVIDE / DISPLAY / STOP RUN. Anything else fails
# closed.) PASS=n FAIL=n.
set -u
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"; PREFIX="$ROOT/lab/oracle/prefix"
export PATH="$PREFIX/bin:$PATH" LD_LIBRARY_PATH="$PREFIX/lib" COB_CONFIG_DIR="$PREFIX/share/gnucobol/config" LC_ALL=C.UTF-8
command -v cobc >/dev/null 2>&1 || { echo "cobc not built"; exit 2; }
( cd "$ROOT" && cargo build --release -p gnucobol-rs --example cobrun >/dev/null 2>&1 ) || exit 2
COBRUN="$ROOT/target/release/examples/cobrun"
CORPUS="$ROOT/lab/corpus/frontend"
P312="$ROOT/lab/oracle/prefix-312"
TMP="$(mktemp -d)"; trap 'rm -rf "$TMP"' EXIT

PASS=0; FAIL=0; DIFF=0
shopt -s nullglob
for cob in "$CORPUS"/*.cob; do
  name="$(basename "$cob" .cob)"
  # Optional per-program dialect: a `*> @std: NAME` header line selects -std= for BOTH cobc and cobrun
  # (so the dialect-sensitive corpus -- e.g. defaultbyte fill -- is compared under the same dialect).
  STD="$(sed -n 's/^[[:space:]]*\*>[[:space:]]*@std:[[:space:]]*\([A-Za-z0-9]*\).*/\1/p' "$cob" | head -1)"
  STDOPT=""; [ -n "$STD" ] && STDOPT="-std=$STD"
  # Optional per-program runtime env: a `*> @env: NAME=VALUE` header is exported for BOTH the cobc binary
  # and cobrun (so a runtime-env-driven program -- e.g. a UPSI COB_SWITCH_n -- is compared under the same env).
  ENVKV="$(sed -n 's/^[[:space:]]*\*>[[:space:]]*@env:[[:space:]]*\([A-Za-z0-9_]*=[^ ]*\).*/\1/p' "$cob" | head -1)"
  # Optional source format: an `@format: fixed` marker (free `*>` OR fixed col-7 `*` comment) compiles +
  # runs the program FIXED-format for BOTH cobc and cobrun; the default is free.
  FMT="$(sed -n 's/.*@format:[[:space:]]*\(fixed\|free\).*/\1/p' "$cob" | head -1)"
  FMTOPT="-free"; FIXEDOPT=""; [ "$FMT" = fixed ] && { FMTOPT="-fixed"; FIXEDOPT="-fixed"; }
  if ! cobc -x $FMTOPT $STDOPT -o "$TMP/p" "$cob" 2>"$TMP/cobc.err"; then
    echo "$name: cobc compile FAIL"; head -2 "$TMP/cobc.err"; FAIL=$((FAIL+1)); continue
  fi
  env $ENVKV "$TMP/p" </dev/null > "$TMP/oracle.out" 2>/dev/null
  if ! env $ENVKV "$COBRUN" $FIXEDOPT $STDOPT "$cob" > "$TMP/rust.out" 2>"$TMP/rust.err"; then
    echo "$name: cobrun FAIL: $(cat "$TMP/rust.err")"; FAIL=$((FAIL+1)); continue
  fi
  if cmp -s "$TMP/oracle.out" "$TMP/rust.out"; then
    PASS=$((PASS+1)); echo "$name: IDENTICAL"
    # DIFFERENTIAL: the same program through GnuCOBOL 3.1.2 must also match cobrun (version-stability).
    # EXCEPT dialect-tagged programs (`*> @std:`): the compile-time dialect config legitimately evolves
    # across GnuCOBOL versions (e.g. -std=ibm defaultbyte for alphanumeric storage is space in 3.1.2 but
    # 0x00 in 3.2). The port targets the admitted 3.2 oracle, so such programs are exempt from the 3.1.2
    # cross-check by design (not a divergence in the port).
    if [ -n "$STD" ]; then
      echo "$name: 3.1.2 differential SKIPPED (dialect $STD evolves across versions; port targets 3.2)"
    elif [ -x "$P312/bin/cobc" ]; then
      if PATH="$P312/bin:$PATH" LD_LIBRARY_PATH="$P312/lib" COB_CONFIG_DIR="$P312/share/gnucobol/config" \
           cobc -x $FMTOPT $STDOPT -o "$TMP/p312" "$cob" 2>/dev/null; then
        env $ENVKV LD_LIBRARY_PATH="$P312/lib" "$TMP/p312" </dev/null > "$TMP/o312.out" 2>/dev/null
        cmp -s "$TMP/o312.out" "$TMP/rust.out" && DIFF=$((DIFF+1)) || { echo "$name: 3.1.2 DIFFER"; FAIL=$((FAIL+1)); }
      fi
    fi
  else
    FAIL=$((FAIL+1))
    echo "$name: DIFFER"
    echo "  cobc:   $(cat -A "$TMP/oracle.out")"
    echo "  cobrun: $(cat -A "$TMP/rust.out")"
  fi
done
echo "PASS=$PASS FAIL=$FAIL (3.1.2 differential-matched=$DIFF)"
[ "$FAIL" -eq 0 ] || exit 1

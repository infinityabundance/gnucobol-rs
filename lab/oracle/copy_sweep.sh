#!/usr/bin/env bash
# COPY expansion differential sweep (GNURUST.5): compare the Rust copybook expander to GnuCOBOL's
# preprocessor (`cobc -P`) at TEXT-WORD granularity (immune to the preprocessor's column/indent
# reformatting). Prints PASS=n FAIL=n over curated COPY programs. On FAIL emits the word diff.
set -u
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
PREFIX="$ROOT/lab/oracle/prefix"
CASES="$ROOT/lab/oracle/copy_cases"
export PATH="$PREFIX/bin:$PATH" LD_LIBRARY_PATH="$PREFIX/lib"
export COB_CONFIG_DIR="$PREFIX/share/gnucobol/config" COB_COPY_DIR="$PREFIX/share/gnucobol/copy"
export LC_ALL=C.UTF-8

echo "building Rust copy_rows (release)..." >&2
( cd "$ROOT" && cargo build --release -p gnucobol-rs --examples >/dev/null 2>&1 ) || exit 2
ROWS="$ROOT/target/release/examples/copy_rows"

TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT
# Collapse to one text-word per line (whitespace-delimited), dropping blank lines.
tok() { tr -s '[:space:]' '\n' | grep -vE '^$'; }

PASS=0
FAIL=0
for prog in "$CASES"/prog*.cob; do
  [ -f "$prog" ] || continue
  base=$(basename "$prog")
  # Oracle: cobc -P expands COPY; strip the leading line-number column, then tokenize.
  if ! cobc -P="$TMP/exp.i" -I "$CASES" -fsyntax-only "$prog" 2>"$TMP/cerr"; then
    echo "cobc -P failed on $base: $(cat "$TMP/cerr")" >&2; FAIL=$((FAIL + 1)); continue
  fi
  sed -E 's/^[[:space:]]*[0-9]+[[:space:]]//' "$TMP/exp.i" | tok > "$TMP/oracle.tok"
  # Rust: expand and tokenize.
  "$ROWS" "$CASES" < "$prog" | tok > "$TMP/rust.tok"
  if diff -q "$TMP/oracle.tok" "$TMP/rust.tok" >/dev/null; then
    PASS=$((PASS + 1))
  else
    FAIL=$((FAIL + 1))
    echo "--- WORD MISMATCH in $base (oracle '<' vs rust '>') ---" >&2
    diff "$TMP/oracle.tok" "$TMP/rust.tok" | head -20 >&2
  fi
done

echo "programs=$((PASS + FAIL))  PASS=$PASS FAIL=$FAIL"
[ "$FAIL" -eq 0 ]

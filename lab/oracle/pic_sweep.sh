#!/usr/bin/env bash
# PIC differential sweep (GNURUST.3): compare the Rust `build_field` PIC->attr+size computation to
# the GnuCOBOL compiler's own (via lab/oracle/pic_harness.sh). Prints PASS=n FAIL=n; on FAIL emits
# replayable rows. ROOT derived from script path.
set -u
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"

echo "building Rust pic examples (release)..."
( cd "$ROOT" && cargo build --release -p gnucobol-rs --examples >/dev/null 2>&1 ) || exit 2
GEN="$ROOT/target/release/examples/gen_pics"
PIC_ROWS="$ROOT/target/release/examples/pic_rows"

TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

"$GEN" > "$TMP/cases.txt"
TOTAL=$(grep -cvE '^\s*$' "$TMP/cases.txt")

# Oracle wants (label, pic, usage, sign); cases already have those 4 columns.
bash "$ROOT/lab/oracle/pic_harness.sh" < "$TMP/cases.txt" | sort > "$TMP/oracle.txt"
"$PIC_ROWS" < "$TMP/cases.txt" | sort > "$TMP/rust.txt"

if diff -q "$TMP/oracle.txt" "$TMP/rust.txt" >/dev/null; then
  echo "PASS=$TOTAL FAIL=0"
  exit 0
fi

FAILS=$(diff "$TMP/oracle.txt" "$TMP/rust.txt" | grep -cE '^<')
echo "PASS=$((TOTAL - FAILS)) FAIL=$FAILS"
echo "--- divergences (oracle '<' vs rust '>'), with case row ---"
diff "$TMP/oracle.txt" "$TMP/rust.txt" | grep -E '^[<>]' | head -20 | while read -r dir label rest; do
  row=$(grep -E "^$label\b" "$TMP/cases.txt" | head -1)
  echo "$dir $label $rest    [case: $row]"
done
exit 1

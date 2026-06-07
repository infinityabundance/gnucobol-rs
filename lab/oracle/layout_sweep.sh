#!/usr/bin/env bash
# DATA DIVISION layout differential sweep (GNURUST.4): for each curated record, compare the Rust
# layout engine's offsets/sizes to the GnuCOBOL compiler's (via layout_harness.sh). Every item the
# compiler emits must match in offset and size; the record total must match. Prints PASS=n FAIL=n
# counting per-item comparisons. On FAIL emits the diverging items + record.
set -u
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
CASES="${1:-$ROOT/lab/oracle/layout_cases.txt}"

echo "building Rust layout example (release)..." >&2
( cd "$ROOT" && cargo build --release -p gnucobol-rs --examples >/dev/null 2>&1 ) || exit 2
ROWS="$ROOT/target/release/examples/layout_rows"

TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

PASS=0
FAIL=0
RECS=0

# Split CASES into record blocks on blank lines (ignoring comment lines).
awk 'BEGIN{n=0} /^[[:space:]]*#/ {next} /^[[:space:]]*$/ {if (buf!=""){print buf > ("'"$TMP"'/rec_" n); n++; buf=""} next} {buf=(buf=="")?$0:buf"\n"$0} END{if (buf!=""){print buf > ("'"$TMP"'/rec_" n)}}' "$CASES"

for f in "$TMP"/rec_*; do
  [ -f "$f" ] || continue
  RECS=$((RECS + 1))
  if ! bash "$ROOT/lab/oracle/layout_harness.sh" < "$f" > "$TMP/oracle.raw" 2>"$TMP/herr"; then
    echo "harness failed on $(head -1 "$f"): $(cat "$TMP/herr")" >&2; FAIL=$((FAIL + 1)); continue
  fi
  sort "$TMP/oracle.raw" > "$TMP/oracle.txt"
  "$ROWS" < "$f" | sort > "$TMP/rust.txt"
  if grep -q LAYOUT_ERROR "$TMP/rust.txt"; then
    echo "rust layout error: $(cat "$TMP/rust.txt")  [rec: $(head -1 "$f")]" >&2
    FAIL=$((FAIL + 1)); continue
  fi
  # Every oracle item (name off size) must match the rust line of the same name.
  while read -r name off size; do
    rust=$(grep -E "^$name " "$TMP/rust.txt" | head -1)
    if [ "$rust" = "$name $off $size" ]; then
      PASS=$((PASS + 1))
    else
      FAIL=$((FAIL + 1))
      echo "MISMATCH [$name] oracle='$name $off $size' rust='$rust'  [rec: $(head -1 "$f")]" >&2
    fi
  done < "$TMP/oracle.txt"
done

echo "records=$RECS  PASS=$PASS FAIL=$FAIL"
[ "$FAIL" -eq 0 ]

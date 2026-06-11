#!/usr/bin/env bash
# INSPECT byte-effect sweep (GNURUST.INSPECT.1). For each case build an INSPECT statement, run it against
# cobc/libcob, capture the count receiver bytes (TALLYING) or the target bytes (REPLACING/CONVERTING) via a
# DISPLAY, and check inspect_* == the oracle bytes.
set -u
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"; PREFIX="$ROOT/lab/oracle/prefix"
export PATH="$PREFIX/bin:$PATH" LD_LIBRARY_PATH="$PREFIX/lib" COB_CONFIG_DIR="$PREFIX/share/gnucobol/config" LC_ALL=C.UTF-8
command -v cobc >/dev/null 2>&1 || { echo "cobc not built"; exit 2; }
( cd "$ROOT" && cargo build --release -p gnucobol-rs --examples >/dev/null 2>&1 ) || exit 2
GEN="$ROOT/target/release/examples/gen_inspect"; ROWS="$ROOT/target/release/examples/inspect_rows"
TMP="$(mktemp -d)"; trap 'rm -rf "$TMP"' EXIT
"$GEN" > "$TMP/cases.tsv"

( cd "$ROOT" && cargo run -q -p xtask -- sweep-inspect "$TMP/cases.tsv" "$TMP" ) | "$ROWS"

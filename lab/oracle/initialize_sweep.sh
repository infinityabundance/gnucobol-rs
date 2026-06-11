#!/usr/bin/env bash
# INITIALIZE byte-effect sweep (GNURUST.INITIALIZE.1). For each case: build a record, MOVE ALL "~" sentinel
# into it via a REDEFINES X view, INITIALIZE it, DISPLAY the raw bytes, and check initialize_record == the
# post-INITIALIZE bytes (which bytes are changed vs preserved).
set -u
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"; PREFIX="$ROOT/lab/oracle/prefix"
export PATH="$PREFIX/bin:$PATH" LD_LIBRARY_PATH="$PREFIX/lib" COB_CONFIG_DIR="$PREFIX/share/gnucobol/config" LC_ALL=C.UTF-8
command -v cobc >/dev/null 2>&1 || { echo "cobc not built"; exit 2; }
( cd "$ROOT" && cargo build --release -p gnucobol-rs --examples >/dev/null 2>&1 ) || exit 2
GEN="$ROOT/target/release/examples/gen_initialize"; ROWS="$ROOT/target/release/examples/initialize_rows"
TMP="$(mktemp -d)"; trap 'rm -rf "$TMP"' EXIT
"$GEN" > "$TMP/cases.tsv"

( cd "$ROOT" && cargo run -q -p xtask -- sweep-initialize "$TMP/cases.tsv" "$TMP" ) | "$ROWS"

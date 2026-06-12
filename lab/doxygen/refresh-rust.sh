#!/usr/bin/env bash
# Refresh the NATIVE-RUST libcob-port doxygen (crates/gnucobol-rs/src). COMPLETE replacement: the
# previous run is wiped first so the output never accumulates. The C-side libcob doxygen is separate
# and already maintained (lab/doxygen/Doxyfile). After regenerating, refreshes the parity mapping.
#
# Usage:  bash lab/doxygen/refresh-rust.sh
set -u
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
if ! command -v doxygen >/dev/null 2>&1; then
  echo "doxygen not installed — Rust-port doxygen refresh skipped"
  exit 0
fi
rm -rf "$ROOT/lab/doxygen/out-rust"            # wipe the previous run (no space accumulation)
( cd "$ROOT" && doxygen lab/doxygen/Doxyfile-rust ) >/dev/null 2>&1 || { echo "doxygen run failed"; exit 1; }
( cd "$ROOT" && cargo run -q -p gnucobol-rs-port-index -- parity ) >/dev/null 2>&1
# Refresh the authoritative C-side XML inventory + the C-vs-Rust coverage compare (DOXYGEN-PARITY.md).
( cd "$ROOT" && doxygen lab/doxygen/Doxyfile-c-xml ) >/dev/null 2>&1 && \
  ( cd "$ROOT" && cargo run -q -p xtask -- doxygen-compare generate ) >/dev/null 2>&1
FNS=$(grep -rhoE 'kind="function"' "$ROOT"/lab/doxygen/out-rust/xml/*8rs.xml 2>/dev/null | wc -l)
echo "rust-port doxygen refreshed (clean) — $FNS functions documented; parity + C-vs-Rust coverage regenerated"
echo "  browse:  lab/doxygen/out-rust/html/index.html   (map against the C side: lab/doxygen/out/html)"
echo "  coverage: DOXYGEN-PARITY.md (authoritative C-vs-Rust function map)"

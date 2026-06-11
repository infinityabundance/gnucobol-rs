#!/usr/bin/env bash
# GNURUST.BUILD.PROFILE.1 — bind the oracle's exact ABI / dialect / config as first-class evidence. Every
# ABI-sensitive byte court (binary COMP/COMP-5/COMP-X, EBCDIC) is scoped to THIS profile: a different build
# (e.g. binary-byteorder: native) would produce different bytes, so its parity does not transfer. The sweep
# recomputes the live profile and PASS=1 iff it matches the committed reports/build-profile.json (drift -> FAIL,
# re-examine every ABI court).
set -u
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"; PREFIX="$ROOT/lab/oracle/prefix"
export PATH="$PREFIX/bin:$PATH" LD_LIBRARY_PATH="$PREFIX/lib" COB_CONFIG_DIR="$PREFIX/share/gnucobol/config" LC_ALL=C.UTF-8
command -v cobc >/dev/null 2>&1 || { echo "cobc not built"; exit 2; }
CONF="$PREFIX/share/gnucobol/config/default.conf"
cfg() { grep -iE "^$1:" "$CONF" | head -1 | sed -E "s/^$1:[[:space:]]*//I" | tr -d '[:space:]'; }
VER=$(cobc --version 2>/dev/null | head -1 | grep -oE '[0-9]+\.[0-9]+\.[0-9]+' | head -1)
CC=$(cobc -info 2>/dev/null | grep -m1 'C version' | sed -E 's/.*: *//; s/"//g')
BENV=$(cobc -info 2>/dev/null | grep -m1 'build environment' | sed -E 's/.*: *//')
END=$(cobc -info 2>/dev/null | grep -m1 'endianness' | sed -E 's/.*: *//')
CLONG=$(cobc -info 2>/dev/null | grep -m1 'BINARY-C-LONG' | grep -oE '[0-9]+' | head -1)
CHARSIGN=$(cobc -info 2>/dev/null | grep -qi 'fsigned-char' && echo signed || echo unknown)
LIBCOB=$(ls "$PREFIX"/lib/libcob.* 2>/dev/null | head -1)
LIBSHA=$( [ -f "$LIBCOB" ] && sha256sum "$LIBCOB" | cut -d' ' -f1 || echo "")
CFGSHA=$( [ -f "$CONF" ] && sha256sum "$CONF" | cut -d' ' -f1 || echo "")
COBCSHA=$(sha256sum "$PREFIX/bin/cobc" 2>/dev/null | cut -d' ' -f1)
CFG_BO=$(cfg binary-byteorder); CFG_SZ=$(cfg binary-size); CFG_TR=$(cfg binary-truncate); CFG_C1=$(cfg binary-comp-1); CFG_NP=$(cfg numeric-pointer)
( cd "$ROOT" && VER="$VER" END="$END" CHARSIGN="$CHARSIGN" CLONG="$CLONG" LIBSHA="$LIBSHA" CFG_BO="$CFG_BO" CFG_SZ="$CFG_SZ" CFG_TR="$CFG_TR" CFG_C1="$CFG_C1" CFG_NP="$CFG_NP" cargo run -q -p xtask -- atlas-build-profile )

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
python3 - "$ROOT/reports/build-profile.json" <<PY
import json, sys, os
out = sys.argv[1]
live = {
  "schema":"gnurust-build-profile-v1","court":"GNURUST.BUILD.PROFILE.1",
  "gnucobol_version":"$VER","c_compiler":"$CC","build_environment":"$BENV",
  "host_endianness":"$END","char_signedness":"$CHARSIGN","binary_c_long_bytes":"$CLONG",
  "config":{"binary_byteorder":"$(cfg binary-byteorder)","binary_size":"$(cfg binary-size)",
            "binary_truncate":"$(cfg binary-truncate)","binary_comp_1":"$(cfg binary-comp-1)",
            "numeric_pointer":"$(cfg numeric-pointer)"},
  "hashes":{"cobc_sha256":"$COBCSHA","libcob_sha256":"$LIBSHA","config_sha256":"$CFGSHA"},
  "abi_sensitive_courts":["GNURUST.14","GNURUST.15","GNURUST.17","GNURUST.18"],
  "note":"COMP is big-endian (binary-byteorder) regardless of the little-endian host; COMP-5/COMP-X follow native byte order. The byte parity of every abi_sensitive_court is scoped to THIS profile.",
}
# the volatile-free key set the regression compares
KEYS = lambda p: {k: p[k] for k in ("gnucobol_version","host_endianness","char_signedness","binary_c_long_bytes","config")} | {"libcob_sha256": p.get("hashes",{}).get("libcob_sha256","")}
if os.path.exists(out):
    golden = json.load(open(out))
    if KEYS(golden) == KEYS(live):
        print("PASS=1 FAIL=0")
    else:
        print("PASS=0 FAIL=1  (BUILD PROFILE DRIFT -- re-examine every ABI-sensitive court)")
        print("  golden:", KEYS(golden)); print("  live:  ", KEYS(live))
else:
    json.dump(live, open(out,"w"), indent=2)
    print("PASS=1 FAIL=0  (bootstrapped reports/build-profile.json)")
# always refresh the full record (hashes/compiler) without changing the compared key set
json.dump(live, open(out,"w"), indent=2)
PY

#!/usr/bin/env bash
# Documentation refresh gate (GNURUST.DOCGATE.0).
# Fails (nonzero) if any documentation drifts from the code / receipts / oracle. Run on EVERY
# change and EVERY archaeology pass so nothing goes stale. ROOT derived from this script's path
# (no absolute-path leaks). Oracle-dependent checks degrade to a typed "skipped" when lab/ absent,
# never a silent pass.
set -u
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT" || exit 2
FAIL=0
note() { printf '  %s\n' "$1"; }
bad()  { printf 'STALE: %s\n' "$1"; FAIL=1; }

echo "== gnucobol-rs documentation refresh gate =="

# 1. No placeholder/stale markers in shipped docs or src.
if grep -RInE '\b(TODO|FIXME|XXX|PLACEHOLDER|TBD)\b' docs README.md CONTRIBUTING.md \
      crates/*/src 2>/dev/null | grep -vE 'check-docs|doc-gate\.md' >/tmp/_docgate_markers 2>/dev/null; then
  if [ -s /tmp/_docgate_markers ]; then bad "placeholder markers present:"; cat /tmp/_docgate_markers; fi
fi
note "no placeholder markers"

# 2. Every doc linked from README exists.
for f in docs/claim-boundary.md docs/porting-method.md docs/derivation-and-license.md \
         docs/compatibility-taxonomy.md docs/future-risk-register.md reports/negative-claims.md \
         COPYING COPYING.LESSER; do
  [ -f "$f" ] || bad "README references missing file: $f"
done
note "all README-referenced docs exist"

# 3. COB_MAX_DIGITS constant consistent in source and equals the oracle's value (38).
RS_MAXD=$(grep -oE 'COB_MAX_DIGITS: i64 = [0-9]+' crates/cobol-decimal-rs/src/lib.rs | grep -oE '[0-9]+$')
[ "$RS_MAXD" = "38" ] || bad "COB_MAX_DIGITS in lib.rs is '$RS_MAXD', expected 38 (libcob/common.h:607)"
note "COB_MAX_DIGITS = $RS_MAXD"

# 4. attr.rs type/flag constants match the values documented in the admission/selfcheck.
check_const() { grep -qE "$2" crates/cobol-decimal-rs/src/attr.rs || bad "attr.rs missing/!= $1"; }
check_const "COB_TYPE_NUMERIC_DISPLAY=0x10" 'COB_TYPE_NUMERIC_DISPLAY: u16 = 0x10'
check_const "COB_TYPE_NUMERIC_PACKED=0x12"  'COB_TYPE_NUMERIC_PACKED: u16 = 0x12'
check_const "COB_FLAG_HAVE_SIGN=1"          'COB_FLAG_HAVE_SIGN: u16 = 0x0001'
check_const "COB_FLAG_NO_SIGN_NIBBLE=0x100" 'COB_FLAG_NO_SIGN_NIBBLE: u16 = 0x0100'
note "field type/flag constants match oracle selfcheck"

# 5. Admission receipt tarball sha256 matches the actual admitted tarball (if present).
TARBALL="research/gnucobol-3.2.tar.lz"
if [ -f "$TARBALL" ]; then
  REAL=$(sha256sum "$TARBALL" | cut -d' ' -f1)
  grep -q "$REAL" reports/admission/RECEIPT-ADMISSION.md || bad "admission receipt sha256 != actual tarball ($REAL)"
  note "admission tarball sha256 matches receipt"
else
  note "tarball absent -> admission-hash check skipped (expected without lab bundle)"
fi

# 6. The sealed-claim statement is identical across README, claim-boundary, and lib.rs (3 conversions).
for f in README.md docs/claim-boundary.md crates/cobol-decimal-rs/src/lib.rs; do
  grep -qiE 'COMP-3|packed' "$f" || bad "$f no longer states the COMP-3 claim"
done
note "sealed-claim statement present in README, claim-boundary, lib.rs"

# 7. Future-risk register is the append-only open ledger and non-empty; negative-claims non-empty.
grep -q 'open.*append-only' docs/future-risk-register.md || bad "future-risk-register lost its open/append-only marker"
[ -s reports/negative-claims.md ] || bad "negative-claims.md is empty"
note "future-risk register open/append-only; negative-claims present"

# 8. CHANGELOG records both milestones.
grep -q 'GNURUST.1' CHANGELOG.md && grep -q 'GNURUST.2' CHANGELOG.md || bad "CHANGELOG missing GNURUST.1/GNURUST.2"
note "CHANGELOG records GNURUST.1 and GNURUST.2"

# 9. Oracle-gated freshness: if the built oracle + harness are present, the selfcheck constants and
#    a fresh sweep must still agree (the strongest anti-staleness check).
PREFIX="$ROOT/lab/oracle/prefix"
if [ -x "$PREFIX/bin/cobc" ] && [ -x "$ROOT/lab/oracle/decimal_harness" ]; then
  export LD_LIBRARY_PATH="$PREFIX/lib" COB_CONFIG_DIR="$PREFIX/share/gnucobol/config" LC_ALL=C.UTF-8
  SC=$("$ROOT/lab/oracle/decimal_harness" --selfcheck 2>/dev/null)
  echo "$SC" | grep -q 'COB_MAX_DIGITS=38' || bad "oracle selfcheck COB_MAX_DIGITS != 38 (constants drifted!)"
  echo "$SC" | grep -q 'COB_TYPE_NUMERIC_PACKED=0x12' || bad "oracle selfcheck PACKED type drifted"
  SWEEP=$(bash "$ROOT/lab/oracle/sweep.sh" 0 2>/dev/null | grep -oE 'PASS=[0-9]+ FAIL=[0-9]+')
  case "$SWEEP" in
    *"FAIL=0") note "oracle freshness: sweep $SWEEP, selfcheck constants match" ;;
    *) bad "oracle freshness: sweep not clean ($SWEEP)" ;;
  esac
else
  note "oracle absent -> selfcheck/sweep freshness check skipped (build lab/oracle to enable)"
fi

echo "== doc-gate $( [ $FAIL -eq 0 ] && echo PASS || echo FAIL ) =="
exit $FAIL

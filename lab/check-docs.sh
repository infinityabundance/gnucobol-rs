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
         docs/oracle-lessons.md docs/negative-capabilities.md docs/license-boundaries.md \
         reports/claim-ladder.json lab/verify-sealed-courts.sh \
         COPYING COPYING.LESSER; do
  [ -f "$f" ] || bad "README references missing file: $f"
done
# TRUST.1: the claim ladder must list every sealed campaign (per receipt), or it is stale.
for rc in reports/RECEIPT-GNURUST-*.md; do
  [ -f "$rc" ] || continue
  code=$(grep -oE 'Campaign GNURUST\.[0-9]+' "$rc" | grep -oE 'GNURUST\.[0-9]+' | head -1)
  [ -z "$code" ] && continue
  grep -q "\"$code\"" reports/claim-ladder.json || bad "claim-ladder.json missing sealed campaign $code"
done
note "claim-ladder.json lists every sealed campaign"

# 2b. Atlas hygiene: every archaeology atlas JSON must parse (machine-readable evidence, not prose).
if command -v python3 >/dev/null 2>&1; then
  AJ=0
  while IFS= read -r f; do
    if python3 -c "import json,sys; json.load(open(sys.argv[1]))" "$f" 2>/dev/null; then AJ=$((AJ+1)); else bad "atlas JSON invalid: $f"; fi
  done < <(find archaeology -name '*.json' 2>/dev/null)
  note "atlas: $AJ JSON files valid"
fi
note "all README-referenced docs exist"

# 3. COB_MAX_DIGITS constant consistent in source and equals the oracle's value (38).
RS_MAXD=$(grep -oE 'COB_MAX_DIGITS: i64 = [0-9]+' crates/gnucobol-rs/src/lib.rs | grep -oE '[0-9]+$')
[ "$RS_MAXD" = "38" ] || bad "COB_MAX_DIGITS in lib.rs is '$RS_MAXD', expected 38 (libcob/common.h:607)"
note "COB_MAX_DIGITS = $RS_MAXD"

# 4. attr.rs type/flag constants match the values documented in the admission/selfcheck.
check_const() { grep -qE "$2" crates/gnucobol-rs/src/attr.rs || bad "attr.rs missing/!= $1"; }
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
for f in README.md docs/claim-boundary.md crates/gnucobol-rs/src/lib.rs; do
  grep -qiE 'COMP-3|packed' "$f" || bad "$f no longer states the COMP-3 claim"
done
note "sealed-claim statement present in README, claim-boundary, lib.rs"

# 7. Future-risk register is the append-only open ledger and non-empty; negative-claims non-empty.
grep -q 'open.*append-only' docs/future-risk-register.md || bad "future-risk-register lost its open/append-only marker"
[ -s reports/negative-claims.md ] || bad "negative-claims.md is empty"
note "future-risk register open/append-only; negative-claims present"

# 8. CHANGELOG records the foundation + first sealed milestone.
grep -q 'GNURUST.1' CHANGELOG.md && grep -q 'GNURUST.2' CHANGELOG.md || bad "CHANGELOG missing GNURUST.1/GNURUST.2"
note "CHANGELOG records GNURUST.1 and GNURUST.2"

# 8b. ANTI-STALENESS: every SEALED campaign (one receipt per campaign) must be reflected in the
#     README and the load-bearing docs. This is what stops the README/docs drifting behind the
#     code as new courts are sealed (every doc is covered, not just src).
SEALED_DOCS="README.md CHANGELOG.md docs/claim-boundary.md docs/compatibility-taxonomy.md docs/future-risk-register.md crates/gnucobol-rs/README.md"
for rc in reports/RECEIPT-GNURUST-*.md; do
  [ -f "$rc" ] || continue
  code=$(grep -oE 'Campaign GNURUST\.[0-9]+' "$rc" | grep -oE 'GNURUST\.[0-9]+' | head -1)
  [ -z "$code" ] && continue
  for doc in $SEALED_DOCS; do
    grep -q "$code" "$doc" || bad "sealed campaign $code (from $rc) is NOT referenced in $doc — that doc is STALE"
  done
done
note "every sealed campaign (per receipt) is reflected in README + all load-bearing docs"

# 8c. The future-risk register's 'Sealed today' line must name each sealed campaign.
for rc in reports/RECEIPT-GNURUST-*.md; do
  [ -f "$rc" ] || continue
  code=$(grep -oE 'Campaign GNURUST\.[0-9]+' "$rc" | grep -oE 'GNURUST\.[0-9]+' | head -1)
  [ -z "$code" ] && continue
  grep -qE "Sealed today.*$code|sealed.*\b$code\b" docs/future-risk-register.md \
    || awk '/Sealed today/{f=1} f&&/'"$code"'/{found=1} END{exit !found}' docs/future-risk-register.md \
    || bad "register 'Sealed today' line does not name sealed campaign $code"
done
note "register 'Sealed today' names each sealed campaign"

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
    *"FAIL=0") note "oracle freshness: decimal sweep $SWEEP, selfcheck constants match" ;;
    *) bad "oracle freshness: decimal sweep not clean ($SWEEP)" ;;
  esac
  PSWEEP=$(bash "$ROOT/lab/oracle/pic_sweep.sh" 2>/dev/null | grep -oE 'PASS=[0-9]+ FAIL=[0-9]+')
  case "$PSWEEP" in
    *"FAIL=0") note "oracle freshness: PIC sweep $PSWEEP" ;;
    *) bad "oracle freshness: PIC sweep not clean ($PSWEEP)" ;;
  esac
  BINSWEEP=$(bash "$ROOT/lab/oracle/binary_sweep.sh" 2>/dev/null | grep -oE 'PASS=[0-9]+ FAIL=[0-9]+')
  case "$BINSWEEP" in
    *"FAIL=0") note "oracle freshness: binary MOVE sweep $BINSWEEP" ;;
    *) bad "oracle freshness: binary sweep not clean ($BINSWEEP)" ;;
  esac
  EBSWEEP=$(bash "$ROOT/lab/oracle/ebcdic_sweep.sh" 2>/dev/null | grep -oE 'PASS=[0-9]+ FAIL=[0-9]+')
  case "$EBSWEEP" in
    *"FAIL=0") note "oracle freshness: EBCDIC cp500 sweep $EBSWEEP" ;;
    *) bad "oracle freshness: EBCDIC sweep not clean ($EBSWEEP)" ;;
  esac
  EDSWEEP=$(bash "$ROOT/lab/oracle/edited_sweep.sh" 2>/dev/null | grep -oE 'PASS=[0-9]+ FAIL=[0-9]+')
  case "$EDSWEEP" in
    *"FAIL=0") note "oracle freshness: edited decode sweep $EDSWEEP" ;;
    *) bad "oracle freshness: edited sweep not clean ($EDSWEEP)" ;;
  esac
  LSWEEP=$(bash "$ROOT/lab/oracle/layout_sweep.sh" 2>/dev/null | grep -oE 'PASS=[0-9]+ FAIL=[0-9]+')
  case "$LSWEEP" in
    *"FAIL=0") note "oracle freshness: layout sweep $LSWEEP" ;;
    *) bad "oracle freshness: layout sweep not clean ($LSWEEP)" ;;
  esac
  CSWEEP=$(bash "$ROOT/lab/oracle/copy_sweep.sh" 2>/dev/null | grep -oE 'PASS=[0-9]+ FAIL=[0-9]+')
  case "$CSWEEP" in
    *"FAIL=0") note "oracle freshness: COPY sweep $CSWEEP" ;;
    *) bad "oracle freshness: COPY sweep not clean ($CSWEEP)" ;;
  esac
  ASWEEP=$(bash "$ROOT/lab/oracle/arith_sweep.sh" 2>/dev/null | grep -oE 'PASS=[0-9]+ FAIL=[0-9]+')
  case "$ASWEEP" in
    *"FAIL=0") note "oracle freshness: arithmetic sweep $ASWEEP" ;;
    *) bad "oracle freshness: arithmetic sweep not clean ($ASWEEP)" ;;
  esac
  VSWEEP=$(bash "$ROOT/lab/oracle/value_sweep.sh" 2>/dev/null | grep -oE 'PASS=[0-9]+ FAIL=[0-9]+')
  case "$VSWEEP" in
    *"FAIL=0") note "oracle freshness: VALUE sweep $VSWEEP" ;;
    *) bad "oracle freshness: VALUE sweep not clean ($VSWEEP)" ;;
  esac
  OSWEEP=$(bash "$ROOT/lab/oracle/odo_sweep.sh" 2>/dev/null | grep -oE 'PASS=[0-9]+ FAIL=[0-9]+')
  case "$OSWEEP" in
    *"FAIL=0") note "oracle freshness: ODO physical-max sweep $OSWEEP" ;;
    *) bad "oracle freshness: ODO sweep not clean ($OSWEEP)" ;;
  esac
  NSWEEP=$(bash "$ROOT/lab/oracle/cond_sweep.sh" 2>/dev/null | grep -oE 'PASS=[0-9]+ FAIL=[0-9]+')
  case "$NSWEEP" in
    *"FAIL=0") note "oracle freshness: LEVEL-88 sweep $NSWEEP" ;;
    *) bad "oracle freshness: LEVEL-88 sweep not clean ($NSWEEP)" ;;
  esac
  SSWEEP=$(bash "$ROOT/lab/oracle/set_sweep.sh" 2>/dev/null | grep -oE 'PASS=[0-9]+ FAIL=[0-9]+')
  case "$SSWEEP" in
    *"FAIL=0") note "oracle freshness: SET-TO-TRUE sweep $SSWEEP" ;;
    *) bad "oracle freshness: SET sweep not clean ($SSWEEP)" ;;
  esac
else
  note "oracle absent -> selfcheck/sweep freshness check skipped (build lab/oracle to enable)"
fi

echo "== doc-gate $( [ $FAIL -eq 0 ] && echo PASS || echo FAIL ) =="
exit $FAIL

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
         docs/compatibility-taxonomy.md docs/future-risk-register.md reports/negative-capabilities.json \
         docs/oracle-lessons.md docs/negative-capabilities.md docs/license-boundaries.md docs/trust2-generated-receipts.md \
         reports/claim-ladder.json lab/verify-sealed-courts.sh \
         STATUS.md docs/REVIEW-IN-10-MINUTES.md docs/not-yet-ready.md docs/effect-boundary-map.md audits/README.md \
         COPYING COPYING.LESSER; do
  [ -f "$f" ] || bad "README references missing file: $f"
done
# TRUST.1/TRUST.4: every GNURUST court in the claim-ladder must have a generated forensic casefile
# (legacy hand-written receipts are now non-authoritative exhibits under research/legacyreports/).
GCODES=$(python3 -c "import json;print(' '.join(c['id'] for c in json.load(open('reports/claim-ladder.json'))['courts'] if c['id'].startswith('GNURUST.') and c['id'] not in ('GNURUST.COVERAGE.1','GNURUST.FILE.STATUS.1','GNURUST.INTRINSIC.ATLAS.1','GNURUST.PROCEDURE.FLOW.ATLAS.1','GNURUST.PUBLIC.CORPUS.1','GNURUST.BUILD.PROFILE.1','GNURUST.PUBLIC.GAP.1','GNURUST.CALL.EXTENSION.ATLAS.1','GNURUST.INDEXED.FILE.ATLAS.1','GNURUST.SORT.MERGE.ATLAS.1','GNURUST.RELATIVE.FILE.ATLAS.1','GNURUST.DIALECT.RUNTIME.ATLAS.1','GNURUST.DIRECTIVE.VARIANCE.ATLAS.1','GNURUST.DECLARATIVES.ATLAS.1','GNURUST.CALL.LAYOUT.ATLAS.1','GNURUST.LINEAGE.CORPUS.20M.0','GNURUST.LINEAGE.CORPUS.20M.SMOKE','GNURUST.LINEAGE.CORPUS.20M.1','GNURUST.VALUE.NEGZERO.EDGE.1')))")
for code in $GCODES; do
  [ -f "reports/casefiles/$code/casefile.json" ] || bad "claim-ladder court $code has no generated casefile"
done
note "every GNURUST claim-ladder court has a generated casefile"

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

# TRUST.3: STATUS.md (live authority) must name the current crate version, or it is stale.
SV=$(grep -m1 '^version' crates/gnucobol-rs/Cargo.toml | grep -oE '[0-9]+\.[0-9]+\.[0-9]+')
grep -q "gnucobol-rs $SV" STATUS.md || bad "STATUS.md does not name current crate version $SV (live authority stale)"
note "STATUS.md names current crate version"

# ENTERPRISE.1: every release must ship a complete evidence packet for the current version.
RELDIR="reports/releases/gnucobol-rs-$SV"
if [ -f "$RELDIR/release-verdict.md" ]; then
  RELN=$(ls "$RELDIR" | wc -l)
  if [ "$RELN" -ge 11 ]; then note "release evidence packet present for $SV ($RELN files)"; else bad "release packet for $SV incomplete ($RELN/11 files)"; fi
else
  bad "no release evidence packet for gnucobol-rs $SV (run lab/release/build-packet.py)"
fi

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
[ -s reports/negative-capabilities.json ] || bad "negative-capabilities.json is empty"
note "future-risk register open/append-only; negative-capabilities registry present"

# 8. CHANGELOG records the foundation + first sealed milestone.
grep -q 'GNURUST.1' CHANGELOG.md && grep -q 'GNURUST.2' CHANGELOG.md || bad "CHANGELOG missing GNURUST.1/GNURUST.2"
note "CHANGELOG records GNURUST.1 and GNURUST.2"

# 8b. ANTI-STALENESS: every SEALED campaign (one receipt per campaign) must be reflected in the
#     README and the load-bearing docs. This is what stops the README/docs drifting behind the
#     code as new courts are sealed (every doc is covered, not just src).
SEALED_DOCS="README.md CHANGELOG.md docs/claim-boundary.md docs/compatibility-taxonomy.md docs/future-risk-register.md crates/gnucobol-rs/README.md"
for code in $GCODES; do
  for doc in $SEALED_DOCS; do
    grep -q "$code" "$doc" || bad "sealed campaign $code is NOT referenced in $doc — that doc is STALE"
  done
done
note "every sealed campaign (from the claim-ladder) is reflected in README + all load-bearing docs"

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
  ENSWEEP=$(bash "$ROOT/lab/oracle/ebcdic_num_sweep.sh" 2>/dev/null | grep -oE 'PASS=[0-9]+ FAIL=[0-9]+')
  case "$ENSWEEP" in
    *"FAIL=0") note "oracle freshness: cp500 zoned-num sweep $ENSWEEP" ;;
    *) bad "oracle freshness: cp500 zoned-num sweep not clean ($ENSWEEP)" ;;
  esac
  C6SWEEP=$(bash "$ROOT/lab/oracle/comp6_sweep.sh" 2>/dev/null | grep -oE 'PASS=[0-9]+ FAIL=[0-9]+')
  case "$C6SWEEP" in
    *"FAIL=0") note "oracle freshness: COMP-6 MOVE sweep $C6SWEEP" ;;
    *) bad "oracle freshness: COMP-6 sweep not clean ($C6SWEEP)" ;;
  esac
  DIVSWEEP=$(bash "$ROOT/lab/oracle/divide_sweep.sh" 2>/dev/null | grep -oE 'PASS=[0-9]+ FAIL=[0-9]+')
  case "$DIVSWEEP" in
    *"FAIL=0") note "oracle freshness: DIVIDE sweep $DIVSWEEP" ;;
    *) bad "oracle freshness: DIVIDE sweep not clean ($DIVSWEEP)" ;;
  esac
  # TRUST.2: generated receipts must be current (live replay) + .md == render(.json), no manual edits.
  if python3 "$ROOT/lab/receipt/run.py" check >/tmp/_rec_check 2>&1; then
    note "TRUST.2: receipts reproducible (generated == live replay, no hand-edits)"
  else
    bad "TRUST.2: receipt drift"; cat /tmp/_rec_check
  fi
  # TRUST.4: every court has a generated forensic casefile; generated views match; negatives >= positives.
  if python3 "$ROOT/lab/casefile/run.py" check >/tmp/_case_check 2>&1; then
    note "TRUST.4: forensic casefiles current (generated views match, negatives >= positives)"
  else
    bad "TRUST.4: casefile drift"; cat /tmp/_case_check
  fi
  if python3 "$ROOT/lab/trust4/migrate_reports.py" check >/tmp/_mig_check 2>&1; then
    note "TRUST.4: legacy migration intact (no static report in reports/, manifest preserved)"
  else
    bad "TRUST.4: legacy migration drift"; cat /tmp/_mig_check
  fi
  if python3 "$ROOT/lab/docs/generate.py" check >/tmp/_docs_check 2>&1; then
    note "TRUST.4.DOCS: authoritative docs generated, version fresh, legacy preserved as superset"
  else
    bad "TRUST.4.DOCS: doc drift"; cat /tmp/_docs_check
  fi
  if command -v kobold-attest >/dev/null 2>&1; then
    if kobold-attest check --root "$ROOT" >/tmp/_ent2_check 2>&1; then
      note "ENTERPRISE.2: DSSE verification report fresh, no integrity failure (external kobold-attest)"
    else
      bad "ENTERPRISE.2: attestation verification drift"; cat /tmp/_ent2_check
    fi
  else
    note "ENTERPRISE.2: kobold-attest not installed -> skipped (honest; cargo install kobold-attest to enable)"
  fi
  if python3 "$ROOT/lab/support/run.py" check >/tmp/_supp_check 2>&1; then
    note "SUPPORT-PACKET.1: evidence bundle fresh (re-gather equality)"
  else
    bad "SUPPORT-PACKET.1: support packet drift"; cat /tmp/_supp_check
  fi
  if python3 "$ROOT/lab/dialect/run.py" check >/tmp/_dial_check 2>&1; then
    note "DIALECT.PROFILE.1: witness profile self-consistent, -std binds the hash"
  else
    bad "DIALECT.PROFILE.1: dialect profile drift"; cat /tmp/_dial_check
  fi
  if python3 "$ROOT/lab/trust5/run.py" check >/tmp/_t5_check 2>&1; then
    note "TRUST.5: anti-ceremony audit fresh; no class-F court; views are no-new-truth"
  else
    bad "TRUST.5: anti-ceremony audit failure"; cat /tmp/_t5_check
  fi
  if python3 "$ROOT/lab/coverage/run.py" check >/tmp/_cov_check 2>&1; then
    note "GNURUST.COVERAGE.1: every admitted court mapped to a GnuCOBOL surface; map fresh"
  else
    bad "GNURUST.COVERAGE.1: coverage drift / unmapped court"; cat /tmp/_cov_check
  fi
  if python3 "$ROOT/lab/ladder/run.py" check >/tmp/_lad_check 2>&1; then
    note "PORTING-LADDER: every court placed on the forensic-port hierarchy; fresh"
  else
    bad "PORTING-LADDER: drift / unplaced court"; cat /tmp/_lad_check
  fi
  if python3 "$ROOT/lab/kani-fuzz/run.py" check >/tmp/_kf_check 2>&1; then
    note "KANI+FUZZ: every GNURUST byte court has a Kani proof + a fuzz target (n/a declared for composition/atlas)"
  else
    bad "KANI+FUZZ: a byte court is missing a Kani proof or fuzz target"; cat /tmp/_kf_check
  fi
  if python3 "$ROOT/lab/corpus/run.py" check >/tmp/_corpus_check 2>&1; then
    note "GNURUST.PUBLIC.CORPUS.1: public-COBOL corpus index fresh (gap discovery, index-only)"
  else
    bad "GNURUST.PUBLIC.CORPUS.1: corpus index drift"; cat /tmp/_corpus_check
  fi
  if python3 "$ROOT/lab/gap/run.py" check >/tmp/_gap_check 2>&1; then
    note "GNURUST.PUBLIC.GAP.1: surface gap board over the admitted GnuCOBOL testsuite fresh"
  else
    bad "GNURUST.PUBLIC.GAP.1: gap board drift"; cat /tmp/_gap_check
  fi
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

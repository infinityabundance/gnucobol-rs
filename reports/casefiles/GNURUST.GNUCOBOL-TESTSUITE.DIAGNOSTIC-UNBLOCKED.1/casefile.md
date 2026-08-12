<!-- DO NOT EDIT BY HAND. Generated from casefile.json by kobold-courts.
     Evidence of record: casefile.json. Portable attestations: sarif.json, intoto-statement.json, dsse-envelope.json. -->
# Forensic case file — GNURUST.GNUCOBOL-TESTSUITE.DIAGNOSTIC-UNBLOCKED.1 (court-casefile)

**Verdict: PASS** · lab/gnucobol-testsuite/run-diagnostic-unblocked-docker.sh (one-command two-pass replay) + lab/docker/gnucobol-testsuite-diag-unblocked + reports/gnucobol-testsuite/diagnostic-unblocked/* · crate `gnucobol-rs` 0.8.57

- **Oracle:** the ADMITTED GnuCOBOL 3.2 in-tree build (identical configuration to the pristine testsuite lane); the pristine lane + its evidence are NEVER touched
- **Byte domain(s):** diagnostic-ignore.patch + transformations.json + tree-manifest.json + semantic-reachability.{json,md} + pristine-vs-diagnostic-unblocked.{json,md} + corpus-cross-check.{json,md} + both passes' raw testsuite logs and per-group evidence under reports/gnucobol-testsuite/diagnostic-unblocked/raw/
- **Replay:** `bash lab/gnucobol-testsuite/run-diagnostic-unblocked-docker.sh`
- **Authority:** STATUS.md · receipt_status: current

## Positive claims (5)
- the admitted stable suite source is transformed by gnurust-diag-unblocked-transform-v1 (deciding ONLY from upstream test structure, never candidate behaviour): 621 diagnostic expectations (stdout 1 / stderr 620) across 404 groups become Autotest 'ignore' while commands, exit statuses, COBOL source, runtime output, generated-file expectations, environment, ordering and skip/xfail stay identical
- the real suite is regenerated with the upstream mechanism (make -C tests testsuite, autom4te) and run twice in fresh isolated containers with the oracle and the candidate
- the patch regenerates byte-identically (sha256 712a0b172021c7ec650c6d97e348b465efd355699e309e956d95f99a2dc69230), AT_SETUP 1344==1344 and AT_CHECK 3422==3422 reconcile, the group index is identical across lanes (1282 groups), and the generated testsuite (sha256 9ef79a4baa6ca98386858a42b486e8e8b67d8d05bb0a73b0b17a0148b0e77049) is two-pass deterministic
- measured semantic reachability: 377 of 404 affected groups progress further, 140 later semantic checks become reachable (27 runtime executions, 12 matched, 17 newly-exposed compile/check failures, 15 runtime failures)
- the oracle itself is diagnostic-text-gated in 4 always-xfail groups (116/323/336/350): unblocked oracle XPASS 4 vs pristine 0, proving suite-vs-3.2 diagnostic drift independent of the candidate

## Negative claims (7) — negative capability is the trust surface
- NOT a GnuCOBOL test-suite parity claim and NOT 'tests passed after fixing the suite': the pristine suite remains the compatibility authority
- ignored diagnostic text is NOT diagnostic compatibility
- no candidate parser-success claim for steps validated only by the oracle
- no new language/runtime compatibility claim from diagnostic-only steps
- no weakening of exit-status, runtime-output or generated-file semantics
- no self-validating evidence (the transformer never sees candidate behaviour)
- lie prevented: 'the candidate passes the GnuCOBOL testsuite once diagnostics are ignored' is the lie this prevents -- the lane measures semantic REACHABILITY, not passes, and the 4 oracle XPASS groups prove the suite's own diagnostic text drifts from GnuCOBOL 3.2

## Damage if overclaimed
counting diagnostic-unblocked steps as pristine passes would certify coverage the candidate does not have and misstate the lane's purpose

> Generated forensic evidence (TRUST.4). The binding record is `casefile.json`; this `.md` is a rendering.
> Portable attestations: `sarif.json` (findings), `intoto-statement.json` (provenance), `dsse-envelope.json`.

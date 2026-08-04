<!-- DO NOT EDIT BY HAND. Generated from casefile.json by kobold-courts.
     Evidence of record: casefile.json. Portable attestations: sarif.json, intoto-statement.json, dsse-envelope.json. -->
# Forensic case file — GNURUST.GNUCOBOL-TESTSUITE.1 (court-casefile)

**Verdict: PASS** · gnucobol-3.2.tar.lz (admitted, sha256-pinned) + lab/gnucobol-testsuite/run-docker.sh (one-command replay) + lab/docker/gnucobol-testsuite · crate `gnucobol-rs` 0.8.54

- **Oracle:** the ADMITTED GnuCOBOL 3.2 in-tree build (never a distribution package), configured identically in every tree
- **Byte domain(s):** admitted source identity + fresh in-tree build + the generated testsuite.log + per-group logs + the invocation census (argv preserved)
- **Replay:** `bash lab/gnucobol-testsuite/run-docker.sh`
- **Authority:** STATUS.md · receipt_status: current

## Positive claims (1)
- the admitted gnucobol-3.2 source (sha256 8ecc77d0...) is built fresh in-tree per pass with a stock configuration (no -fpermissive, no compat -Wno-*), and the generated Autotest suite is run with the REAL admitted cobc via `make check TESTSUITEFLAGS=--jobs=12`, producing an oracle baseline of 1242 pass / 9 skip / 31 xfail / 0 fail in this environment, with a full invocation census (argv boundaries preserved: 0 cobc/cobcrun invocations) and the raw testsuite.log + per-group logs preserved

## Negative claims (4) — negative capability is the trust surface
- no claim about gnucobol-rs (that is TESTSUITE.2/.3)
- the baseline measures THIS build and environment, not upstream
- oracle-side skips/xfails are the suite's own declared conditions
- lie prevented: an oracle-side pass/fail is evidence about THIS admitted build, never a claim about upstream quality

## Damage if overclaimed
presenting the baseline as a certification would certify a compiler build the suite was not designed to certify

> Generated forensic evidence (TRUST.4). The binding record is `casefile.json`; this `.md` is a rendering.
> Portable attestations: `sarif.json` (findings), `intoto-statement.json` (provenance), `dsse-envelope.json`.

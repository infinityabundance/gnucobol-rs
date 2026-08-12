<!-- DO NOT EDIT BY HAND. Generated from casefile.json by kobold-courts.
     Evidence of record: casefile.json. Portable attestations: sarif.json, intoto-statement.json, dsse-envelope.json. -->
# Forensic case file — GNURUST.CCVS85.2 (court-casefile)

**Verdict: PASS** · lab/corpus/ccvs85/newcob.val.Z (committed, hash-pinned) + lab/docker/ccvs85 + lab/ccvs85/run-docker.sh (one-command replay) · crate `gnucobol-rs` 0.8.56

- **Oracle:** pinned GnuCOBOL 3.2.0 built in-container from the admitted source tarball (sha256 8ecc77d0...); never a distribution package
- **Byte domain(s):** corpus split + materialized unit bytes (hashes) + per-unit cobc compile/run outcomes, report bytes, and parsed CCVS85 verdict counts
- **Replay:** `bash lab/ccvs85/run-docker.sh`
- **Authority:** STATUS.md · receipt_status: current

## Positive claims (1)
- the admitted CCVS85 v4.0 corpus is decompressed (hash-verified vs GNURUST.CCVS85.1), split into 512 units, materialized to stable files (SHA-256 per unit, fixed-format columns and col-73-80 tags byte-preserved), copybook/library/data dependencies derived, the documented site-adaptation applied (X-cards, deleted/obsolete markers, RERUN card, CCVS1 counter unification), and every applicable unit compiled and (where possible) executed by the pinned GnuCOBOL 3.2 oracle with per-unit compile/run/report/verdict evidence

## Negative claims (6) — negative capability is the trust surface
- no claim about gnucobol-rs (that is CCVS85.3)
- oracle acceptance/rejection is specific to this GnuCOBOL 3.2 build
- no NIST certification
- no COBOL-85 conformance claim
- CLBRY/DATA* units are support units, not executable tests
- lie prevented: an oracle compile rejection is only evidence about THIS GnuCOBOL build, never about the source's validity under other implementations

## Damage if overclaimed
claiming oracle acceptance as conformance would certify a 1993 validation corpus against a compiler the suite was never designed to certify

> Generated forensic evidence (TRUST.4). The binding record is `casefile.json`; this `.md` is a rendering.
> Portable attestations: `sarif.json` (findings), `intoto-statement.json` (provenance), `dsse-envelope.json`.

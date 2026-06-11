<!-- DO NOT EDIT BY HAND. Generated from casefile.json by kobold-courts.
     Evidence of record: casefile.json. Portable attestations: sarif.json, intoto-statement.json, dsse-envelope.json. -->
# Forensic case file — GNURUST.COVERAGE.1 (court-casefile)

**Verdict: PASS** · lab/coverage/run.py generate/check -> reports/gnurust-coverage.{json,md} (27 surfaces; every admitted court mapped) · crate `gnucobol-rs` governance (gnucobol-rs lab)

- **Oracle:** the admitted GnuCOBOL 3.2 source + the claim-ladder (re-derive equality + mapping completeness)
- **Byte domain(s):** GnuCOBOL source surfaces -> {status, court/refusal/future, risk} with admitted-court mapping enforced
- **Replay:** `the admitted GnuCOBOL 3.2 source + the claim-ladder (re-derive equality + mapping completeness)`
- **Authority:** STATUS.md · receipt_status: no-trust2-receipt

## Positive claims (3)
- a generated map of 27 declared GnuCOBOL semantic surfaces, each bound to admitted GnuCOBOL 3.2 source module(s) (lab/admit/gnucobol-3.2) and to its current court(s) / refusal / future campaign with a status (sealed/observed/negative/missing) and a risk-if-unported
- the gate FAILS if an admitted claim-ladder GNURUST/atlas court is not mapped to a surface, if a surface declares a court not in the claim-ladder, if a sealed surface has no court, if a surface binds to a missing source module, or if the map is stale
- current honest state: 11 sealed (the data-representation spine), 1 observed, 7 refused, 8 MISSING

## Negative claims (6) — negative capability is the trust surface
- that GnuCOBOL is ported (sealed is only the data-representation spine
- file I/O, intrinsics, Procedure Division mostly MISSING)
- that the surface list is exhaustive
- that a status is a quality score
- new truth
- lie prevented: 'the data-representation courts are green, so GnuCOBOL is mostly ported' -- COVERAGE.1 shows 8 MISSING surfaces (file I/O, file status, intrinsics, INITIALIZE, INSPECT, STRING/UNSTRING, ACCEPT/DISPLAY, Procedure Division flow) and refuses the completeness reading

## Damage if overclaimed
presenting the fixed-record spine as a complete GnuCOBOL port would send migrations into unported file-I/O, intrinsic, and control-flow behavior with no evidence

> Generated forensic evidence (TRUST.4). The binding record is `casefile.json`; this `.md` is a rendering.
> Portable attestations: `sarif.json` (findings), `intoto-statement.json` (provenance), `dsse-envelope.json`.

<!-- DO NOT EDIT BY HAND. Generated from casefile.json by lab/casefile/run.py.
     Evidence of record: casefile.json. Portable attestations: sarif.json, intoto-statement.json, dsse-envelope.json. -->
# Forensic case file — GNURUST.PUBLIC.GAP.1 (court-casefile)

**Verdict: PASS** · lab/gap/run.py 29 files -> reports/public-gap-board.json · crate `gnucobol-rs` meta board (gnucobol-rs lab)

- **Oracle:** GnuCOBOL upstream testsuite (admitted in lab/admit)
- **Byte domain(s):** verb/feature frequency scan of the admitted testsuite -> sealed/observed/refused/missing surface board
- **Replay:** `GnuCOBOL upstream testsuite (admitted in lab/admit)`
- **Authority:** STATUS.md · receipt_status: no-trust2-receipt

## Positive claims (1)
- a surface-frequency GAP BOARD over the ADMITTED GnuCOBOL 3.2.0 upstream testsuite (29 run/syn .at files, ZERO network -- the authoritative corpus #1). Classifies each COBOL surface as sealed (13) / observed (2) / refused (4) / MISSING (7) against the court map, and ranks the missing-court board by occurrence: CALL/linkage (959x -- the LARGEST gap), START/DELETE/indexed-org (indexed files), SORT/MERGE (145x), SEARCH (74x), relative files. Turns we-sealed-courts into we-know-our-blind-spots-against-the-authoritative-corpus. The companion to GNURUST.PUBLIC.CORPUS.1, run on corpus #1

## Negative claims (6) — negative capability is the trust surface
- compilation/execution/parity over any test
- that a verb's presence proves a court is needed for full behavior
- that the missing set is a roadmap commitment (candidates)
- multi-corpus coverage (one corpus)
- exhaustiveness of surfaces
- lie prevented: we cover the GnuCOBOL testsuite -- NO: this is a VERB-PRESENCE scan that finds 7 MISSING surfaces (CALL the biggest at 959x); it compiles/runs/proves NOTHING and the missing-court board is a candidate set, not a roadmap commitment

## Damage if overclaimed
presenting a surface scan as test-pass / parity coverage fabricates execution evidence the project has not produced

> Generated forensic evidence (TRUST.4). The binding record is `casefile.json`; this `.md` is a rendering.
> Portable attestations: `sarif.json` (findings), `intoto-statement.json` (provenance), `dsse-envelope.json`.

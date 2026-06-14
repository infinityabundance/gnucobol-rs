<!-- DO NOT EDIT BY HAND. Generated from casefile.json by kobold-courts.
     Evidence of record: casefile.json. Portable attestations: sarif.json, intoto-statement.json, dsse-envelope.json. -->
# Forensic case file — GNURUST.LINEAGE.CORPUS.20M.1 (court-casefile)

**Verdict: PASS** · 4000000/4000000 pass, 0 fail · crate `gnucobol-rs` 0.7.60

- **Oracle:** real cobc/libcob compile+run over 4M generated COBOL witnesses (GNURUST.BUILD.PROFILE.1 profile)
- **Byte domain(s):** completed 4M real-cobc witness run: storage differential + directive variant lineage; evidence-of-record = reports/lineage20m/full-run-seal.json (root-of-roots binds the regenerable shards)
- **Replay:** `bash lab/oracle/lineage_fullrun_sweep.sh`
- **Authority:** STATUS.md · receipt_status: current

## Positive claims (2)
- ran the FULL 20M-index lineage corpus to completion under the real pinned GnuCOBOL 3.2 oracle: 4,000,000 DISTINCT real-cobc COBOL witnesses compiled+run+classified (the other ~16M of the 20M index space are the not-yet-emitting families, logged as dropped buckets, not silently covered), and the run PASSED ITS OWN seal-grade gate -- untriaged=0 across all 4M, Merkle root-of-roots 9673654d binds all 1024 shards, stratified replay byte-identical, parallel-vs-serial harness isolation PASS, and the finding-collapse invariant holds (aliases=0, witness_hits==reddening_covered)
- the run scaled the single confirmed harvested finding (COMP-3 integer VALUE -0: cobc 0c vs gnucobol-rs 0d) to 1,056 witness hits collapsing to ONE finding (GNURUST.VALUE.NEGZERO.EDGE.1 candidate) with ZERO new untriaged surprises across 4M witnesses

## Negative claims (10) — negative capability is the trust surface
- full GnuCOBOL parity / standard conformance
- all dialects
- public-corpus execution parity
- business correctness
- compiler replacement
- auto build-profile detection
- the negative-zero finding being FIXED (candidate GNURUST.VALUE.NEGZERO.EDGE.1, not patched)
- the not-yet-emitting families (4M of 20M = storage+directive only
- the other 13 families are dropped buckets, NOT covered)
- lie prevented: 'we ran 20M COBOL tests' would overclaim -- the 20M INDEX SPACE was swept but the v0 engine EMITS 4M real-cobc witnesses (storage+directive); the other 16M are honestly logged as dropped buckets, NOT counted as witnesses; every one of the 4M is Merkle-bound + replay-deterministic + classified, with the one divergence shown (1056 hits -> 1 finding), not hidden; the run is sealed by its OWN gate, the gate's load-bearingness proven by .SMOKE's 4/4 injected-fault tests (same gate code)

## Damage if overclaimed
treating 20M index sweep as 20M witnesses inflates the corpus 5x; treating the run as parity, or the negzero finding as fixed, overclaims; the 13 dropped families are future work, not coverage

> Generated forensic evidence (TRUST.4). The binding record is `casefile.json`; this `.md` is a rendering.
> Portable attestations: `sarif.json` (findings), `intoto-statement.json` (provenance), `dsse-envelope.json`.

<!-- DO NOT EDIT BY HAND. Generated from casefile.json by kobold-courts.
     Evidence of record: casefile.json. Portable attestations: sarif.json, intoto-statement.json, dsse-envelope.json. -->
# Forensic case file — GNURUST.LINEAGE.CORPUS.20M.SMOKE (court-casefile)

**Verdict: PASS** · 200000/200000 pass, 0 fail · crate `gnucobol-rs` 0.7.35

- **Oracle:** real cobc/libcob compile+run over generated COBOL witnesses (GNURUST.BUILD.PROFILE.1 profile)
- **Byte domain(s):** 200K real-cobc witnesses: storage differential (oracle-default vs value_image) + directive variant (default vs -fbinary-* lineage delta)
- **Replay:** `bash lab/oracle/lineage_corpus_sweep.sh`
- **Authority:** STATUS.md · receipt_status: current

## Positive claims (2)
- ran a 200K-witness pilot burn of DISTINCT generated COBOL programs COMPILED+RUN under the real pinned GnuCOBOL 3.2 oracle (NOT Rust fuzz iterations) and classified them by the three-way differential into a behavioral-cluster atlas, gate-GREEN: 0 untriaged, every reddening row covered by a CONFIRMED shrunk finding, Merkle root-of-roots intact, stratified replay byte-identical, harness isolation proven, and the four injected-fault tests (generator-manifest mutation / build-profile mutation / receipt tamper / finding-removal) each FORCE red
- the burn HARVESTED a real oracle/Rust VALUE negative-zero divergence (S9(n) COMP-3 VALUE -0: cobc 0c vs gnucobol-rs 0d) -- shrunk to a 1-field minimal reproducer, classified confirmed_harvested, filed as the GNURUST.VALUE.NEGZERO.EDGE.1 candidate, and NOT hidden (it stays visible in the receipt as reddening_covered_by_confirmed_findings)

## Negative claims (9) — negative capability is the trust surface
- the full 20M run (this is a 200K PILOT
- the full 20M is GNURUST.LINEAGE.CORPUS.20M.1, launched detached and sealed progressively)
- full GnuCOBOL parity / standard conformance
- all dialects
- public-corpus execution parity
- business correctness
- the negative-zero finding being FIXED (filed as a candidate edge court, a blanket fix was reverted as shape-sensitive)
- families not yet emitting (storage + directive only in v0)
- lie prevented: 'we ran 20M COBOL tests' is the cheap claim -- this is a 200K REAL-cobc pilot whose every mismatch is shrunk+filed and whose evidence is Merkle-bound + replay-deterministic + injected-fault-guarded; the 200K counts real compiled COBOL witnesses, not Rust iterations, and the one harvested divergence is shown, not hidden

## Damage if overclaimed
treating the 200K pilot as the full 20M, or as a parity proof, or the negative-zero finding as fixed, each overclaims

> Generated forensic evidence (TRUST.4). The binding record is `casefile.json`; this `.md` is a rendering.
> Portable attestations: `sarif.json` (findings), `intoto-statement.json` (provenance), `dsse-envelope.json`.

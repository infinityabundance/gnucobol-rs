<!-- DO NOT EDIT BY HAND. Generated from casefile.json by kobold-courts.
     Evidence of record: casefile.json. Portable attestations: sarif.json, intoto-statement.json, dsse-envelope.json. -->
# Forensic case file — GNURUST.LINEAGE.CORPUS.20M.0 (court-casefile)

**Verdict: PASS** · 10/10 pass, 0 fail · crate `gnucobol-rs` 0.7.58

- **Oracle:** real cobc/libcob compile+run over generated COBOL witnesses (GNURUST.BUILD.PROFILE.1 profile)
- **Byte domain(s):** meta-engine: deterministic generate + canonical hashing + Merkle + stratified replay + parallel isolation + shrink/file findings path
- **Replay:** `bash lab/oracle/lineage_engine_sweep.sh`
- **Authority:** STATUS.md · receipt_status: current

## Positive claims (3)
- built the deterministic COBOL lineage atlas ENGINE: witnesses are GENERATED FROM SEED (the generator IS the corpus -- regenerable from generator_version+shard_seed+index, no .cob stored), the schema is canonical-hashed, a Merkle root-of-roots binds every shard, a STRATIFIED replay regenerates+re-runs sampled+nonpass witnesses to byte-identical leaves, parallel-vs-serial harness isolation is proven (LINEAGE20M.HARNESS.ISOLATION.1, per-call COB_TMPDIR), and the shrink->file->confirm findings path collapses raw mismatches to minimal reproducers
- the plan budgets sum to exactly 20M across 15 families each owned by a court/atlas/refusal
- verified by the engine self-test + the seal-grade gate (Merkle + stratified replay + isolation + findings-completeness, all GREEN)

## Negative claims (10) — negative capability is the trust surface
- any BURN result (the 200K burn is GNURUST.LINEAGE.CORPUS.20M.SMOKE
- the full 20M is GNURUST.LINEAGE.CORPUS.20M.1, launched detached)
- full GnuCOBOL parity / standard conformance
- all dialects
- public-corpus execution parity
- business correctness
- compiler replacement
- automatic build-profile detection
- the families not yet emitting (v0 emits storage + directive of 15 planned, the rest logged as dropped buckets)
- lie prevented: an engine that 'works' is not the same as one whose evidence is REPLAYABLE -- this engine's rows are Merkle-bound and byte-identical on stratified replay, and a parallel-compile race that poisoned replay was caught BY the replay gate before any scale; the engine seals NO burn results, only the machinery

## Damage if overclaimed
treating the lineage corpus as a parity/conformance proof, or the negative-zero finding as a fixed bug, or the engine as covering all 15 families, would each overclaim; the corpus reveals the oracle's observed shape and files divergences, it does not certify gnucobol-rs against all of GnuCOBOL

> Generated forensic evidence (TRUST.4). The binding record is `casefile.json`; this `.md` is a rendering.
> Portable attestations: `sarif.json` (findings), `intoto-statement.json` (provenance), `dsse-envelope.json`.

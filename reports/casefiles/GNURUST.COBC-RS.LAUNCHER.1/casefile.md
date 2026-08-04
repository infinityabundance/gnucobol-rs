<!-- DO NOT EDIT BY HAND. Generated from casefile.json by kobold-courts.
     Evidence of record: casefile.json. Portable attestations: sarif.json, intoto-statement.json, dsse-envelope.json. -->
# Forensic case file — GNURUST.COBC-RS.LAUNCHER.1 (court-casefile)

**Verdict: PASS** · crates/cobc-rs/tests/cli.rs (launcher_runs_program_and_propagates_exit_status, manifest_self_hash_refuses_tampering, manifest_hash_is_self_consistent_and_stable) · crate `gnucobol-rs` 0.8.54

- **Oracle:** n/a (artifact-model contract; cross-checked by the suite's executable lifecycle expectations)
- **Byte domain(s):** the launcher manifest bytes + their self-hash + the run behavior they control
- **Replay:** `n/a (artifact-model contract; cross-checked by the suite's executable lifecycle expectations)`
- **Authority:** STATUS.md · receipt_status: no-trust2-receipt

## Positive claims (5)
- cobc-rs -x/-m writes a launcher symlink
- JSON manifest + expanded source (gnucobol-rs-launch-manifest-v1) with a self-hash that the launcher VERIFIES at run time (tampered manifests refuse to run, exit 2)
- the artifact is an interpreter launch manifest, explicitly NOT a native COBOL executable
- RETURN-CODE -> exit status, program args tolerated, atomic writes
- integration tests cover the lifecycle including the tamper guard

## Negative claims (7) — negative capability is the trust surface
- the launcher is NOT a native-code-compiled COBOL executable
- no shell-script generation
- no claim that the launcher's runtime matches the oracle outside the sealed corpus
- no claim that a launcher survives a mismatch between the manifest dialect/conf and the environment it runs in
- no claim that RETURN-CODE propagation covers every cobc exit-code convention
- the artifact lifecycle is proven for the admitted suite's usage shapes only
- lie prevented: 'cobc-rs produces native executables' is the lie this prevents

## Damage if overclaimed
calling the launcher a native executable would misrepresent an interpreter as codegen

> Generated forensic evidence (TRUST.4). The binding record is `casefile.json`; this `.md` is a rendering.
> Portable attestations: `sarif.json` (findings), `intoto-statement.json` (provenance), `dsse-envelope.json`.

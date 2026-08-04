<!-- DO NOT EDIT BY HAND. Generated from casefile.json by kobold-courts.
     Evidence of record: casefile.json. Portable attestations: sarif.json, intoto-statement.json, dsse-envelope.json. -->
# Forensic case file — GNURUST.COBC-RS.NATIVE-MODE-BOUNDARY.1 (court-casefile)

**Verdict: PASS** · docs/generated/cobc-rs-option-compatibility.md + reports/gnucobol-testsuite/unsupported-option-census.{json,md} · crate `gnucobol-rs` 0.8.55

- **Oracle:** the admitted suite's native-mode tests (used_binaries.at -C/-S/-c)
- **Byte domain(s):** wrapper option-policy registry + per-test classifications + the invocation census
- **Replay:** `bash lab/oracle/gnucobol-testsuite/run-docker.sh`
- **Authority:** STATUS.md · receipt_status: current

## Positive claims (3)
- the option-policy registry rejects native-code modes rather than faking artifacts
- adapter-compatible workflows (later executable semantics only) are mapped onto candidate manifests with the translation recorded
- native-artifact tests (generated C/assembly/object structure, symbols, relocations, linker behavior) remain a typed boundary with no support claim

## Negative claims (4) — negative capability is the trust surface
- no native code generation
- no C/assembly/object emission
- no linker behavior
- lie prevented: '-c produces an object file' is the lie this prevents

## Damage if overclaimed
counterfeiting native artifacts would misrepresent the interpreter as a code generator

> Generated forensic evidence (TRUST.4). The binding record is `casefile.json`; this `.md` is a rendering.
> Portable attestations: `sarif.json` (findings), `intoto-statement.json` (provenance), `dsse-envelope.json`.

<!-- DO NOT EDIT BY HAND. Generated from casefile.json by kobold-courts.
     Evidence of record: casefile.json. Portable attestations: sarif.json, intoto-statement.json, dsse-envelope.json. -->
# Forensic case file — NIST-STYLE-FIXTURE-FORMAT.1 (court-casefile)

**Verdict: PASS** · tests/fixture.rs (4: positive replays+matches, negative first-class, wrong-expectation fails, risk-without-non-claims fails + changed-record-changes-hash) · crate `kobold-data-shim` kobold 0.7.0

- **Oracle:** the named court's real outcome (replay), compared to the author-declared expected
- **Byte domain(s):** declared fixture + real court replay -> matched/mismatch evidence (kobold-fixture-v1)
- **Replay:** `the named court's real outcome (replay), compared to the author-declared expected`
- **Authority:** STATUS.md · receipt_status: no-trust2-receipt

## Positive claims (5)
- a small named fixture format (kobold-fixture-v1) declares input bytes + copybook/profile + expected verdict + expected findings + expected non-claims + input hashes
- replay_fixture runs the NAMED court for real, captures its ACTUAL outcome, and compares actual-vs-expected so a wrong expected verdict or finding genuinely fails (matched:false)
- negative (fail-closed) fixtures are first-class and a risk-bearing fixture with no non-claims is rejected
- a changed input record changes record_sha256
- nist_conformance is hard-false

## Negative claims (7) — negative capability is the trust surface
- NIST COBOL conformance
- language-suite parity
- certification
- that expected output is oracle authority
- that a pass is business truth
- that fixture bytes are customer data
- lie prevented: 'these fixtures prove NIST/COBOL conformance' -- NIST-STYLE-FIXTURE-FORMAT.1 only proves that one declared input replays to the author's expected verdict/findings/non-claims; nist_conformance:false

## Damage if overclaimed
a fixture pack sold as NIST/language conformance or certification manufactures standards assurance the project does not provide

> Generated forensic evidence (TRUST.4). The binding record is `casefile.json`; this `.md` is a rendering.
> Portable attestations: `sarif.json` (findings), `intoto-statement.json` (provenance), `dsse-envelope.json`.

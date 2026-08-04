<!-- DO NOT EDIT BY HAND. Generated from casefile.json by kobold-courts.
     Evidence of record: casefile.json. Portable attestations: sarif.json, intoto-statement.json, dsse-envelope.json. -->
# Forensic case file — GNURUST.METHODOLOGY.LIBCOB.1 (court-casefile)

**Verdict: PASS** · docs/methodology/libcob-rust-port.md + reports/methodology/libcob-port-provenance.json + LIBCOB-PARITY.md · crate `gnucobol-rs` 0.8.54

- **Oracle:** n/a (documentation + machine records, cross-checked against the parity tooling)
- **Byte domain(s):** the provenance records + parity reports
- **Replay:** `n/a (documentation + machine records, cross-checked against the parity tooling)`
- **Authority:** STATUS.md · receipt_status: no-trust2-receipt

## Positive claims (2)
- docs/methodology/libcob-rust-port.md + reports/methodology/libcob-port-provenance.json record the admitted source identity (gnucobol-3.2, sha256 8ecc77d0...), the 13 in-scope libcob files, the statement-by-statement translation method with upstream line citations, the 100%% symbol parity (LIBCOB-PARITY.md), the LGPL-3.0-or-later inheritance, and the explicit non-claims
- the tooling history is recorded as UNKNOWN where the committed record does not show it

## Negative claims (3) — negative capability is the trust surface
- not every libcob function is behaviorally proven byte-equal
- only the sealed corpus is
- lie prevented: 'the runtime is a clean-room reimplementation' is the lie this prevents

## Damage if overclaimed
mislabeling a derivative as clean-room would be a licensing/provenance error

> Generated forensic evidence (TRUST.4). The binding record is `casefile.json`; this `.md` is a rendering.
> Portable attestations: `sarif.json` (findings), `intoto-statement.json` (provenance), `dsse-envelope.json`.

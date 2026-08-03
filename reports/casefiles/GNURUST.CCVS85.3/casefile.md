<!-- DO NOT EDIT BY HAND. Generated from casefile.json by kobold-courts.
     Evidence of record: casefile.json. Portable attestations: sarif.json, intoto-statement.json, dsse-envelope.json. -->
# Forensic case file — GNURUST.CCVS85.3 (court-casefile)

**Verdict: PASS** · the CCVS85.2 materialized units (same bytes, same site adaptation) + crates/gnucobol-rs/examples/cobrun.rs · crate `gnucobol-rs` 0.8.50

- **Oracle:** the gnucobol-rs cobrun front-end itself (this gate measures it, it is not an oracle gate)
- **Byte domain(s):** per-unit cobrun prepare/run/timeout outcomes + raw stdout/stderr + the no-delegation proof (candidate_phase_isolated, cobrun_links_no_libcob)
- **Replay:** `bash lab/ccvs85/run-docker.sh`
- **Authority:** STATUS.md · receipt_status: current

## Positive claims (1)
- every applicable materialized CCVS85 unit is run through the current native-Rust front-end/runtime (the canonical cobrun path) with raw stdout/stderr preserved, timeouts and fail-closed rejections recorded, and the mechanical no-delegation proof (oracle prefix renamed away, cobc removed from PATH, cobrun's dynamic dependencies scanned: zero libcob/cobc linkage)

## Negative claims (5) — negative capability is the trust surface
- no suite-pass claim
- candidate rejection is fail-closed and is not conformance evidence
- no claim that acceptance implies COBOL-85 conformance
- no claim about cobc/compiled-code equivalence
- lie prevented: an unsupported-construct rejection recorded as a pass would fabricate conformance

## Damage if overclaimed
presenting candidate acceptance as COBOL-85 support would mislead every migration decision built on this court

> Generated forensic evidence (TRUST.4). The binding record is `casefile.json`; this `.md` is a rendering.
> Portable attestations: `sarif.json` (findings), `intoto-statement.json` (provenance), `dsse-envelope.json`.

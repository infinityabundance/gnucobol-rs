<!-- DO NOT EDIT BY HAND. Generated from casefile.json by lab/casefile/run.py.
     Evidence of record: casefile.json. Portable attestations: sarif.json, intoto-statement.json, dsse-envelope.json. -->
# Forensic case file — GNURUST.18 (court-casefile)

**Verdict: PASS** · 98/98 pass, 0 fail · crate `gnucobol-rs` 0.7.0

- **Oracle:** cobc -C attr + libcob cob_move
- **Byte domain(s):** field-storage + move-result bytes (COMP-6)
- **Replay:** `bash lab/oracle/comp6_sweep.sh`
- **Authority:** STATUS.md · receipt_status: current

## Positive claims (3)
- COMP-6 field model (PACKED
- NO_SIGN_NIBBLE, size ceil(n/2))
- DISPLAY<->COMP-6 MOVE bytes matching cobc/cob_move

## Negative claims (6) — negative capability is the trust surface
- signed COMP-6 (cobc converts to COMP-3)
- COMP-6 arithmetic
- malformed bytes
- dialect portability
- pre-3.2
- lie prevented: 'unsigned packed can reuse signed packed decoding with the sign nibble ignored' — COMP-6 size is ceil(n/2) (no sign byte) and signed COMP-6 is actually COMP-3

## Damage if overclaimed
treating signed COMP-6 as COMP-6 (it is COMP-3) mis-reads the field entirely

> Generated forensic evidence (TRUST.4). The binding record is `casefile.json`; this `.md` is a rendering.
> Portable attestations: `sarif.json` (findings), `intoto-statement.json` (provenance), `dsse-envelope.json`.

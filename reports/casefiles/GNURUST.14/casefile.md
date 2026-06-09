<!-- DO NOT EDIT BY HAND. Generated from casefile.json by lab/casefile/run.py.
     Evidence of record: casefile.json. Portable attestations: sarif.json, intoto-statement.json, dsse-envelope.json. -->
# Forensic case file — GNURUST.14 (court-casefile)

**Verdict: PASS** · 546/546 pass, 0 fail · crate `gnucobol-rs` 0.7.17

- **Oracle:** cobc -C attr witness + libcob cob_move
- **Byte domain(s):** field-storage + move-result bytes
- **Replay:** `bash lab/oracle/binary_sweep.sh`
- **Authority:** STATUS.md · receipt_status: current

## Positive claims (2)
- binary field model
- DISPLAY<->binary MOVE bytes matching cobc/cob_move

## Negative claims (6) — negative capability is the trust surface
- binary arithmetic
- SYNCHRONIZED
- host-portable endian
- COMP-6
- float
- lie prevented: 'binary is one rule' — COMP-X sizes tightly; IBM/MVS use 2-4-8, MF 1--8

## Damage if overclaimed
a wrong binary width/endian mis-reads a key, branch, or amount on the wrong platform

> Generated forensic evidence (TRUST.4). The binding record is `casefile.json`; this `.md` is a rendering.
> Portable attestations: `sarif.json` (findings), `intoto-statement.json` (provenance), `dsse-envelope.json`.

<!-- DO NOT EDIT BY HAND. Generated from casefile.json by kobold-courts.
     Evidence of record: casefile.json. Portable attestations: sarif.json, intoto-statement.json, dsse-envelope.json. -->
# Forensic case file — GNURUST.CLASS.1 (court-casefile)

**Verdict: PASS** · 26/26 pass, 0 fail · crate `gnucobol-rs` 0.7.30

- **Oracle:** cobc IF data-item IS <class> branch
- **Byte domain(s):** alphanumeric field bytes -> class-condition truth
- **Replay:** `bash lab/oracle/class_sweep.sh`
- **Authority:** STATUS.md · receipt_status: current

## Positive claims (1)
- IS NUMERIC / ALPHABETIC / ALPHABETIC-UPPER / ALPHABETIC-LOWER byte predicates over an alphanumeric field matching cobc (digits-only / letters-or-space / upper-or-space / lower-or-space)

## Negative claims (5) — negative capability is the trust surface
- signed-numeric (overpunch) NUMERIC on PIC S9
- user-defined CLASS names
- national/UTF-8/DBCS classes
- locale collating sequence
- lie prevented: 'NUMERIC means parseable as a number' -- it means every byte is a digit (spaces/signs are NOT numeric)

## Damage if overclaimed
treating a space-padded or signed field as NUMERIC mis-validates input and lets bad data through

> Generated forensic evidence (TRUST.4). The binding record is `casefile.json`; this `.md` is a rendering.
> Portable attestations: `sarif.json` (findings), `intoto-statement.json` (provenance), `dsse-envelope.json`.

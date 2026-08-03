<!-- DO NOT EDIT BY HAND. Generated from casefile.json by kobold-courts.
     Evidence of record: casefile.json. Portable attestations: sarif.json, intoto-statement.json, dsse-envelope.json. -->
# Forensic case file — GNURUST.COMMON.NUMCHECK.1 (court-casefile)

**Verdict: PASS** · 2/2 pass, 0 fail · crate `gnucobol-rs` 0.8.53

- **Oracle:** cobc -debug EC-DATA-INCOMPATIBLE runtime check (libcob/common.c cob_check_numeric), captured from BOTH GnuCOBOL 3.1.2 and 3.2
- **Byte domain(s):** a non-numeric value reaching arithmetic (field name + type + raw bytes) -> the exact runtime not-numeric diagnostic message bytes
- **Replay:** `bash lab/oracle/numeric_check_sweep.sh`
- **Authority:** STATUS.md · receipt_status: current

## Positive claims (2)
- a faithful pure port of common.c's numeric-class diagnostics (explain_field_type + cob_check_numeric), reproducing the EXACT EC-DATA-INCOMPATIBLE runtime error text GnuCOBOL prints when a non-numeric value reaches arithmetic (under cobc -debug): '<name>' (Type: <type>) not numeric: '<escaped>'. explain_field_type maps the field-type code to its human name (NUMERIC DISPLAY / PACKED-DECIMAL / COMP-6 / ALPHANUMERIC / ...)
- the escaping shows printable bytes as-is and non-printable bytes as \<ooo> octal (for NUMERIC DISPLAY / alphanumeric fields) or the whole value as 0x<hex> otherwise. The pure verdict + message is separated from the abort. DIFFERENTIAL: proven byte-identical against BOTH oracles GnuCOBOL 3.1.2 + 3.2 (numeric_check_sweep 2/0). Example: "'N' (Type: NUMERIC DISPLAY) not numeric: '12X'".

## Negative claims (6) — negative capability is the trust surface
- the cob_is_numeric VALIDITY decision itself (the per-type digit/sign/BCD/float-finite check -- this court takes the numeric verdict as input and reproduces the MESSAGE
- cob_is_numeric is a follow-on)
- the libcob: <file>:<line>: error: prefix framing
- the cob_hard_failure abort + exit
- NATIONAL/UTF-16 byte widths in the escaping
- lie prevented: the not-numeric runtime diagnostic is GnuCOBOL-internal and unportable -- NO: its text is a deterministic function of the field name/type/bytes, reproduced byte-identically and proven stable across two GnuCOBOL versions

## Damage if overclaimed
claiming 'numeric checking' would hide that only the DIAGNOSTIC MESSAGE is sealed here, not the cob_is_numeric validity decision, the abort, or the prefix framing

> Generated forensic evidence (TRUST.4). The binding record is `casefile.json`; this `.md` is a rendering.
> Portable attestations: `sarif.json` (findings), `intoto-statement.json` (provenance), `dsse-envelope.json`.

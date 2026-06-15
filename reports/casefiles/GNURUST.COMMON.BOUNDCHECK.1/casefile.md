<!-- DO NOT EDIT BY HAND. Generated from casefile.json by kobold-courts.
     Evidence of record: casefile.json. Portable attestations: sarif.json, intoto-statement.json, dsse-envelope.json. -->
# Forensic case file — GNURUST.COMMON.BOUNDCHECK.1 (court-casefile)

**Verdict: PASS** · 6/6 pass, 0 fail · crate `gnucobol-rs` 0.7.79

- **Oracle:** cobc -debug runtime bounds checks (libcob/common.c cob_check_*), captured from BOTH GnuCOBOL 3.1.2 and 3.2
- **Byte domain(s):** a bounds-check input (index/offset/length + field size/limits + names) -> the exact runtime EC-BOUND diagnostic message + hint bytes
- **Replay:** `bash lab/oracle/bounds_check_sweep.sh`
- **Authority:** STATUS.md · receipt_status: current

## Positive claims (3)
- a faithful pure port of common.c's runtime bounds checks (cob_check_subscript / cob_check_odo / cob_check_ref_mod / cob_check_ref_mod_detailed / cob_check_ref_mod_minimal), reproducing the EXACT cob_runtime_error message + cob_runtime_hint text GnuCOBOL prints when a table subscript, a reference modification, or an OCCURS DEPENDING ON length goes out of bounds (under cobc -debug). The pure bounds decision + message text is separated from the abort side effect (cob_hard_failure). DIFFERENTIAL: the messages are proven byte-identical against BOTH admitted oracles -- GnuCOBOL 3.1.2 AND 3.2 (bounds_check_sweep 6/0) -- confirming they are version-stable semantics, not a single-version quirk. Examples: "subscript of 'E' out of bounds: 5" + "maximum subscript for 'E': 3"
- "length of 'F' out of bounds: 9, maximum: 5"
- "OCCURS DEPENDING ON 'N' out of bounds: 7" + "maximum subscript for 'E': 5".

## Negative claims (6) — negative capability is the trust surface
- the cob_runtime_error PREFIX framing (libcob: <file>:<line>: error: / note:) which is the runtime wrapper, not the check (the check produces the core text)
- cob_check_numeric's not-numeric message (octal/hex byte escaping -- a follow-on)
- the actual cob_hard_failure abort + exit code
- the EC exception numeric ids
- the 2.0-ABI cannot_check_subscript global state path beyond the modelled zero-subscript case
- lie prevented: the runtime bounds diagnostics are GnuCOBOL-internal and unportable -- NO: the message text is a deterministic function of the bounds inputs, reproduced byte-identically and proven stable across two GnuCOBOL versions

## Damage if overclaimed
claiming 'runtime diagnostics' broadly would hide that only these three bounds messages are sealed (not the not-numeric message, the abort, or the libcob: prefix framing)

> Generated forensic evidence (TRUST.4). The binding record is `casefile.json`; this `.md` is a rendering.
> Portable attestations: `sarif.json` (findings), `intoto-statement.json` (provenance), `dsse-envelope.json`.

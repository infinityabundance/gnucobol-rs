<!-- DO NOT EDIT BY HAND. Generated from casefile.json by kobold-courts.
     Evidence of record: casefile.json. Portable attestations: sarif.json, intoto-statement.json, dsse-envelope.json. -->
# Forensic case file — GNURUST.DIALECT.RUNTIME.ATLAS.1 (court-casefile)

**Verdict: PASS** · 7/7 pass, 0 fail · crate `gnucobol-rs` 0.7.62

- **Oracle:** cobc -std=<dialect> compile + run (cobc/config/*.conf dialect engine + libcob display)
- **Byte domain(s):** cross-dialect divergence: stored zoned-sign bytes (invariant) vs DISPLAY presentation sign placement (leading/trailing camps) vs compile-acceptance of extensions
- **Replay:** `bash lab/oracle/dialect_runtime_atlas_sweep.sh`
- **Authority:** STATUS.md · receipt_status: current

## Positive claims (3)
- observed where GnuCOBOL RUNTIME behavior diverges across its own -std dialects under the gnucobol-3.2.0 witness, verified by the sweep (7/0): the STORED zoned-sign bytes of S9(4)=-123 are dialect-INVARIANT (30 31 32 73 across default/cobol85/cobol2014/ibm-strict/mvs-strict/mf-strict/bs2000-strict/rm-strict/gcos-strict) so the record-decode lane is NOT dialect-sensitive for zoned sign
- the DISPLAY presentation of a signed field DIVERGES into a LEADING-sign camp (default/cobol85/cobol2014/mf-strict render -0123) and a TRAILING-sign camp (ibm-strict/mvs-strict/bs2000-strict/rm-strict/gcos-strict render 0123-), a presentation difference only
- and compile-acceptance of extensions diverges (COMP-5 rejected by cobol85/cobol2014, FUNCTION TRIM rejected by cobol85, USAGE BINARY-LONG rejected by cobol85 and ibm-strict)

## Negative claims (8) — negative capability is the trust surface
- non-default dialect implementation (gnucobol-rs runs only the default dialect)
- vendor-compiler parity (the -std modes are GnuCOBOL approximations of IBM/Micro Focus/MVS/BS2000/RM/GCOS, never the vendor compilers)
- the -std mode being the vendor dialect itself
- a complete enumeration of every cross-dialect divergence (this samples sign placement, presentation, and compile-acceptance)
- decoding the DISPLAY presentation form back to a value
- the screen-routed dialects (acu/realia route DISPLAY through a terminal)
- runtime portability across platforms
- lie prevented: COBOL dialect changes the record bytes -- NO: the STORED zoned-sign bytes are dialect-INVARIANT (so a record decode is not dialect-sensitive for zoned sign); what DIVERGES is the DISPLAY *presentation* (IBM/MVS/BS2000/RM/GCOS put the sign TRAILING '0123-' while default/MF put it LEADING '-0123') and the COMPILE-acceptance of extensions -- and 'GnuCOBOL -std=ibm' is NOT IBM Enterprise COBOL, only GnuCOBOL's approximation of it

## Damage if overclaimed
assuming a non-default -std mode is the real vendor compiler imports a false parity claim; conversely, assuming the stored record bytes shift with dialect would wrongly re-decode every zoned field; confusing the DISPLAY presentation (trailing sign) with the stored field corrupts both

> Generated forensic evidence (TRUST.4). The binding record is `casefile.json`; this `.md` is a rendering.
> Portable attestations: `sarif.json` (findings), `intoto-statement.json` (provenance), `dsse-envelope.json`.

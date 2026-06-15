<!-- DO NOT EDIT BY HAND. Generated from casefile.json by kobold-courts.
     Evidence of record: casefile.json. Portable attestations: sarif.json, intoto-statement.json, dsse-envelope.json. -->
# Forensic case file — GNURUST.SIZE.ERROR.1 (court-casefile)

**Verdict: PASS** · 12/12 pass, 0 fail · crate `gnucobol-rs` 0.7.80

- **Oracle:** cobc arithmetic SIZE ERROR (libcob/numeric.c)
- **Byte domain(s):** overflow -> low-order truncated store (no ON SIZE ERROR) + size-error condition
- **Replay:** `bash lab/oracle/size_error_sweep.sh`
- **Authority:** STATUS.md · receipt_status: current

## Positive claims (4)
- the byte effect of an arithmetic result that does not fit a fixed numeric receiver, matching cobc/libcob (verified 12/0): a SIZE ERROR condition occurs when the result integer part has MORE significant digits than the receiver integer capacity (overflow) or on divide-by-zero
- WITHOUT ON SIZE ERROR the receiver stores the TRUNCATED result -- low-order integer digits (most-significant DROPPED) with the fraction truncated toward zero to the receiver scale (999+999=1998 into 9(3) stores 998
- 1234.567 into 9(3)V99 stores 234.56)
- WITH ON SIZE ERROR the receiver is LEFT UNCHANGED and the imperative runs. Implements the observed SIZE.ERROR.ATLAS.1

## Negative claims (7) — negative capability is the trust surface
- the arithmetic itself (GNURUST.7/13/19)
- ROUNDED
- intermediate-result precision
- SIZE ERROR on MOVE
- floating-point receivers
- all dialects
- lie prevented: overflow drops the LOW digits -- NO: it drops the HIGH-order (most-significant) digits, keeping the LOW-order ones (1998 into 9(3) is 998 not 199), and ON SIZE ERROR LEAVES THE RECEIVER UNCHANGED rather than storing a partial result

## Damage if overclaimed
assuming overflow keeps the high digits (or that ON SIZE ERROR still stores something) corrupts amounts silently when a computed value exceeds its PICTURE

> Generated forensic evidence (TRUST.4). The binding record is `casefile.json`; this `.md` is a rendering.
> Portable attestations: `sarif.json` (findings), `intoto-statement.json` (provenance), `dsse-envelope.json`.

<!-- DO NOT EDIT BY HAND. Generated from casefile.json by kobold-courts.
     Evidence of record: casefile.json. Portable attestations: sarif.json, intoto-statement.json, dsse-envelope.json. -->
# Forensic case file — GNURUST.FILE.FILTER.SLICE.1 (court-casefile)

**Verdict: PASS** · 4/4 pass, 0 fail · crate `gnucobol-rs` 0.8.53

- **Oracle:** cobc filter read-loop (libcob/fileio.c + cobc control flow)
- **Byte domain(s):** read-loop with a per-record IF gating the accumulation -> resulting WORKING-STORAGE
- **Replay:** `bash lab/oracle/file_filter_sweep.sh`
- **Authority:** STATUS.md · receipt_status: current

## Positive claims (1)
- the SELECTIVE-accumulation read-loop -- the workhorse of COBOL batch -- executed to oracle-identical WORKING-STORAGE (verified 4/4). Each record's accumulation is gated by a per-record IF: a NUMERIC relation (compare the field's decoded unsigned value -- so R-AMT>=50 keeps 100/50/200, excludes 25/7) or an ALPHANUMERIC relation (space-padded byte compare in the ASCII collating sequence -- R-ST=A). Deepens GNURUST.FILE.FLOW.SLICE.1 with the per-record condition the read-loop court refused, distinguishing numeric (5<10) from alphanumeric ('5'>'10') comparison

## Negative claims (7) — negative capability is the trust surface
- compound conditions (AND/OR)
- signed/packed numeric filter
- per-record MOVE/transform
- multi-branch EVALUATE filter
- indexed/relative
- all dialects
- lie prevented: filtering is the same whether the field is numeric or text -- NO: a NUMERIC filter compares VALUES (R-AMT>=50 keeps 100 and 200) while an ALPHANUMERIC filter compares BYTES (where '5' > '10'), so applying the wrong comparison kind silently selects the wrong records

## Damage if overclaimed
using byte comparison on a numeric amount filter (or vice versa) selects the wrong record set, corrupting every conditional total/extract

> Generated forensic evidence (TRUST.4). The binding record is `casefile.json`; this `.md` is a rendering.
> Portable attestations: `sarif.json` (findings), `intoto-statement.json` (provenance), `dsse-envelope.json`.

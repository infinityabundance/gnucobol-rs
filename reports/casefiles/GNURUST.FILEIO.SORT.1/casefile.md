<!-- DO NOT EDIT BY HAND. Generated from casefile.json by kobold-courts.
     Evidence of record: casefile.json. Portable attestations: sarif.json, intoto-statement.json, dsse-envelope.json. -->
# Forensic case file — GNURUST.FILEIO.SORT.1 (court-casefile)

**Verdict: PASS** · 1/1 pass, 0 fail · crate `gnucobol-rs` 0.7.70

- **Oracle:** cobc SORT ... ON ASCENDING/DESCENDING KEY USING/GIVING (libcob/fileio.c)
- **Byte domain(s):** SORT keys (offset/size/direction) over records -> the sorted record order (GIVING file bytes)
- **Replay:** `bash lab/oracle/sort_sweep.sh`
- **Authority:** STATUS.md · receipt_status: current

## Positive claims (3)
- a faithful port of libcob/fileio.c sort_cmps + cob_file_sort_compare + cob_file_sort_init_key -- the record ordering a SORT produces, proven to match the admitted libcob's GIVING-file order (sort_sweep 1/0): each ON ASCENDING/DESCENDING KEY is a byte range of the record compared via sort_cmps (a byte-by-byte compare, optionally through a 256-entry collating table)
- the first key whose bytes differ decides, negated for DESCENDING
- a full tie breaks by insertion order (the unique field) so the sort is STABLE. Verified end-to-end against a real SORT ON ASCENDING KEY K1 ON DESCENDING KEY K2 with duplicate keys

## Negative claims (8) — negative capability is the trust surface
- numeric keys (routed through cob_numeric_cmp, a declared composition with GNURUST.NUMCMP.1)
- the SORT engine's tempfile merge / queues / chunking
- INPUT/OUTPUT PROCEDURE (RELEASE/RETURN)
- MERGE of multiple files
- DUPLICATES IN ORDER beyond the stable tiebreak
- the EXTFH sort path
- the fd tempfile syscalls (declared OS boundary)
- lie prevented: a SORT just orders records left-to-right -- NO: each key is an independent byte range, ASCENDING and DESCENDING keys mix within one SORT, a DESCENDING key inverts the byte compare, and equal-key records keep their input order (a STABLE sort), not an arbitrary one

## Damage if overclaimed
getting the key precedence, a DESCENDING inversion, or the stable tiebreak wrong reorders records a downstream report or match step assumes are in key order

> Generated forensic evidence (TRUST.4). The binding record is `casefile.json`; this `.md` is a rendering.
> Portable attestations: `sarif.json` (findings), `intoto-statement.json` (provenance), `dsse-envelope.json`.

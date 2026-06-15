<!-- DO NOT EDIT BY HAND. Generated from casefile.json by kobold-courts.
     Evidence of record: casefile.json. Portable attestations: sarif.json, intoto-statement.json, dsse-envelope.json. -->
# Forensic case file — GNURUST.SORT.MERGE.ATLAS.1 (court-casefile)

**Verdict: PASS** · 2/2 pass, 0 fail · crate `gnucobol-rs` 0.7.66

- **Oracle:** cobc SORT/MERGE (libcob/fileio.c sort engine + work file)
- **Byte domain(s):** SORT reordering byte-effect: ASCENDING/DESCENDING KEY, USING/GIVING over an SD work file
- **Replay:** `bash lab/oracle/sort_merge_atlas_sweep.sh`
- **Authority:** STATUS.md · receipt_status: current

## Positive claims (2)
- observed GnuCOBOL SORT byte-effect (2/0) -- the last big surface gap (SORT/MERGE 145x per GNURUST.PUBLIC.GAP.1): SORT ASCENDING KEY reorders records into KEY-ASCENDING order (input 050,010,099,020 -> 010,020,050,099) and SORT DESCENDING into key-descending order, via USING the input file
- GIVING the sorted output, over a sort-merge work file (SD). OBSERVED court: gnucobol-rs does NOT execute SORT -- it is a runtime sort engine over a work file -- this MAPS the reordering byte-effect, it sorts nothing

## Negative claims (8) — negative capability is the trust surface
- SORT execution
- INPUT/OUTPUT PROCEDURE (RELEASE/RETURN)
- MERGE
- multiple keys
- sort stability for equal keys
- custom collating sequence
- all dialects
- lie prevented: SORT is a simple in-memory reorder -- NO: it is a RUNTIME SORT ENGINE over a sort-merge work file (SD) driven by USING/GIVING or INPUT/OUTPUT PROCEDUREs, gnucobol-rs executes none of it, and the order of EQUAL keys is not a modeled guarantee; the atlas observes the key-ordered output, not the sort

## Damage if overclaimed
assuming a deterministic in-place / stable sort where GnuCOBOL runs an external sort engine misreads record order for equal keys and ignores the work-file/procedure machinery

> Generated forensic evidence (TRUST.4). The binding record is `casefile.json`; this `.md` is a rendering.
> Portable attestations: `sarif.json` (findings), `intoto-statement.json` (provenance), `dsse-envelope.json`.

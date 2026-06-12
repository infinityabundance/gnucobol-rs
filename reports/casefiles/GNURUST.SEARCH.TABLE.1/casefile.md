<!-- DO NOT EDIT BY HAND. Generated from casefile.json by kobold-courts.
     Evidence of record: casefile.json. Portable attestations: sarif.json, intoto-statement.json, dsse-envelope.json. -->
# Forensic case file — GNURUST.SEARCH.TABLE.1 (court-casefile)

**Verdict: PASS** · 6/6 pass, 0 fail · crate `gnucobol-rs` 0.7.39

- **Oracle:** cobc SEARCH/SEARCH ALL (cobc/typeck.c + codegen.c)
- **Byte domain(s):** the 1-based landing index of SEARCH (serial, forward-from-index) / SEARCH ALL (binary on ascending key)
- **Replay:** `bash lab/oracle/search_sweep.sh`
- **Authority:** STATUS.md · receipt_status: current

## Positive claims (2)
- the 1-based landing index of a COBOL table SEARCH, matching cobc/libcob (verified 6/0) -- a REAL byte court (not an atlas) composing the sealed 1-based subscript/decode model, the #2 surface gap (SEARCH 74x). SEARCH (serial) scans FORWARD ONLY from the current index for the first key match (a target before the start index is NOT FOUND -- SET IX TO 3 then searching a key at index 1 yields AT END)
- SEARCH ALL binary-searches the ASCENDING key, finding a match anywhere independent of the start index. search_serial / search_all over a keyed OCCURS table return the matching 1-based index or None

## Negative claims (7) — negative capability is the trust surface
- multi-key/DESCENDING keys
- alphanumeric/signed/V-scaled keys
- VARYING/AT END/WHEN control flow (only the landing index)
- SEARCH ALL on an unsorted table (undefined)
- OCCURS DEPENDING ON
- all dialects
- lie prevented: SEARCH and SEARCH ALL are interchangeable lookups -- NO: serial SEARCH scans FORWARD FROM THE CURRENT INDEX (so it MISSES a match before the start index), while SEARCH ALL is a BINARY search that requires the table sorted ASCENDING and finds the key anywhere; using one for the other (or SEARCH ALL on unsorted data) returns the wrong index or AT END

## Damage if overclaimed
treating serial SEARCH as position-independent (or running SEARCH ALL on an unsorted table) silently returns the wrong table element in a lookup

> Generated forensic evidence (TRUST.4). The binding record is `casefile.json`; this `.md` is a rendering.
> Portable attestations: `sarif.json` (findings), `intoto-statement.json` (provenance), `dsse-envelope.json`.

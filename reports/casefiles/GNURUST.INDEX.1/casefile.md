<!-- DO NOT EDIT BY HAND. Generated from casefile.json by kobold-courts.
     Evidence of record: casefile.json. Portable attestations: sarif.json, intoto-statement.json, dsse-envelope.json. -->
# Forensic case file — GNURUST.INDEX.1 (court-casefile)

**Verdict: PASS** · 41/41 pass, 0 fail · crate `gnucobol-rs` 0.7.70

- **Oracle:** cobc SET index-item TO/UP BY/DOWN BY; dumped via REDEFINES + FUNCTION ORD
- **Byte domain(s):** occurrence value + SET op -> 4 native-endian index bytes
- **Replay:** `bash lab/oracle/index_sweep.sh`
- **Authority:** STATUS.md · receipt_status: current

## Positive claims (2)
- the bytes a USAGE INDEX / INDEXED BY index item holds: a 4-byte native-endian (little-endian on the admitted x86-64 oracle) signed two's-complement word storing the OCCURRENCE NUMBER (element-size independent -- SET IDX TO 5 stores 5, never 5*element_size), and SET IDX TO/UP BY/DOWN BY as plain integer arithmetic on that word (DOWN past zero stores the negative two's-complement value
- cobc does not clamp)

## Negative claims (6) — negative capability is the trust surface
- 8-byte INDEX on LP64 builds where cob index width != 4
- SEARCH/SEARCH ALL execution (observed separately)
- index used as a subscript (the (idx-1)*stride multiply lives in GNURUST.SUBSCRIPT.1)
- relation conditions between index items
- SET index TO pointer/address
- lie prevented: 'an INDEX item stores a byte offset (occurrence*element_size)' -- NO, it stores the plain occurrence number; the stride multiply happens at subscript time, not in the index word

## Damage if overclaimed
mis-modelling an index as an offset (or wrong endianness) corrupts every SET/SEARCH and every index-driven table access

> Generated forensic evidence (TRUST.4). The binding record is `casefile.json`; this `.md` is a rendering.
> Portable attestations: `sarif.json` (findings), `intoto-statement.json` (provenance), `dsse-envelope.json`.

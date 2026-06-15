<!-- DO NOT EDIT BY HAND. Generated from casefile.json by kobold-courts.
     Evidence of record: casefile.json. Portable attestations: sarif.json, intoto-statement.json, dsse-envelope.json. -->
# Forensic case file — GNURUST.BUILD.PROFILE.1 (court-casefile)

**Verdict: PASS** · 1/1 pass, 0 fail · crate `gnucobol-rs` 0.7.71

- **Oracle:** cobc --version / cobc -info / default.conf / libcob sha256
- **Byte domain(s):** cobc/libcob/config build profile (version, endianness, binary-byteorder, char signedness, C-long, sha256)
- **Replay:** `bash lab/oracle/build_profile_sweep.sh`
- **Authority:** STATUS.md · receipt_status: current

## Positive claims (3)
- binds the EXACT build profile of the oracle that produced every witness, as first-class citable evidence: GnuCOBOL 3.2.0 built with GCC 16.1.1 on x86_64-pc-linux-gnu, host LITTLE-endian, char SIGNED (-fsigned-char), BINARY-C-LONG 8 bytes, and the dialect config (binary-byteorder=BIG-endian, binary-size=1-2-4-8, binary-truncate=yes, binary-comp-1=no, numeric-pointer=no), each with a sha256 of cobc/libcob/config. The byte parity of every ABI-SENSITIVE court (GNURUST.14 binary COMP/COMP-5/COMP-X
- GNURUST.15/17 EBCDIC) is SCOPED TO THIS PROFILE
- the sweep PASS=1 iff the live profile matches the committed golden, so a rebuild that changes the ABI is flagged

## Negative claims (6) — negative capability is the trust surface
- other builds/dialects/configs
- cross-architecture ABI
- that the profile transfers to a different binary-byteorder
- compiler-internal struct layout beyond the recorded fields
- all dialects
- lie prevented: COMP is big-endian (or little-endian) universally -- NO: COMP byte order is the CONFIG binary-byteorder (BIG-endian here) NOT the host order (LITTLE-endian), COMP-5/COMP-X follow native, char is signed by a build flag, so byte parity is a claim ABOUT A SPECIFIC BUILD PROFILE, not COBOL in the abstract; a different profile changes the bytes

## Damage if overclaimed
presenting byte parity as build-independent silently breaks on any oracle rebuilt with a different binary-byteorder / char-signedness / dialect -- the ABI abyss

> Generated forensic evidence (TRUST.4). The binding record is `casefile.json`; this `.md` is a rendering.
> Portable attestations: `sarif.json` (findings), `intoto-statement.json` (provenance), `dsse-envelope.json`.

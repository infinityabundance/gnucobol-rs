<!-- DO NOT EDIT BY HAND. Generated from casefile.json by kobold-courts.
     Evidence of record: casefile.json. Portable attestations: sarif.json, intoto-statement.json, dsse-envelope.json. -->
# Forensic case file — GNURUST.DIRECTIVE.VARIANCE.ATLAS.1 (court-casefile)

**Verdict: PASS** · 6/6 pass, 0 fail · crate `gnucobol-rs` 0.7.62

- **Oracle:** cobc -f<directive> compile + run (cobc/config.c directive engine + libcob/move.c binary codec)
- **Byte domain(s):** compiler-directive byte delta from the default profile: -fbinary-size (layout), -fbinary-byteorder (endianness), -fbinary-truncate (MOVE result)
- **Replay:** `bash lab/oracle/directive_variance_atlas_sweep.sh`
- **Authority:** STATUS.md · receipt_status: current

## Positive claims (3)
- observed how COMPILER DIRECTIVES change the record bytes under the gnucobol-3.2.0 witness, verified by the sweep (6/0): -fbinary-size=2-4-8 allocates 9(2) COMP as 2 bytes instead of 1 so a record's COMP offsets and total length SHIFT (group LENGTH 7 -> 8)
- -fbinary-byteorder=native stores a COMP field host-little-endian (4660 -> 34 12) vs the default config big-endian (12 34)
- and -fbinary-truncate (default, ANSI) truncates a COMP MOVE to the PIC digits (MOVE 300 TO 9(2)COMP -> 00) while -fno-binary-truncate keeps the raw binary value (-> 44) -- the byte-level delta from the BUILD.PROFILE.1 default profile

## Negative claims (8) — negative capability is the trust surface
- implementation of non-default directives (gnucobol-rs decodes under the BUILD.PROFILE.1 default only)
- auto-detection of a binary's build profile from its bytes
- a complete enumeration of every cobc directive
- dialect-selection flags (-std, owned by GNURUST.DIALECT.RUNTIME.ATLAS.1)
- code-generation/optimization directives
- runtime environment variables (COB_*)
- vendor-specific directives
- lie prevented: a COBOL binary's record bytes are fixed by the copybook alone -- NO: a COMPILER DIRECTIVE shifts them: -fbinary-size=2-4-8 makes a small COMP field 2 bytes (mis-offsetting every later field), -fbinary-byteorder=native flips multi-byte COMP endianness, and -fno-binary-truncate changes what a MOVE stores -- so a correct decode REQUIRES the producer's build profile (BUILD.PROFILE.1), it cannot be inferred from the copybook

## Damage if overclaimed
decoding a record produced under a non-default binary-size/byteorder/truncate with the default profile silently mis-offsets fields, reverses binary values, or mis-rounds MOVEs; assuming the profile can be auto-detected from bytes invents a build environment that was never recorded

> Generated forensic evidence (TRUST.4). The binding record is `casefile.json`; this `.md` is a rendering.
> Portable attestations: `sarif.json` (findings), `intoto-statement.json` (provenance), `dsse-envelope.json`.

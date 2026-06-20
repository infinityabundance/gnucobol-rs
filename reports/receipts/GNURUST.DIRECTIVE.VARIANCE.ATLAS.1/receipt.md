<!-- GENERATED from receipt.json by xtask receipt — DO NOT EDIT BY HAND.
     Regenerate: cargo run -p xtask -- receipt generate -->
# GNURUST.DIRECTIVE.VARIANCE.ATLAS.1 — observed compiler-directive byte-variance atlas

**Verdict: PASS** · replay `PASS=6 FAIL=0`

| field | value |
|-------|-------|
| campaign | `GNURUST.DIRECTIVE.VARIANCE.ATLAS.1` |
| court | observed compiler-directive byte-variance atlas |
| crate_version | `0.8.11` |
| oracle | cobc (GnuCOBOL) 3.2.0 |
| byte_domain | compiler-directive byte delta from the default profile: -fbinary-size (layout), -fbinary-byteorder (endianness), -fbinary-truncate (MOVE result) |
| replay command | `bash lab/oracle/directive_variance_atlas_sweep.sh` |
| generated_at | unstamped |
| git_commit | `unstamped` |
| receipt_status | current |

## Non-claims
- implementation of non-default directives
- auto-detection of a binary's build profile
- complete enumeration of every cobc directive
- dialect-selection (-std) flags
- code-generation/optimization directives
- runtime environment variables (COB_*)
- vendor-specific directives

> A receipt is the reproducible output of a replayable court run, not a static claim. This `.md` is
> generated from `receipt.json`; the binding evidence is the JSON. Regenerate with
> `cargo run -p xtask -- receipt generate`.

<!-- GENERATED from receipt.json by xtask receipt — DO NOT EDIT BY HAND.
     Regenerate: cargo run -p xtask -- receipt generate -->
# GNURUST.MLIO.PARSE.1 — XML PARSE (native state machine)

**Verdict: PASS** · replay `PASS=2 FAIL=0`

| field | value |
|-------|-------|
| campaign | `GNURUST.MLIO.PARSE.1` |
| court | XML PARSE (native state machine) |
| crate_version | `0.8.38` |
| oracle | cobc (GnuCOBOL) 3.2.0 |
| byte_domain | an XML PARSE input field + cross-call state -> the XML-EVENT / XML-CODE / XML-TEXT register sequence |
| replay command | `bash lab/oracle/ml_parse_sweep.sh` |
| generated_at | unstamped |
| git_commit | `unstamped` |
| receipt_status | current |

## Non-claims
- an actual XML tree/content parse (GnuCOBOL 3.2 XML PARSE is an unimplemented stub; the port reproduces its observable START-OF-DOCUMENT->END-OF-INPUT->END-OF-DOCUMENT event walk, not a real parser)
- JSON PARSE (no such statement in GnuCOBOL 3.2 mlio.c)
- COMPAT (non-XMLNSS) full-document XML-TEXT delivery beyond the modelled path
- multi-chunk streaming + schema VALIDATING

> A receipt is the reproducible output of a replayable court run, not a static claim. This `.md` is
> generated from `receipt.json`; the binding evidence is the JSON. Regenerate with
> `cargo run -p xtask -- receipt generate`.

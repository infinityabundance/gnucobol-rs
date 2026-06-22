<!-- GENERATED from receipt.json by xtask receipt — DO NOT EDIT BY HAND.
     Regenerate: cargo run -p xtask -- receipt generate -->
# GNURUST.MLIO.GENERATE.1 — XML / JSON GENERATE (native serializer)

**Verdict: PASS** · replay `PASS=5 FAIL=0`

| field | value |
|-------|-------|
| campaign | `GNURUST.MLIO.GENERATE.1` |
| court | XML / JSON GENERATE (native serializer) |
| crate_version | `0.8.37` |
| oracle | cobc (GnuCOBOL) 3.2.0 |
| byte_domain | a cob_ml_tree -> the XML / JSON GENERATE output bytes |
| replay command | `bash lab/oracle/ml_generate_sweep.sh` |
| generated_at | unstamped |
| git_commit | `unstamped` |
| receipt_status | current |

## Non-claims
- XML/JSON PARSE is sealed separately (GNURUST.MLIO.PARSE.1)
- NATIONAL/UTF-16 content + non-ASCII multibyte escaping
- pretty-print / indentation options
- namespaces / complex attributes

> A receipt is the reproducible output of a replayable court run, not a static claim. This `.md` is
> generated from `receipt.json`; the binding evidence is the JSON. Regenerate with
> `cargo run -p xtask -- receipt generate`.

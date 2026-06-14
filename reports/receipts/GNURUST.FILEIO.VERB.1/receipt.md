<!-- GENERATED from receipt.json by xtask receipt — DO NOT EDIT BY HAND.
     Regenerate: cargo run -p xtask -- receipt generate -->
# GNURUST.FILEIO.VERB.1 — file verb open/access-mode preconditions

**Verdict: PASS** · replay `PASS=7 FAIL=0`

| field | value |
|-------|-------|
| campaign | `GNURUST.FILEIO.VERB.1` |
| court | file verb open/access-mode preconditions |
| crate_version | `0.7.52` |
| oracle | cobc (GnuCOBOL) 3.2.0 |
| byte_domain | cob_write/read/read_next/rewrite/delete/start preconditions -> FILE STATUS (43/44/46/47/48/49/23) |
| replay command | `bash lab/oracle/verb_sweep.sh` |
| generated_at | unstamped |
| git_commit | `unstamped` |
| receipt_status | current |

## Non-claims
- the organization dispatch (sealed by LINESEQ/SEQ/RELATIVE)
- the compile-time START-on-RANDOM rejection
- LINE SEQUENTIAL validate-71 at the verb layer
- CODE-SET conversion
- the variable_record size resolution
- EOP / exception side effects
- the indexed suppressed-key skip
- the fd read/write syscalls (declared OS boundary)

> A receipt is the reproducible output of a replayable court run, not a static claim. This `.md` is
> generated from `receipt.json`; the binding evidence is the JSON. Regenerate with
> `cargo run -p xtask -- receipt generate`.

<!-- GENERATED from receipt.json by xtask receipt — DO NOT EDIT BY HAND.
     Regenerate: cargo run -p xtask -- receipt generate -->
# GNURUST.ELITE-REPLAY.2 — broad public-corpus replay -- the GCC-COBOL testsuite + the hand-authored front-end corpus run through cobc 3.2 AND cobrun; observable behaviour (stdout bytes + process exit) must agree, with cobrun failing CLOSED (never wrong) wherever it cannot run a program

**Verdict: FAIL** · replay `PASS=339 FAIL=6 SKIP=381 MATCH=339`

| field | value |
|-------|-------|
| campaign | `GNURUST.ELITE-REPLAY.2` |
| court | broad public-corpus replay -- the GCC-COBOL testsuite + the hand-authored front-end corpus run through cobc 3.2 AND cobrun; observable behaviour (stdout bytes + process exit) must agree, with cobrun failing CLOSED (never wrong) wherever it cannot run a program |
| crate_version | `0.8.55` |
| oracle | cobc (GnuCOBOL) 3.2.0 |
| byte_domain | a real public COBOL program -> its exact stdout bytes + process exit status, byte-identical to cobc; the GREEN invariant is that cobrun is byte-identical wherever it runs and fails-closed everywhere else (never a silent wrong answer) |
| replay command | `bash lab/oracle/elite_replay2_sweep.sh` |
| generated_at | unstamped |
| git_commit | `unstamped` |
| receipt_status | current |

## Non-claims
- MATCH 281/718: programs cobrun runs byte-identical to cobc. The rest are categorized, never silently wrong: ~233 no-oracle (cobc 3.2 itself cannot compile / the GCC-COBOL-built oracle crashes -> no baseline), ~182 boundary (cobrun fails CLOSED with a typed cobrun: error -- a construct outside the sealed subset).
- 22 KNOWN silent divergences are committed + tracked in lab/oracle/elite_replay2_known.txt (each a real cobrun bug/gap with a reason) and driven down to empty -- the ratchet; an UN-tracked divergence FAILs the sweep, so a regression can never hide.
- this is the BROAD public differential; the focused opencbs real-program court is GNURUST.ELITE-REPLAY.1 (39/39). NIST CCVS-85 (524 programs) is held under a separate custody gate and is the path to a 1000+ program corpus (CCVS-format extraction is a follow-on).
- no file/DB-state differential yet (stdout + exit only); programs requiring external data files run from their own directory

> A receipt is the reproducible output of a replayable court run, not a static claim. This `.md` is
> generated from `receipt.json`; the binding evidence is the JSON. Regenerate with
> `cargo run -p xtask -- receipt generate`.

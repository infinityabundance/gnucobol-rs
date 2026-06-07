# RECEIPT-GNURUST-REPLACING-6 — sealed: COPY ... REPLACING (whole-text-word)

**Campaign GNURUST.6.** Goal: implement GnuCOBOL-compatible `COPY ... REPLACING ==old== BY ==new==`
pseudo-text replacement at **text-word** granularity, preserving COPY provenance, and prove the
expanded text-word stream matches `cobc -P`.

## Claim (exact)

`gnucobol_rs::copybook::expand` applies `COPY name REPLACING ==p== BY ==q== ….` (one or more
pseudo-text operand pairs, the statement possibly spanning lines to its terminating period) so that
the expanded source's **text-word stream is identical to `cobc -P`**. Replacement is **whole
text-word**, not string substitution: a text word is a maximal run of `[_-0-9A-Za-z]` + high bytes
(GnuCOBOL `is_word`, `cobc/replace.c:603`). Diagnosed and reproduced behaviours:

- `==AA==` does **not** touch `AA-X`, `KEEP-AA`, or `BB-` (each a single text word); the `:tag:`
  idiom works because `:` (non-word) splits `:PFX:` into its own text word, and the adjacent words
  re-concatenate on output (`:PFX:-ID` → `CUST-ID`).
- **Multiple pairs** apply per text word, first-listed-pair-wins, left to right.
- **Nesting composes**: the outer REPLACING does **not** alter a nested `COPY`'s operands, but it
  **does** penetrate the text the nested copy brings in — applied **after** the nested copy's own
  REPLACING (verified: outer `==AMOUNT== BY ==QQ==` turns a nested-copied `AMOUNT` into `QQ`, while
  the nested `==:PFX:== BY ==INNER==` still fires because the outer did not rewrite its operand).

## Non-claims (fail closed)

Non-pseudo-text REPLACING forms are **deferred, not half-applied** (`CopyError::ReplacingDeferred`):
`LEADING`/`TRAILING`, identifier/literal operands (`REPLACING A BY B`), unterminated `==`, and
`REPLACE` (the standalone directive, distinct from `COPY ... REPLACING`). Comparison is at
text-word granularity, not column/byte formatting.

## Oracle

`cobc -P` (the GnuCOBOL preprocessor) expands `COPY ... REPLACING`; the sweep strips its line-number
column and tokenizes to text-words. The Rust mirror tokenizes its expansion identically.

## Evidence

| Check | Result |
|-------|--------|
| Differential sweep vs `cobc -P` | **programs=7, PASS=7 FAIL=0** (`lab/oracle/copy_sweep.sh`): plain COPY, nested COPY, multiple COPYs (GNURUST.5) + simple replacement, multiple pairs with whole-word non-match, nested composing replacement, multi-line REPLACING statement (GNURUST.6) |
| Self-contained `cargo test` | copybook: 4/4 (whole-text-word; nested compose-but-not-operands; fail-closed forms) |
| Fuzz (`copybook` target, arbitrary source+copybooks incl. REPLACING) | **4,000,000 runs, 0 crashes** |
| `fmt` / `clippy -D warnings` / doc-gate | clean |

## Determinism

`expand` remains a pure function of `(source, resolver)`. Same pinned oracle/env as the other
courts. Provenance is preserved through replacement (the diagnostic spine, `GNURUST.COPYMAP.0`).

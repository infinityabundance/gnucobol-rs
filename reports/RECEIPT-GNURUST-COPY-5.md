# RECEIPT-GNURUST-COPY-5 — sealed: COPY copybook expansion

**Campaign GNURUST.5.** Goal: expand `COPY` copybook statements into the source — recursively, with
cycle detection and a provenance map — and prove the expansion matches the GnuCOBOL preprocessor.

## Claim (exact)

For a line-oriented `COPY <name>.` statement (the name a COBOL word; the statement on its own line),
`gnucobol_rs::copybook::expand` produces an expanded source whose **text-word stream is identical**
to GnuCOBOL's preprocessor (`cobc -P`), including nested `COPY` (a copybook that copies others) and
multiple sequential copies. Comparison is at text-word granularity, which is immune to the
preprocessor's column/indent reformatting (whitespace is not semantic to COPY). A **provenance map**
records, for each expanded line, the file (`<main>` or copybook name) and original line it came from.

## Non-claims (fail closed)

- `COPY ... REPLACING` — the text-word replacement algorithm is a separate court
  (`GNURUST.6` / `GNURUST.REPLACEALG.0`); rejected here with `CopyError::ReplacingDeferred`, never
  half-applied. (Confirmed against the oracle: GnuCOBOL REPLACING is **whole-text-word** matching —
  `==AA==` does not touch `AA-X` or `BAAB` — and the `:tag:` idiom relies on `:`-delimited words;
  reproducing that faithfully warrants its own port.)
- Inline / multi-line `COPY` statements, `COPY ... OF/IN library`, `SUPPRESS` — not parsed.
- A recursive `COPY` cycle, a missing copybook, nesting deeper than the limit, or an oversized
  expansion (`GNURUST.DOS.0`) — typed `CopyError`.

## Oracle

`cobc -P=<file> -I <copydir> -fsyntax-only <prog.cob>` writes the preprocessed (COPY-expanded)
source; the sweep strips its line-number column and tokenizes to text-words. The Rust mirror
(`examples/copy_rows`) resolves copybooks from the same directory (mirroring `cobc`'s name+extension
search) and tokenizes its expansion. (`cobc -P` is the GnuCOBOL preprocessor's own output —
`GNURUST.GENC.0`-style witness of the COPY phase.)

## Evidence

| Check | Result |
|-------|--------|
| Differential sweep vs `cobc -P` | **programs=3, PASS=3 FAIL=0** (`lab/oracle/copy_sweep.sh`): simple COPY, nested COPY (a copybook copying two others), multiple sequential COPYs |
| Self-contained `cargo test` | copybook: 3/3 (splice + provenance; nested; recursive/missing/REPLACING fail closed) |
| Fuzz (`copybook` target, arbitrary source+copybooks) | **3,000,000 runs, 0 crashes** — cycles / missing / deep nesting all yield typed `CopyError` |
| `fmt` / `clippy -D warnings` / doc-gate | clean |

## Provenance (the diagnostic spine)

`Expanded.provenance[i]` maps expanded line `i` → `{file, line}`. This is the foundation for future
diagnostic source spans and `COPY REPLACING` origin tracking (`GNURUST.COPYMAP.0`).

## Determinism

`expand` is a pure function of `(source, resolver)`; the resolver owns the filesystem/search path,
so the core stays pure (`GNURUST.PUREDEC.0`). Same pinned oracle/env as the other courts.

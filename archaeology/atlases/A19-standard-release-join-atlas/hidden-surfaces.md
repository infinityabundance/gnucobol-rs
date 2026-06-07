# Hidden / not-well-known surfaces the join surfaces

Generated reading of `join.json` — the obscure stuff normal searches miss.

1. **Syntax recognized for compatibility but nonfunctional at runtime** — the `-std` vendor dialects add
   *hundreds* of reserved words (e.g. `mf-strict` carries 111 not in `default`; see G-axis) without
   implying any of those features are functional. Reserved-word presence is recognition, not behavior.
2. **Backend-dependent features** — `JSON/XML GENERATE` (cJSON/JSON-C/libxml2), indexed files
   (BDB/VBISAM), screen section (curses): the syntax can parse while the runtime is optional. A build
   without the lib changes the answer.
3. **Physical vs logical byte domains** — `OCCURS DEPENDING ON` allocates the physical maximum in
   generated C; the active/logical length is a *different* fact. Conflating them is a classic migration
   bug (`gnucobol-rs` seals physical-max only).
4. **GnuCOBOL-vs-standard storage drift** — `COMP-X` uses a tighter byte table than `COMP`/`COMP-5`.
   "Binary numeric" is not one rule.
5. **Standard-before-GnuCOBOL gaps** — `SET..TO FALSE` / FALSE clause: standardized, but a court must
   still prove the exact bytes; `gnucobol-rs` fails closed rather than guess.
6. **Vendor-only extensions** — `COMP-6` (unsigned packed, no sign nibble): never standard; real in
   legacy data. Admit only when oracle-witnessed.
7. **EBCDIC is platform/table-specific** — GnuCOBOL 3.2 ships `ebcdic500` (cp500), *not* cp037. Admit
   the table the oracle ships, defer the one it doesn't.
8. **Reserved-word-before-feature** — a word can be reserved across dialects long before (or without)
   the feature being functional in any build.

Each of these is a `drift` tag in `drift-matrix.tsv`; the `fixture_priority` column says which deserve a
`gnucobol-rs` court next.

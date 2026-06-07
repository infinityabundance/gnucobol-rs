# gnucobol-rs roadmap, generated from the atlas join

Ranked from `fixture-candidates.json` (unsealed/partial surfaces, by `fixture_priority` then real-
copybook likelihood + oracle-testability). The atlas IS the roadmap generator.

| rank | surface | fixture_priority | gnucobol-rs status | next court |
|------|---------|------------------|--------------------|------------|
| 1 | code_page_ebcdic (numeric zoned) | high | cp500 text sealed; numeric zoned deferred | GNURUST.EBCDIC-NUM |
| 2 | edited_pictures (financial decorations) | high | 16a decode sealed | GNURUST.16b ($ * CR DB B 0 /) |
| 3 | condition_names_false_clause | medium | SET TRUE sealed; FALSE fails closed | GNURUST.12b |
| 4 | arithmetic_divide | medium | deferred | GNURUST.17 |
| 5 | synchronized_alignment | medium | fails closed | (defer — platform-dependent) |

Rule: a surface graduates to a court only when it is **real in copybooks**, **oracle-testable**, and
**raises migration trust** — never because the standard mentions it. `out_of_model` surfaces
(files, reports, screen, JSON/XML, CALL, OO) stay out until they directly improve migration evidence.

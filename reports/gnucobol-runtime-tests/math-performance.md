# GnuCOBOL runtime/mathematics — performance (strictly labeled views)

Method: output-equivalence is proven FIRST (only byte-identical programs are timed); views are
NEVER averaged together; native compile+run vs interpreter adapt+run are DIFFERENT work, so
view A is observational only and no cross-implementation speed claim is made. View B is the
SAME program run repeatedly (native executable vs candidate launcher; the candidate re-parses
every run — that cost is included and labeled). Per-sample totals under raw-samples/;
N=200 after 20 warmups, monotonic ms timer, pinned machine/container.

| program | View A native (compile+run, ms) | View A candidate (adapt+run, ms) | View B native (ms/run) | View B candidate (ms/run) |
|---|---|---:|---:|---:|
| mixed_moves | 62 | 13 | 1.015 | 12.18 |
| packed_math | 66 | 13 | 1.87 | 12.05 |
| display_arith | 68 | 25 | 2.04 | 24.11 |
| packed_loop | 70 | 18 | 1.265 | 17.335 |

Caveats: these numbers describe THIS pinned machine/workload only; they are not a product
comparison. View C (runtime-operation microbenchmarks over the admitted libcob C harness vs
the Rust runtime ops) is a separately-designed court; it is not mixed into these views.

# GnuCOBOL runtime/mathematics — performance (strictly labeled views)

Method: output-equivalence is proven FIRST (only byte-identical programs are timed); views are
NEVER averaged together; native compile+run vs interpreter adapt+run are DIFFERENT work, so
view A is observational only and no cross-implementation speed claim is made. View B is the
SAME program run repeatedly (native executable vs candidate launcher; the candidate re-parses
every run — that cost is included and labeled). Per-sample totals under raw-samples/;
N=200 after 20 warmups, monotonic ms timer, pinned machine/container.

| program | View A native (compile+run, ms) | View A candidate (adapt+run, ms) | View B native (ms/run) | View B candidate (ms/run) |
|---|---|---:|---:|---:|
| mixed_moves | 58 | 13 | 1.01 | 12.07 |
| packed_math | 63 | 12 | 1.435 | 12.035 |
| display_arith | 67 | 25 | 2.005 | 23.785 |
| packed_loop | 66 | 18 | 1.3 | 17.48 |

Caveats: these numbers describe THIS pinned machine/workload only; they are not a product
comparison. View C (runtime-operation microbenchmarks over the admitted libcob C harness vs
the Rust runtime ops) is a separately-designed court; it is not mixed into these views.

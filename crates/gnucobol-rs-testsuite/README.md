# gnucobol-rs-testsuite

`gnucobol-rs-testsuite` — the `GNURUST.GNUCOBOL-TESTSUITE.{1,2,3}` differential court harness for
the **native GnuCOBOL 3.2 Autotest suite**.

The GnuCOBOL test suite runs through its **own machinery** (fresh in-tree build, `make check`
baseline with the real admitted `cobc`, then `make localcheck` with `COBC=cobc-rs`) inside an
isolated rootless-Docker court. This crate turns the raw suite output into typed evidence:

1. **census** — the invocation census: every `cobc`/`cobcrun` invocation, argument boundaries
   preserved, grouped into `invocation-census.json` + `options-frequency.csv`.
2. **classify** — every generated test group receives **exactly one** final classification
   (oracle verdict × wrapper × candidate × comparison), with per-test reason codes and
   first-failure attribution.
3. **determinism** — fresh-run two-pass reconciliation (stable summaries identical).
4. **gate check** — the host-side gate: all-tests-accounted invariant, math-subset 323
   reconciliation, no-delegation proof, raw-evidence and receipt freshness.

All 1,282 generated test groups reconcile exactly. No suite-parity or conformance claim is made.

## Usage

```sh
# one-command replay (rootless-Docker court, configurable storage root):
bash lab/gnucobol-testsuite/run-docker.sh

# host-side gate after a court run:
cargo run -p gnucobol-rs-testsuite -- gate check --root .

# math-subset freshness check (sum(classification counts) == 323 invariant):
cargo run -p gnucobol-rs-testsuite -- math check --results reports/gnucobol-testsuite/test-inventory.json
```

## Repository context

Full methodology and the measured ledgers live in the
[`gnucobol-rs`](https://github.com/infinityabundance/gnucobol-rs) repository under
`reports/gnucobol-testsuite/` and `reports/maintainer-review/GNUCOBOL-TESTSUITE-FOLLOWUP.md`.

## License

LGPL-3.0-or-later, matching the project license.

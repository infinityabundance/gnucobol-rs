# gnucobol-rs-ccvs85

`gnucobol-rs-ccvs85` — the `GNURUST.CCVS85.2 / .3 / .4` differential court harness.

It runs the admitted NIST CCVS85 v4.0 corpus (512 indexed units) through three evidence phases and
gates the result:

1. **materialize** — byte-preserving corpus materialization (SHA-256 per unit) from the admitted
   `newcob.val.Z` spine.
2. **oracle-run** — the pinned real-GnuCOBOL 3.2 oracle baseline (compiled in-container from the
   pinned source tarball, never a distro package).
3. **candidate-run** — the native-Rust `cobrun` candidate with the mechanical no-delegation proof
   (oracle renamed away, `cobc` off `PATH`, `cobrun` links no `libcob`).
4. **classify** — every unit into exactly one typed category, with fresh-container determinism
   verification and a host-side `gate check`.

**Observation only** — no NIST certification, no COBOL-85 conformance claim, no `cobc`-replacement
claim. Every unit is explicitly classified; none is silently dropped.

## Usage

```sh
# one-command replay (rootless-Docker court, configurable storage root):
bash lab/ccvs85/run-docker.sh

# individual phases (see the binary --help):
cargo run -p gnucobol-rs-ccvs85 -- materialize --help
cargo run -p gnucobol-rs-ccvs85 -- classify --help
```

## Repository context

The full methodology lives in the [`gnucobol-rs`](https://github.com/infinityabundance/gnucobol-rs)
repository: `docs/methodology/`, `reports/ccvs85/`, and the forensic casefiles under
`reports/casefiles/GNURUST.CCVS85.*/`.

## License

LGPL-3.0-or-later. This crate is court harness tooling; the runtime it tests is a **faithful
copyleft derivative** of GnuCOBOL's `libcob` (not clean-room) — see the repository's
`docs/derivation-and-license.md` and `docs/license-boundaries.md`.

<!-- DO NOT EDIT BY HAND. Generated from casefile.json by kobold-courts.
     Evidence of record: casefile.json. Portable attestations: sarif.json, intoto-statement.json, dsse-envelope.json. -->
# Forensic case file — GNURUST.PERFORMANCE.BUSINESS.1 (court-casefile)

**Verdict: PASS** · 1/1 pass, 0 fail · crate `gnucobol-rs` 0.8.57

- **Oracle:** GnuCOBOL 3.2.0 (admitted lab/oracle build) + the committed corpus evidence under reports/valid-corpus/
- **Byte domain(s):** reports/valid-corpus/performance/benchmarks.json
- **Replay:** `bash lab/oracle/../valid-corpus/corpus_court_sweep.sh performance`
- **Authority:** STATUS.md · receipt_status: current

## Positive claims (1)
- ten purpose-built workload families (payroll, invoice, seqfile, relative, tables, strings, modules, float, report, mixed) x four deterministic scales are correctness-gated byte-exact against the host oracle before any timing (benchmarks.json)

## Negative claims (3) — negative capability is the trust surface
- workloads are project-owned
- inputs come from deterministic Rust generators and expected outputs are independently computed, never by the candidate
- lie prevented: every number in the report is aggregated from the committed per-family evidence; this court re-verifies the evidence tree, it never re-measures or invents values

## Damage if overclaimed
benchmarking a workload whose correctness was not established would invalidate every timing

> Generated forensic evidence (TRUST.4). The binding record is `casefile.json`; this `.md` is a rendering.
> Portable attestations: `sarif.json` (findings), `intoto-statement.json` (provenance), `dsse-envelope.json`.

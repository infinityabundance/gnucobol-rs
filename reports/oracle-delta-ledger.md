# Oracle delta ledger

Not every mismatch is a port bug. When the differential sweep or a fixture diverges from the
oracle, the divergence is **classified here**, never waved off informally. Empty today (the
sealed decimal slice is `FAIL=0`); the schema exists so the first real delta has a home.

```yaml
schema: gnucobol-rs-oracle-delta-v1
columns: [case, oracle_version, input, observed_oracle, rust_behavior, classification, status, future_campaign]
classifications:
  - port_bug                      # our error — fix it
  - unsupported_surface           # outside the sealed claim — fail closed, named
  - gnucobol_build_option_diff    # different --configure / flags
  - dialect_config_diff           # different -conf / -std / runtime.cfg
  - platform_diff                 # endianness / charset / OS
  - known_upstream_bug            # cite the upstream ticket
  - intentional_safety_rejection  # Rust rejects hostile input the C would UB on
  - unclassified                  # must not stay here — triage to one of the above
entries: []   # decimal slice: PASS, no deltas
```

# Pristine vs diagnostic-unblocked — reconciliation

_schema: gnurust-diag-unblocked-reconcile-v1

PRISTINE: exact diagnostic parity still required; the upstream suite is the
compatibility authority and remains untouched.

UNBLOCKED: exactly the same test semantics except explicitly admitted compiler
diagnostic bytes are ignored; expected exit status, commands, source, runtime
output and generated-file expectations are still enforced.

## Structural counts

| AT_SETUP pristine | AT_SETUP unblocked | AT_CHECK pristine | AT_CHECK unblocked |
|---|---|---|---|
| 1344 | 1344 | 3422 | 3422 |
- suite groups (oracle evidence): 1282
- pristine candidate evidence groups: 1282; unblocked: 1282
- group identity identical: true

## Integrity proofs

- patch reproducible (regenerated == committed): true
- transformations.json reproducible: true
- committed patch sha256: `712a0b172021c7ec650c6d97e348b465efd355699e309e956d95f99a2dc69230`
- regenerated patch sha256: `712a0b172021c7ec650c6d97e348b465efd355699e309e956d95f99a2dc69230`
- pristine manifest sha256: `1022ce18b3df42267b53d567a243a41cc1804c03bf06c375c28662431e61dafd`
- transformed manifest sha256: `758aea6317f0fd835d7c1adc2f20fc0ef3e3a55ab489b537671bd0515141f5f7`
- command census hash (all 3422 commands): `27c582a9d7660e1987f0596415eb9c4767f0df086be66f24ecd755cb0a19b2db`
- expected-status census hash: `09cf7af0849ef583d4afa4415b2f803b5de132665fb0e16108326db3302974ff`
- policy gate: 0 failures

The semantic reachability delta (what the unblocked lane actually exposed) is
reported separately in `semantic-reachability.json` / `.md`.

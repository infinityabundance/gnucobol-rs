# TRUST.4.DOCS — generated documentation authority

> **Doctrine.** TRUST.4.DOCS makes documentation a forensic artifact: every authoritative document is a
> generated view over structured evidence, preserved legacy content, negative capabilities, and current
> replay state; the generated document must be an information superset of the original, and **staleness
> becomes a machine-detectable negative claim** rather than a hidden documentation bug.

TRUST.2 generated receipts; TRUST.3 made proof safe to interpret; TRUST.4 made every *report* a forensic
casefile; **TRUST.4.DOCS** extends that to the *documentation* itself.

## How it works

```
docs-src/<id>.model.json   authored prose fragments + {{machine placeholders}} + legacy_preservation
        +
machine evidence           Cargo.toml version, claim-ladder, casefiles, receipts
        ↓  lab/docs/generate.py
<generated authoritative doc>.md   (header: DO NOT EDIT; machine authority; legacy source + sha)
```

The generated doc is an **information superset**: it carries forward every legacy claim, non-claim,
caveat, and note (recorded in the model's `legacy_preservation`, with the original preserved byte-for-byte
under `research/legacyreports/docs/`), then adds machine-derived facts (current version, a generated
sealed-courts table, casefile/receipt links) that the hand-written original could not keep fresh.

## Staleness is a negative capability

Documentation drift is no longer a silent bug — it is a machine-detected finding with an ID
(`reports/negative-capabilities.json`): `NEG.DOC.STALE_VERSION`, `NEG.DOC.STALE_CLAIM`,
`NEG.DOC.MANUAL_EDIT`, `NEG.DOC.LEGACY_NOT_REPLACED`, `NEG.DOC.INFORMATION_LOSS`,
`NEG.DOC.FRONTDOOR_CONFLICT`, `NEG.DOC.STALE_NONCLAIM`, `NEG.DOC.STALE_RECEIPT_LINK`.

## The gate (`lab/docs/generate.py check`, run by the doc-gate)

Fails if a generated doc ≠ regenerated (hand-edit / staleness), the version drifts from `Cargo.toml`,
`legacy_preservation` is missing or its sha ≠ the actual legacy file, the doc is not an information
superset, or a claim-ladder court is absent from STATUS.

## Status

**STATUS.md is generated** (`docs-src/STATUS.model.json`) — the highest-staleness-risk front-door doc
(it had a stale court count and a stale "FILE.1 deferred" debt; generation corrected both). The framework
is extensible: each further authoritative doc (README, REVIEW-IN-10-MINUTES, not-yet-ready, …) is migrated
the same way — preserve the legacy copy losslessly, author a `.model.json`, generate, gate — one batch at
a time, never moving a doc until its information-superset replacement exists.

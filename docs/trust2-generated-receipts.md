# TRUST.2 — generated receipts, not static receipts

> **Doctrine.** A KOBOLD/GNURUST receipt is not a static claim document. It is generated evidence from
> a replayable court run. Human documentation may summarize receipts, but only generated receipts bind
> a claim.

A receipt with a hand-typed `PASS=120 FAIL=0` is a static artifact of *confidence*. A receipt should be
a reproducible artifact of *replay*:

```
oracle + fixture + gnucobol-rs court  →  replay command  →  machine output  →  receipt.json  →  receipt.md
                                                                              (binding)        (generated)
```

## What is generated

`reports/receipts/<CAMPAIGN>/receipt.json` is the **receipt of record** — produced by running the
court's sweep **live** (`lab/receipt/run.py generate`). It records the campaign, court, crate version,
oracle identity, replay command, the live sweep result, byte domain, non-claims, and verdict.
`receipt.md` is **generated from the JSON** — never hand-edited (the header says so).

The authored `reports/RECEIPT-GNURUST-*.md` files remain the human prose (doctrine / exact claim), but
their stated sweep numbers are **gated against the generated receipts**, so prose cannot drift either.

## The gate (`lab/receipt/run.py check`, run by the doc-gate)

Fails if:
- a generated `receipt.json` evidence differs from a fresh live replay (stale results);
- `receipt.md` != `render(receipt.json)` (a manual edit);
- a `claim-ladder.json` campaign has no generated receipt;
- a static `RECEIPT-*.md` does not state the current live result.

So a publish is blocked when *“README says pass, receipt says pass, but replay would fail today.”*
**The receipt is the replay.**

## Regenerate

```sh
python3 lab/receipt/run.py generate "$(git rev-parse --short HEAD)-replay" "$(git rev-parse --short HEAD)"
python3 lab/receipt/run.py check   # what the doc-gate runs
```

Future: a `reports/receipts/archive/` layer pinning release receipts (hash + crate version + git tag) at
publish time, distinct from the regenerating `current` set.

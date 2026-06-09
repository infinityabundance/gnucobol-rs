# Release verdict — gnucobol-rs 0.7.22

_Generated 2026-06-09T09:54:40Z from the machine files in this packet. A release is an evidence packet, not merely a
version number._

| evidence | value |
|----------|-------|
| crate / version | `gnucobol-rs` 0.7.22 (gnurust) |
| git commit | `b6792d629a30f8e959d7147138dcb6bd764ae236` |
| publish status | pending_crates_io_rate_limit_window |
| this-crate license | LGPL-3.0-or-later |
| dependencies | 2 (SBOM: `sbom.spdx.json`) |
| sealed courts in this crate | 43 (`claim-ladder-snapshot.json`) |
| TRUST.2 receipts | 39 (`receipt-manifest.json`) |
| cargo-audit | **pass** (`cargo-audit.txt`) |
| cargo-geiger | **not_run** (`cargo-geiger.txt`) — every shipped crate is `#![forbid(unsafe_code)]` |

## What this release admits
The sealed courts in `claim-ladder-snapshot.json`, each proven against the admitted GnuCOBOL 3.2 oracle
with a reproducible TRUST.2 receipt.

## What this release does NOT admit
The non-claims in `negative-capabilities-snapshot.json`. **No production-readiness claim** beyond the
KRL level in `status-snapshot.md`. Unavailable tools above are marked honestly (`not_installed` /
`not_run` / `network_unavailable`), never faked green.

> Doctrine: ENTERPRISE.1 treats a release as an evidence packet — reproducible receipts, dependency/
> license inventory, feature flags, audit status, claim boundaries, negative capabilities, and a verdict
> that refuses to overstate production readiness.

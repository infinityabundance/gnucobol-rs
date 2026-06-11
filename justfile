# gnucobol-rs verification entry points. The forensic/court/license METHOD lives in external tools
# (KOBOLD + gpl-license-guard); this repo CONSUMES them and commits the evidence. Runtime crate has NO
# dependency on any of them -- this is a dev/CI/evidence relationship only.

# full self-governance guard (sealed courts + doc-gate + cargo test)
verify:
    bash lab/verify-sealed-courts.sh

# independent GPL/LGPL boundary audit -> committed receipt (requires `cargo install --path ../gpl-license-guard`)
verify-license:
    gpl-license-guard check --root . --policy lgpl-faithful-derivative

# regenerate the committed license receipt
license-receipt:
    gpl-license-guard receipt --root . --policy lgpl-faithful-derivative --out reports/license/gpl-license-guard-receipt.json

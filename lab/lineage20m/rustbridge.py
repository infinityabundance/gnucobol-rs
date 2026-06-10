"""Rust-court bridge for GNURUST.LINEAGE.CORPUS.20M.

For court-addressed (differential) witnesses, compute the gnucobol-rs byte output for the SAME field
model and compare to the oracle. v0 reuses the GREEN value_image mirror (examples/value_rows), batched:
all STORAGE witness specs go through ONE value_rows process (fast) -> {id: bytes_hex | RUST_ERR}.

Only families whose court CLAIMS the shape get a rust comparison; everything else stays oracle/atlas.
We do NOT over-refactor the Rust examples yet (per plan) -- we wrap value_rows and consolidate later."""

import os
import subprocess

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
VALUE_ROWS = os.path.join(ROOT, "target", "release", "examples", "value_rows")


def ensure_built():
    if not os.path.exists(VALUE_ROWS):
        subprocess.run(["cargo", "build", "--release", "-p", "gnucobol-rs", "--examples"],
                       cwd=ROOT, check=True, capture_output=True)
    return os.path.exists(VALUE_ROWS)


def value_mirror(specs):
    """specs: list of value_rows spec lines 'id|01:REC::G:|...'. Returns {id: hex | 'RUST_ERR:<reason>'}."""
    if not specs:
        return {}
    ensure_built()
    inp = ("\n".join(specs) + "\n").encode()
    cp = subprocess.run([VALUE_ROWS], input=inp, capture_output=True)
    out = {}
    for line in cp.stdout.decode("utf-8", "replace").splitlines():
        line = line.strip()
        if not line:
            continue
        parts = line.split(None, 1)
        if len(parts) == 2:
            wid, val = parts
            out[wid] = val.lower() if all(c in "0123456789abcdef" for c in val.lower()) else f"RUST_ERR:{val}"
        else:
            out[parts[0]] = "RUST_ERR:empty"
    return out

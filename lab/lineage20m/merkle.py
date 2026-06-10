"""Merkle hash chain for GNURUST.LINEAGE.CORPUS.20M -- the auditability spine.

A reviewer verifies the corpus without rerunning 20M: each shard receipt carries a Merkle root over
its row hashes; the manifest carries the root-of-roots. verify-merkle recomputes; receipt tamper ->
root mismatch -> fail. Pure sha256 of canonical bytes (no deps)."""

import hashlib


def h(b: bytes) -> str:
    return hashlib.sha256(b).hexdigest()


def leaf(row_canonical: bytes) -> str:
    # domain-separated leaf vs node to resist second-preimage across levels
    return hashlib.sha256(b"\x00" + row_canonical).hexdigest()


def _node(a: str, b: str) -> str:
    return hashlib.sha256(b"\x01" + bytes.fromhex(a) + bytes.fromhex(b)).hexdigest()


def root(leaves):
    """Merkle root of a list of hex leaf hashes. Empty -> sha256(b'')."""
    level = list(leaves)
    if not level:
        return hashlib.sha256(b"").hexdigest()
    while len(level) > 1:
        nxt = []
        for i in range(0, len(level), 2):
            a = level[i]
            b = level[i + 1] if i + 1 < len(level) else level[i]  # duplicate last if odd
            nxt.append(_node(a, b))
        level = nxt
    return level[0]


def root_of_roots(shard_roots):
    """Stable order: sort by shard id is the caller's job; we chain the given order."""
    return root([leaf(r.encode()) for r in shard_roots])

"""Schemas, canonical hashing, and the classification taxonomy for GNURUST.LINEAGE.CORPUS.20M.

Canonical JSON = sorted keys, no whitespace, UTF-8 -> stable sha256 (the hash binding under every row,
shard receipt, and the manifest). The classification taxonomy is the user-refined three-way verdict
model that prevents false-red: only oracle-default vs rust-court is a parity check; variant rows are
lineage discovery, never Rust failures unless the court explicitly claims that variant profile."""

import json
import hashlib

SCHEMA_ROW = "gnurust-lineage20m-row-v1"
SCHEMA_SHARD = "gnurust-lineage20m-shard-receipt-v1"
SCHEMA_RECEIPT = "gnurust-lineage20m-receipt-v1"
GENERATOR_VERSION = "lineage20m-gen-v1"

# witness_kind -- what role this row plays (refinement #2). Not every fixture has a Rust comparison.
WITNESS_KINDS = {"oracle_default", "oracle_variant", "rust_court", "atlas_only", "refusal_probe"}

# classification -- the user-refined three-way verdict taxonomy (refinement #3).
# Reddening (untriaged-class) ONLY: default_mismatch, variant_claimed_and_mismatch, untriaged.
CLASSES = {
    # parity lane (oracle-default vs rust-court) -- the only lane that can redden on a court claim
    "default_match",
    "default_mismatch",          # the prize: a real Rust<->oracle divergence -> shrink + feed PUBLIC.GAP.1
    # variant lane (oracle-default vs oracle-variant) -- lineage DISCOVERY, never a Rust failure
    "variant_same_as_default",
    "variant_differs_from_default",
    "variant_not_claimed_by_rust",
    "variant_claimed_and_match",
    "variant_claimed_and_mismatch",
    # lineage data (not errors)
    "compile_fail",
    "compile_diagnostic_delta",
    "runtime_delta",
    "known_gap",                 # rust typed-Err matching a NEG.* refusal
    "new_cluster",               # oracle behavior not yet in the registry -- SUCCESS, must be named
    "atlas_cluster",             # observed-atlas family, oracle clustered
    # the only non-shrunk failure state
    "untriaged",
}
# Classes whose presence (unshrunk / unnamed) FAILS the gate.
REDDENING = {"default_mismatch", "variant_claimed_and_mismatch", "untriaged"}


def canon(obj) -> bytes:
    return json.dumps(obj, sort_keys=True, separators=(",", ":"), ensure_ascii=False).encode("utf-8")


def sha(obj) -> str:
    return hashlib.sha256(canon(obj)).hexdigest()


def sha_bytes(b: bytes) -> str:
    return hashlib.sha256(b).hexdigest()


def validate_row(r: dict):
    assert r["schema"] == SCHEMA_ROW, r.get("schema")
    assert r["witness_kind"] in WITNESS_KINDS, r["witness_kind"]
    assert r["classification"] in CLASSES, r["classification"]
    for k in ("id", "seed", "generator", "surface", "court_target", "build_profile_sha256",
              "dialect", "oracle", "classification"):
        assert k in r, f"row missing {k}"
    return True

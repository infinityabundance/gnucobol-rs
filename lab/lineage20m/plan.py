"""Deterministic shard plan for GNURUST.LINEAGE.CORPUS.20M.

The plan IS the contract: it assigns each of N shards a per-family witness budget derived from the
lineage-weighted distribution (user's revised, gap-board-aligned). Witnesses are regenerated from the
plan byte-identically; no .cob is stored. Every family maps to a court/atlas/refusal owner."""

# (family, budget_at_20M, court_target, surface, mode)
#   mode: "differential" (oracle-default vs rust-court parity) | "variant" (dialect/directive lineage)
#         | "atlas" (observed-only clustering)
FAMILIES = [
    ("storage",          2_000_000, "GNURUST.8/2/14",          "MOVE/storage",            "differential"),
    ("pic_layout",       1_000_000, "GNURUST.4/3/9/10",        "PIC/layout",              "differential"),
    ("copy_replace",     1_000_000, "GNURUST.5/6",             "COPY/REPLACING",          "differential"),
    ("arithmetic",       2_000_000, "GNURUST.7/13/19",         "arithmetic",              "differential"),
    ("init_inspect_str", 2_000_000, "GNURUST.STRING.UNSTRING.1","INITIALIZE/INSPECT/STRING","differential"),
    ("intrinsics",       1_500_000, "GNURUST.INTRINSIC",       "intrinsics",              "differential"),
    ("accept_display",   1_500_000, "GNURUST.ACCEPT.DISPLAY.1","ACCEPT/DISPLAY",          "differential"),
    ("file_seq",         1_500_000, "GNURUST.FILE.SEQUENTIAL.1","file-io-sequential",     "differential"),
    ("flow_slices",      1_500_000, "GNURUST.PERFORM.SLICE.1", "procedure-flow",          "differential"),
    ("search_table",       800_000, "GNURUST.SEARCH.TABLE.1",  "procedure-flow",          "differential"),
    ("call_atlas",         800_000, "GNURUST.CALL.LAYOUT.ATLAS.1","CALL/linkage",         "atlas"),
    ("file_atlas",       1_200_000, "GNURUST.INDEXED.FILE.ATLAS.1","file-io-indexed",     "atlas"),
    ("directive_matrix", 2_000_000, "GNURUST.DIRECTIVE.VARIANCE.ATLAS.1","dialect-runtime","variant"),
    ("public_shapes",      500_000, "GNURUST.PUBLIC.CORPUS.1", "public-mined",            "atlas"),
    ("adversarial",        700_000, "GNURUST.CROSS",           "cross-court",             "variant"),
]

TOTAL = sum(f[1] for f in FAMILIES)
assert TOTAL == 20_000_000, TOTAL

# Families IMPLEMENTED in the v0 engine (the others are planned-but-not-yet-generating; logged as
# dropped buckets with a reason so "20M" stays honest about what the v0 engine actually covers).
IMPLEMENTED = {"storage", "directive_matrix"}


def shard_plan(total: int, n_shards: int, seed_base0: int = 0x1234_5678_9ABC_DEF0):
    """Return list of shard dicts. Budgets split evenly (last shard takes the remainder)."""
    per = total // n_shards
    shards = []
    assigned = 0
    for s in range(n_shards):
        count = per if s < n_shards - 1 else (total - assigned)
        assigned += count
        shards.append({
            "shard_id": s,
            "count": count,
            "seed_base": (seed_base0 + s * 0x9E3779B97F4A7C15) & ((1 << 64) - 1),
        })
    return shards


def family_for_index(global_index: int):
    """Map a global witness index -> (family, court_target, surface, mode) by stratified blocks.

    Deterministic: index space is partitioned into contiguous family blocks scaled to budget so the
    realized distribution matches FAMILIES exactly. Index is taken mod TOTAL (cyclic for >20M probes)."""
    i = global_index % TOTAL
    acc = 0
    for fam, budget, court, surface, mode in FAMILIES:
        if i < acc + budget:
            return fam, court, surface, mode
        acc += budget
    f = FAMILIES[-1]
    return f[0], f[2], f[3], f[4]

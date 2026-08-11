# Mutation / metamorphic testing (Phase 10.5)

30 bases, 56 variants (56 equivalent, 0 divergent, 94 skipped); every run bounded at 2s.

Only defensible transformations are claimed; anything not provably safe is
skipped with a recorded reason. Divergent variants are reported honestly.
See `mutation-results.json` for the per-base variant rows.

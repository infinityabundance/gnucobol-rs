"""Deterministic Lehmer LCG -- byte-identical to the Rust generators' RNG
(crates/gnucobol-rs/examples/gen_value.rs: state = state*6364136223846793005 + 1442695040888963407; >> 16).

The generator IS the corpus: every witness is reproducible from (seed_base, index) with NO stored .cob.
This is the determinism spine of GNURUST.LINEAGE.CORPUS.20M -- no Math.random, no clock, ever."""

MASK64 = (1 << 64) - 1
MUL = 6364136223846793005
INC = 1442695040888963407


class Lcg:
    __slots__ = ("state",)

    def __init__(self, seed: int):
        self.state = seed & MASK64

    def step(self) -> int:
        self.state = (self.state * MUL + INC) & MASK64
        return self.state >> 16

    def below(self, n: int) -> int:
        return self.step() % max(1, n)

    def pick(self, seq):
        return seq[self.below(len(seq))]


# The project-wide golden constant (gen_value.rs base); witness seed = mix(seed_base, index).
GOLDEN = 0x9E3779B97F4A7C15


def witness_seed(seed_base: int, index: int) -> int:
    """Stable per-witness seed. SplitMix-style avalanche so adjacent indices diverge."""
    z = (seed_base + index * 0x9E3779B97F4A7C15) & MASK64
    z = ((z ^ (z >> 30)) * 0xBF58476D1CE4E5B9) & MASK64
    z = ((z ^ (z >> 27)) * 0x94D049BB133111EB) & MASK64
    return (z ^ (z >> 31)) & MASK64

"""DIRECTIVE_MATRIX family (the crown jewel) -- compiler-directive lineage deltas (variant mode).

The SAME source is compiled under the default profile AND under a -f variant; the engine clusters the
byte/length delta. Mirrors the proven GNURUST.DIRECTIVE.VARIANCE.ATLAS.1 axes, now at corpus scale:
  size:      DISPLAY LENGTH OF a COMP group        -> default 7 vs -fbinary-size=2-4-8 8
  byteorder: raw bytes of a COMP field (REDEFINES) -> big-endian vs -fbinary-byteorder=native
  truncate:  MOVE over-range into 9(2)COMP         -> -fbinary-truncate(default) vs -fno-binary-truncate

These are oracle-default vs oracle-variant rows: lineage DISCOVERY, never a Rust failure. The Rust binary
court (GNURUST.14) claims the DEFAULT profile only, so variant rows are variant_* clusters by design."""
from ..lcg import Lcg

_AXES = ["size", "size", "byteorder", "byteorder", "truncate"]


def gen(wseed: int, wid: str, court: str, surface: str) -> dict:
    rng = Lcg(wseed)
    axis = rng.pick(_AXES)
    pid = "P" + wid.replace("-", "")
    if axis == "size":
        d2 = rng.below(2) + 1
        d4 = rng.below(2) + 3
        d9 = rng.below(4) + 5
        body = (
            "       01 REC.\n"
            f"          05 A PIC 9({d2}) COMP VALUE 7.\n"
            f"          05 B PIC 9({d4}) COMP VALUE 7.\n"
            f"          05 C PIC 9({d9}) COMP VALUE 7.\n"
            "       PROCEDURE DIVISION.\n"
            "           DISPLAY LENGTH OF REC WITH NO ADVANCING.\n"
        )
        variant_flags = ["-fbinary-size=2-4-8"]
        dialect = "directive:binary-size=2-4-8"
        shape = f"directive:size:9({d2})/9({d4})/9({d9})"
    elif axis == "byteorder":
        val = rng.below(60000) + 1
        body = (
            f"       01 N PIC 9(4) COMP VALUE {val}.\n"
            "       01 R REDEFINES N PIC X(2).\n"
            "       PROCEDURE DIVISION.\n"
            "           DISPLAY R WITH NO ADVANCING.\n"
        )
        variant_flags = ["-fbinary-byteorder=native"]
        dialect = "directive:binary-byteorder=native"
        shape = f"directive:byteorder:9(4)COMP={val}"
    else:  # truncate
        over = rng.below(900) + 100  # 100..999 -> overflows 9(2)
        body = (
            "       01 N PIC 9(2) COMP.\n"
            "       PROCEDURE DIVISION.\n"
            f"           MOVE {over} TO N.\n"
            "           DISPLAY N WITH NO ADVANCING.\n"
        )
        variant_flags = ["-fno-binary-truncate"]
        dialect = "directive:no-binary-truncate"
        shape = f"directive:truncate:9(2)COMP<-{over}"
    cob = (
        "       IDENTIFICATION DIVISION.\n"
        f"       PROGRAM-ID. {pid}.\n"
        "       DATA DIVISION.\n"
        "       WORKING-STORAGE SECTION.\n"
        + body +
        "           STOP RUN.\n"
    )
    return {
        "id": wid, "generator": "directive_matrix", "surface": surface, "court_target": court,
        "witness_kind": "oracle_variant", "mode": "variant", "dialect": dialect,
        "base_flags": [], "variant_flags": variant_flags, "cob": cob, "rust_spec": None,
        "record_name": "REC", "shape_key": shape,
    }

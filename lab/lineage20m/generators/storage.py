"""STORAGE family -- MOVE / VALUE / numeric-representation witnesses (court GNURUST.8/2).

mode=differential: oracle compiles+runs a VALUE record (DISPLAY REC raw bytes) and the Rust value_image
mirror (value_rows) computes the same bytes -> default_match / default_mismatch. This reuses the GREEN
value_sweep semantics (PASS=392/0) so the parity lane is honest from witness #1.

Witness contract (consumed by oracle.py + rustbridge.py):
  {id, generator, surface, court_target, witness_kind, mode, dialect, base_flags, variant_flags,
   cob (source text), rust_spec (value_rows line) | None, record_name, shape_key}
"""
from ..lcg import Lcg

# (pic, usage_char D|C, signed, digits, scale) -- DISPLAY zoned + COMP-3 packed only (value-court green set)
SHAPES = [
    ("9(3)", "D", False, 3, 0), ("S9(3)", "D", True, 3, 0),
    ("9(2)", "D", False, 2, 0), ("S9(1)", "D", True, 1, 0),
    ("9(4)V99", "D", False, 6, 2), ("S9(4)V99", "D", True, 6, 2),
    ("9(5)", "C", False, 5, 0), ("S9(5)", "C", True, 5, 0),
    ("S9(3)V99", "C", True, 5, 2), ("9(7)V9", "C", False, 8, 1),
    ("S9(7)V9", "C", True, 8, 1), ("9(9)", "C", False, 9, 0),
    ("S9(11)V99", "C", True, 13, 2), ("9(18)", "D", False, 18, 0),
]
ALNUM = ["X(1)", "X(3)", "X(5)", "X(8)"]
# corner values exercised deterministically (financial-plausibility + boundary weighting)
VALUE_KINDS = ["rand", "rand", "rand", "zero", "max9", "maxneg", "unvalued", "figZ", "figS"]


def _num_literal(rng: Lcg, digits: int, scale: int, signed: bool, kind: str):
    intd = digits - scale
    if kind == "zero":
        body = "0"
    elif kind == "max9":
        body = "9" * intd + ("." + "9" * scale if scale else "")
        return ("-" if signed else "") + body if False else body  # max positive magnitude
    elif kind == "maxneg":
        if not signed:
            return "9" * intd + ("." + "9" * scale if scale else "")
        return "-" + "9" * intd + ("." + "9" * scale if scale else "")
    else:  # rand
        nintd = rng.below(intd) + 1
        s = "".join(str(rng.below(10)) for _ in range(nintd))
        if scale:
            s += "." + "".join(str(rng.below(10)) for _ in range(scale))
        if signed and rng.below(2) == 0:
            s = "-" + s
        return s
    if signed and kind == "max9" and rng.below(2) == 0:
        body = "-" + body
    return body


def _alnum_literal(rng: Lcg, width: int):
    n = rng.below(width) + 1
    return "".join(chr(ord("A") + rng.below(26)) for _ in range(n))


def gen(wseed: int, wid: str, court: str, surface: str) -> dict:
    rng = Lcg(wseed)
    nfields = rng.below(4) + 1
    spec_fields = ["01:REC::G:"]
    cob_fields = ["       01 REC."]
    for fi in range(nfields):
        name = f"F{fi+1}"
        if rng.below(5) == 0:
            pic = rng.pick(ALNUM)
            width = int(pic[2:-1])
            kind = rng.pick(["rand", "unvalued", "figS"])
            if kind == "unvalued":
                val_code = ""; cob_val = ""
            elif kind == "figS":
                val_code = "S"; cob_val = " VALUE SPACE"
            else:
                lit = _alnum_literal(rng, width); val_code = "A" + lit; cob_val = f' VALUE "{lit}"'
            spec_fields.append(f"05:{name}:{pic}:D:{val_code}")
            cob_fields.append(f"       05 {name} PIC {pic}{cob_val}.")
        else:
            pic, usage, signed, digits, scale = rng.pick(SHAPES)
            kind = rng.pick(VALUE_KINDS)
            if kind == "unvalued":
                val_code = ""; cob_val = ""
            elif kind in ("figZ", "figS"):
                # numeric fields take ZERO, never SPACE (SPACE is alphanumeric-only)
                val_code = "Z"; cob_val = " VALUE ZERO"
            else:
                lit = _num_literal(rng, digits, scale, signed, kind)
                val_code = "N" + lit; cob_val = f" VALUE {lit}"
            uchar = "C" if usage == "C" else "D"
            usage_clause = " USAGE COMP-3" if usage == "C" else ""
            spec_fields.append(f"05:{name}:{pic}:{uchar}:{val_code}")
            cob_fields.append(f"       05 {name} PIC {pic}{usage_clause}{cob_val}.")
    rust_spec = wid + "|" + "|".join(spec_fields)
    cob = (
        "       IDENTIFICATION DIVISION.\n"
        f"       PROGRAM-ID. P{wid.replace('-','')}.\n"
        "       DATA DIVISION.\n"
        "       WORKING-STORAGE SECTION.\n"
        + "\n".join(cob_fields) + "\n"
        "       PROCEDURE DIVISION.\n"
        "           DISPLAY REC WITH NO ADVANCING.\n"
        "           STOP RUN.\n"
    )
    shape_key = f"storage:{nfields}f:" + ",".join(s.split(":")[2] for s in spec_fields[1:])
    return {
        "id": wid, "generator": "storage", "surface": surface, "court_target": court,
        "witness_kind": "rust_court", "mode": "differential", "dialect": "default",
        "base_flags": [], "variant_flags": None, "cob": cob, "rust_spec": rust_spec,
        "record_name": "REC", "shape_key": shape_key,
    }

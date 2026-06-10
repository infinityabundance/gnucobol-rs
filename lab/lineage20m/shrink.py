"""Deterministic shrinker for GNURUST.LINEAGE.CORPUS.20M.

A default_mismatch is only useful if it shrinks to a minimal reproducer. This reduces a STORAGE witness
(the differential family) to the smallest record that still diverges oracle-vs-rust, preserving the
classification. The minimal .cob + a root-cause signature are filed; many raw mismatches collapse to ONE
finding (e.g. all S9(n) COMP-3 VALUE -0 cases -> a single `value-negzero-comp3-integer` finding)."""

import os
from . import oracle, rustbridge

# minimal building blocks tried during shrink (smallest first)
_MIN_PICS = ["S9(1)", "9(1)", "S9(1)V9", "X(1)"]


def _spec_to_cob(fields, pid="PSHRINK"):
    cob_fields = ["       01 REC."]
    for f in fields:
        level, name, pic, uchar, val = f.split(":", 4)
        usage = " USAGE COMP-3" if uchar == "C" else ""
        if val == "":
            cv = ""
        elif val.startswith("N"):
            cv = f" VALUE {val[1:]}"
        elif val.startswith("A"):
            cv = f' VALUE "{val[1:]}"'
        elif val == "Z":
            cv = " VALUE ZERO"
        elif val == "S":
            cv = " VALUE SPACE"
        else:
            cv = ""
        if uchar == "G":
            continue
        cob_fields.append(f"       05 {name} PIC {pic}{usage}{cv}.")
    return (
        "       IDENTIFICATION DIVISION.\n"
        f"       PROGRAM-ID. {pid}.\n"
        "       DATA DIVISION.\n"
        "       WORKING-STORAGE SECTION.\n"
        + "\n".join(cob_fields) + "\n"
        "       PROCEDURE DIVISION.\n"
        "           DISPLAY REC WITH NO ADVANCING.\n"
        "           STOP RUN.\n"
    )


def _mismatches(fields, scratch, tag):
    """True iff this REC spec compiles+runs and oracle bytes != rust value_image bytes."""
    spec = "SHR|01:REC::G:|" + "|".join(fields)
    cob = _spec_to_cob(fields)
    orc = oracle._compile_run(cob, [], scratch, tag)
    if orc["compile_status"] != "pass":
        return False, None, None
    rust = rustbridge.value_mirror([spec]).get("SHR")
    if rust is None or rust.startswith("RUST_ERR"):
        return False, orc.get("bytes_hex"), rust
    return (rust != orc.get("bytes_hex")), orc.get("bytes_hex"), rust


def shrink_storage(witness, scratch):
    """Return (minimal_fields, root_cause, minimal_cob, oracle_hex, rust_hex) or None if not reproducible."""
    # spec fields excluding the group header
    fields = [f for f in witness["rust_spec"].split("|")[1:] if not f.endswith(":G:") and ":G:" not in f]
    os.makedirs(scratch, exist_ok=True)
    bad, oh, rh = _mismatches(fields, scratch, "shr_full")
    if not bad:
        return None
    # 1) reduce to the single culprit field
    cur = fields
    changed = True
    while changed and len(cur) > 1:
        changed = False
        for i in range(len(cur)):
            cand = cur[:i] + cur[i + 1:]
            b, o2, r2 = _mismatches(cand, scratch, f"shr_drop{i}")
            if b:
                cur, oh, rh = cand, o2, r2
                changed = True
                break
    # 2) minimize the surviving field's pic/value. Try CANONICAL minimal values FIRST so equivalent
    # cases collapse to ONE finding: an integer negative-zero ("N-0") and a scaled one ("N-0.0") are
    # DIFFERENT lineage behaviors (cobc canonicalizes the integer, preserves the scaled), so the order
    # below separates them deterministically rather than keeping the witness's accidental "N-00"/"N-000".
    level, name, pic, uchar, val = cur[0].split(":", 4)
    for mp in _MIN_PICS:
        mu = "C" if uchar == "C" else "D"
        for mv in ["N-0", "N0", "N-0.0", val]:
            cand = [f"05:F:{mp}:{mu}:{mv}"]
            b, o2, r2 = _mismatches(cand, scratch, "shr_min")
            if b:
                cur, oh, rh = cand, o2, r2
                level, name, pic, uchar, val = "05", "F", mp, mu, mv
                break
        else:
            continue
        break
    root_cause = f"value-negzero|{pic}|{('comp3' if uchar=='C' else 'display')}|{val}"
    return cur, root_cause, _spec_to_cob(cur), oh, rh

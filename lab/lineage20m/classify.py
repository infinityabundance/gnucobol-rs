"""Classification + behavior-cluster registry for GNURUST.LINEAGE.CORPUS.20M.

The user-refined three-way verdict taxonomy. Only oracle-default vs rust-court is a parity check that
can redden (default_mismatch). Variant rows (oracle-default vs oracle-variant) are lineage DISCOVERY --
clusters, never Rust failures (unless the court explicitly claims that variant -> variant_claimed_*).
compile_fail / known_gap / atlas_cluster / new_cluster are lineage data, not errors."""

REDDENING = {"default_mismatch", "variant_claimed_and_mismatch", "untriaged"}


def behavior_cluster(w: dict, orc: dict, variant: dict = None) -> str:
    """A deterministic descriptor of the observed oracle behavior -- the topology key."""
    fam = w["generator"]
    if orc["compile_status"] == "fail":
        err = (orc.get("stderr_head") or "").split("error:")[-1].strip()
        tok = (err.splitlines()[0] if err else "").strip()[:40]
        return f"{fam}|compile_fail|{tok or 'unknown'}"
    if orc.get("exit") == "timeout":
        return f"{fam}|runtime_timeout"
    nbytes = len(orc["bytes_hex"] or "") // 2
    if w.get("mode") == "variant" and variant is not None:
        if variant["compile_status"] == "fail":
            return f"{fam}|variant_compile_fail"
        same = variant.get("bytes_hex") == orc.get("bytes_hex")
        return f"{fam}|variant_{'same' if same else 'differs'}|{w.get('dialect','?')}|len{nbytes}"
    return f"{fam}|ok|len{nbytes}"


def classify(w: dict, orc: dict, rust_hex: str = None, variant: dict = None) -> str:
    mode = w.get("mode")
    if orc["compile_status"] == "fail":
        return "compile_fail"
    if orc.get("exit") == "timeout":
        return "runtime_delta"

    if mode == "differential":
        if rust_hex is None:
            return "untriaged"  # court-addressed but no rust output captured -> must triage
        if rust_hex.startswith("RUST_ERR"):
            # the value court fails closed on shapes it does not admit -> a known, admitted gap
            return "known_gap"
        return "default_match" if rust_hex == orc.get("bytes_hex") else "default_mismatch"

    if mode == "variant":
        if variant is None:
            return "variant_not_claimed_by_rust"
        if variant["compile_status"] == "fail" and orc["compile_status"] == "pass":
            return "compile_diagnostic_delta"
        if variant.get("bytes_hex") == orc.get("bytes_hex"):
            return "variant_same_as_default"
        return "variant_differs_from_default"

    # atlas / observed-only
    return "atlas_cluster"

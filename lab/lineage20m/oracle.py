"""Oracle runner -- the evidence-producing act of GNURUST.LINEAGE.CORPUS.20M.

Real cobc compile + libcob run, raw-byte capture. This is the expensive part and it is NOT avoided: the
mass real-cobc compile/run IS the artifact. Parallel across cores; tmpfs scratch; deterministic env.

A witness with mode in {differential, atlas} compiles once (base_flags). A variant witness compiles
twice (base_flags + variant_flags) so the row can classify oracle-default vs oracle-variant lineage."""

import os
import subprocess
import hashlib

PREFIX = os.path.join(os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__)))),
                      "lab", "oracle", "prefix")


def oracle_env():
    e = dict(os.environ)
    e["PATH"] = os.path.join(PREFIX, "bin") + os.pathsep + e.get("PATH", "")
    e["LD_LIBRARY_PATH"] = os.path.join(PREFIX, "lib")
    e["COB_CONFIG_DIR"] = os.path.join(PREFIX, "share", "gnucobol", "config")
    e["COB_COPY_DIR"] = os.path.join(PREFIX, "share", "gnucobol", "copy")
    e["LC_ALL"] = "C.UTF-8"
    e["TERM"] = "dumb"
    return e


_ENV = oracle_env()
_COBC = os.path.join(PREFIX, "bin", "cobc")


def _sha(b: bytes) -> str:
    return hashlib.sha256(b).hexdigest()


def _compile_run(cob_text: str, flags, scratch: str, tag: str):
    """Compile cob_text under flags, run it, capture raw stdout bytes. Returns a dict.

    Each call gets its OWN workdir + COB_TMPDIR so parallel cobc invocations never collide on
    intermediate temp files (a real race that corrupts bytes -- the cause of replay non-determinism)."""
    import shutil
    workdir = os.path.join(scratch, tag + "_w")
    os.makedirs(workdir, exist_ok=True)
    src = os.path.join(workdir, "p.cob")
    binp = os.path.join(workdir, "p")
    with open(src, "w") as f:
        f.write(cob_text)
    env = dict(_ENV)
    cobtmp = os.path.join(workdir, "cobtmp")
    os.makedirs(cobtmp, exist_ok=True)
    # set ALL temp vars -- cobc respects one but ambiguity is the enemy of determinism
    env["COB_TMPDIR"] = cobtmp
    env["TMPDIR"] = cobtmp
    env["TEMP"] = cobtmp
    env["TMP"] = cobtmp
    cmd = [_COBC, "-free", "-x"] + list(flags) + ["-o", binp, "p.cob"]
    cp = subprocess.run(cmd, env=env, capture_output=True, cwd=workdir)
    res = {"compile_status": "pass" if cp.returncode == 0 else "fail",
           "compile_rc": cp.returncode, "stderr_sha256": _sha(cp.stderr),
           "stderr_head": cp.stderr.decode("utf-8", "replace")[:200]}
    if cp.returncode != 0:
        res.update({"exit": None, "stdout_sha256": None, "bytes_hex": None, "bytes_sha256": None})
        shutil.rmtree(workdir, ignore_errors=True)
        return res
    try:
        rp = subprocess.run([binp], env=env, capture_output=True, timeout=10, cwd=workdir)
        out = rp.stdout
        res.update({"exit": rp.returncode, "stdout_sha256": _sha(out),
                    "bytes_hex": out.hex(), "bytes_sha256": _sha(out)})
    except subprocess.TimeoutExpired:
        res.update({"exit": "timeout", "stdout_sha256": None, "bytes_hex": None, "bytes_sha256": None})
    shutil.rmtree(workdir, ignore_errors=True)
    return res


def run_witness(w: dict, scratch: str) -> dict:
    """Compile+run a witness. Returns {oracle:{...}, oracle_variant:{...}|None}."""
    base = _compile_run(w["cob"], w.get("base_flags") or [], scratch, w["id"] + "_d")
    out = {"oracle": base, "oracle_variant": None}
    if w.get("mode") == "variant" and w.get("variant_flags") is not None:
        out["oracle_variant"] = _compile_run(w["cob"], w["variant_flags"], scratch, w["id"] + "_v")
    return out

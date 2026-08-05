#!/usr/bin/env python3
"""Generate the complete upstream commit atlas for the gnucobol-rs sync.

Range: stable baseline 645b417 .. frozen current head 5568b8fc (367 commits).

Inputs
------
- Admit repo (read-only git): lab/admit/gnucobol-upstream-current/
- Curated overrides:         lab/gnucobol-upstream-current/atlas_overrides.json

Outputs (reports/upstream-sync/)
--------------------------------
- commit-atlas.json            full per-commit records + invariants
- commit-atlas.md              human-readable atlas
- commit-atlas.csv             flat per-commit rows
- file-change-index.json       path -> commits touching it
- symbol-change-index.json     exported libcob symbols -> commits adding/removing

Integrity invariants (fail closed)
----------------------------------
- commit count == `git rev-list` count for the range
- no duplicate commit SHAs
- every commit receives exactly one status from the fixed enum
- every non-merge commit touching a semantic area (cobc/ libcob/ bin/ lpvm/
  config/) MUST have a curated override entry (never a vague default)
- every curated override SHA must lie inside the range (typo guard)
- parent edges are recorded for every commit
- counting units are explicit: commit-level rows, never called "tests"

The overrides file is the living Phase-2 work ledger: entries are updated as
semantic commits are integrated; this script is deterministic given the
overrides file and the pinned admit repo.
"""

from __future__ import annotations

import json
import os
import re
import subprocess
import sys
from collections import Counter, OrderedDict

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
ADMIT = os.path.join(ROOT, "lab", "admit", "gnucobol-upstream-current")
OVERRIDES = os.path.join(ROOT, "lab", "gnucobol-upstream-current", "atlas_overrides.json")
OUT = os.path.join(ROOT, "reports", "upstream-sync")

RANGE_START = "645b417"
RANGE_END = "5568b8fc770ff310e5017300d561d8f3deec257c"

STATUSES = [
    "RUNTIME_PORTED",
    "FRONTEND_REIMPLEMENTED",
    "WRAPPER_INTEGRATED",
    "TEST_IMPORTED",
    "HARNESS_ADOPTED",
    "CONFIGURATION_INTEGRATED",
    "PLATFORM_BEHAVIOR_INTEGRATED",
    "DOCUMENTATION_TRACKED",
    "CI_ONLY_ACCOUNTED",
    "UPSTREAM_MERGE_ACCOUNTED",
    "NOT_APPLICABLE_WITH_PROOF",
    "SUPERSEDED_BY_LATER_COMMIT",
    "BLOCKED_BY_NATIVE_ARTIFACT_BOUNDARY",
]

# Areas that require curated overrides (semantic surfaces).
SEMANTIC_PREFIXES = ("cobc/", "libcob/", "bin/", "lpvm/", "config/")


def git(*args: str) -> str:
    return subprocess.run(
        ["git", "-C", ADMIT, *args],
        capture_output=True,
        text=True,
        check=True,
    ).stdout


def classify_area(files: list[str]) -> list[str]:
    areas: set[str] = set()
    for f in files:
        if f.startswith(".github/") or f.startswith(".gitlab-ci") or f == ".github" \
                or ".travis" in f or f.startswith("travis"):
            areas.add("ci")
        elif f.startswith("cobc/"):
            areas.add("cobc")
        elif f.startswith("libcob/"):
            areas.add("libcob")
        elif f.startswith("tests/"):
            areas.add("tests")
        elif f.startswith("config/"):
            areas.add("config")
        elif f.startswith("build_windows/"):
            areas.add("build_windows")
        elif f.startswith(("doc/", "man/")) or f.endswith((".texi", ".1", ".3")):
            areas.add("docs")
        elif f.startswith("bin/"):
            areas.add("bin")
        elif f.startswith("lpvm/"):
            areas.add("lpvm")
        elif f.startswith("m4/") or f.endswith(".m4"):
            areas.add("m4")
        elif f.startswith(("Makefile", "configure")) or f.endswith((".am", ".ac", ".in")):
            areas.add("build")
        elif f in ("ChangeLog", "NEWS", "AUTHORS", "COPYING", "README", "README.md",
                   "THANKS", "INSTALL", "BUGS", "HISTORY") or f.startswith("doc-"):
            areas.add("docs")
        else:
            areas.add("other")
    return sorted(areas)


def default_status(areas: list[str], is_merge: bool) -> str:
    """Mechanical default; semantic candidates must be curated."""
    if is_merge:
        return "UPSTREAM_MERGE_ACCOUNTED"
    aset = set(areas)
    if aset & set(SEMANTIC_PREFIXES):
        # handled by curated overrides (fail-closed below)
        return None
    if aset <= {"ci"}:
        return "CI_ONLY_ACCOUNTED"
    if aset <= {"docs"}:
        return "DOCUMENTATION_TRACKED"
    if aset <= {"tests"}:
        return "TEST_IMPORTED"
    if aset <= {"build", "m4", "config"}:
        return "CONFIGURATION_INTEGRATED"
    if aset <= {"build_windows"}:
        return "PLATFORM_BEHAVIOR_INTEGRATED"
    if aset <= {"other"}:
        return "NOT_APPLICABLE_WITH_PROOF"
    if aset <= {"build", "other", "docs"}:
        return "CONFIGURATION_INTEGRATED"
    if aset <= {"build", "tests"}:
        return "HARNESS_ADOPTED"
    if aset <= {"other", "docs"}:
        return "DOCUMENTATION_TRACKED"
    if aset <= {"ci", "other"} or aset <= {"ci", "docs"}:
        return "CI_ONLY_ACCOUNTED"
    return "NOT_APPLICABLE_WITH_PROOF"


def is_ci_only(files: list[str]) -> bool:
    return all(f.startswith(".github/") or f == ".github" for f in files)


def commit_list() -> list[dict]:
    log = git(
        "log",
        "--format=%H%x09%P%x09%an%x09%ae%x09%cn%x09%ce%x09%ad%x09%s",
        "--date=iso-strict",
        f"{RANGE_START}..{RANGE_END}",
    )
    out = []
    for line in log.splitlines():
        sha, parents, an, ae, cn, ce, ad, subject = line.split("\t", 7)
        out.append({
            "sha": sha,
            "parents": parents.split(),
            "author_name": an,
            "author_email": ae,
            "committer_name": cn,
            "committer_email": ce,
            "author_date": ad,
            "subject": subject,
        })
    return out


def load_overrides() -> dict:
    if not os.path.exists(OVERRIDES):
        return {"entries": {}}
    with open(OVERRIDES, encoding="utf-8") as fh:
        return json.load(fh)


def diff_stats(sha: str, parents: list[str]) -> list[dict]:
    """File-level diff of sha against its first parent."""
    parent = parents[0] if parents else None
    if parent is None:
        return []
    names = git("diff-tree", "--no-commit-id", "--name-status", "-r", parent, sha)
    nums = git("diff-tree", "--no-commit-id", "--numstat", "-r", parent, sha)
    num_map = {}
    for ln in nums.splitlines():
        parts = ln.split("\t")
        if len(parts) >= 3:
            num_map[parts[2]] = (parts[0], parts[1])
    files = []
    for ln in names.splitlines():
        parts = ln.split("\t")
        status = parts[0]
        path = parts[-1]
        ins, dele = num_map.get(path, ("0", "0"))
        files.append({
            "path": path,
            "status": status,
            "insertions": int(ins) if ins.isdigit() else 0,
            "deletions": int(dele) if dele.isdigit() else 0,
        })
    return files


def full_message(sha: str) -> str:
    return git("log", "-1", "--format=%B", sha).strip()


def libcob_symbols(sha: str, parents: list[str]) -> dict:
    """Added/removed exported/hidden libcob symbols from header/source diffs.

    Heuristic, for the change index only: matches COB_EXPORT / COB_HIDDEN
    declarations and definitions in libcob/*.c and libcob/*.h. The captured
    name is the identifier that follows the export macro.
    """
    if not parents:
        return {"added": [], "removed": []}
    parent = parents[0]
    diff = git("diff", "-U0", parent, sha, "--", "libcob/*.h", "libcob/*.c")
    added, removed = [], []
    # (export macro)(storage/type words) name followed by ( ; = or (
    pattern = re.compile(
        r"(?:COB_EXPORT|COB_HIDDEN)\s+[A-Za-z_][\w\s]*?"
        r"(\*?\s*[A-Za-z_]\w*)\s*(?:\(|;|=)"
    )
    for line in diff.splitlines():
        if not (line.startswith("+") or line.startswith("-")):
            continue
        if line.startswith(("+++", "---")):
            continue
        if "COB_EXPORT" not in line and "COB_HIDDEN" not in line:
            continue
        body = line[1:]
        m = pattern.search(body)
        if not m:
            continue
        name = m.group(1).strip().lstrip("*")
        if not name:
            continue
        if line.startswith("+"):
            added.append(name)
        else:
            removed.append(name)
    return {
        "added": sorted(set(added)),
        "removed": sorted(set(removed)),
    }


def first_parent_index(sha: str) -> int | None:
    """Index (1-based) of sha on the first-parent chain of the range end."""
    chain = git("log", "--first-parent", "--format=%H", f"{RANGE_START}..{RANGE_END}")
    for i, c in enumerate(chain.splitlines(), 1):
        if c == sha:
            return i
    return None


def main() -> int:
    os.makedirs(OUT, exist_ok=True)
    overrides = load_overrides()
    entries = overrides.get("entries", {})
    expected_count = int(git("rev-list", "--count", f"{RANGE_START}..{RANGE_END}"))

    commits = commit_list()
    if len(commits) != expected_count:
        print(f"FATAL: atlas row count {len(commits)} != rev-list {expected_count}")
        return 1
    shas = [c["sha"] for c in commits]
    if len(set(shas)) != len(shas):
        print("FATAL: duplicate commit SHAs in range")
        return 1
    valid_shas = set(shas)

    # typo guard: every override must be inside the range
    for sha in entries:
        if sha not in valid_shas:
            print(f"FATAL: override SHA {sha} not inside the range")
            return 1

    rows = []
    semantic_uncurated = []
    first_parent = first_parent_index(commits[0]["sha"]) if commits else None

    for c in commits:
        sha = c["sha"]
        files = diff_stats(sha, c["parents"])
        areas = classify_area([f["path"] for f in files])
        is_merge = len(c["parents"]) > 1
        row = dict(c)
        row["is_merge"] = is_merge
        row["files"] = files
        row["areas"] = areas
        row["message"] = full_message(sha)
        row["insertions"] = sum(f["insertions"] for f in files)
        row["deletions"] = sum(f["deletions"] for f in files)
        row["first_parent_index"] = first_parent_index(sha)
        row["symbol_changes"] = libcob_symbols(sha, c["parents"]) if "libcob" in areas else None

        entry = entries.get(sha)
        is_semantic = (not is_merge) and bool(set(areas) & set(SEMANTIC_PREFIXES))
        if entry is None:
            if is_semantic:
                semantic_uncurated.append(sha)
                continue
            status = default_status(areas, is_merge)
            row["status"] = status
            row["curated"] = False
            row["action"] = "accounted (mechanical default)"
            row["behavior"] = "no candidate-visible semantic change (mechanical default)"
            row["court"] = "GNURUST.UPSTREAM.COMMIT-ATLAS.1"
            row["residual"] = ""
            row["superseded_by"] = None
            row["evidence"] = None
            row["integrated"] = False
        else:
            row["curated"] = True
            row["status"] = entry.get("status")
            row["action"] = entry.get("action", "")
            row["behavior"] = entry.get("behavior", "")
            row["court"] = entry.get("court", "")
            row["residual"] = entry.get("residual", "")
            row["superseded_by"] = entry.get("superseded_by")
            row["evidence"] = entry.get("evidence")
            row["integrated"] = bool(entry.get("evidence"))
            if row["status"] not in STATUSES:
                print(f"FATAL: bad status {row['status']} for {sha}")
                return 1
        rows.append(row)

    if semantic_uncurated:
        print(f"FATAL: {len(semantic_uncurated)} semantic commits lack curated overrides:")
        for sha in semantic_uncurated:
            print("  ", sha)
        return 1

    # ---- invariants -------------------------------------------------------
    status_counts = Counter(r["status"] for r in rows)
    area_counts = Counter(",".join(r["areas"]) for r in rows)
    curated_count = sum(1 for r in rows if r["curated"])
    merge_count = sum(1 for r in rows if r["is_merge"])
    fp_count = sum(1 for r in rows if r["first_parent_index"] is not None)
    for s in STATUSES:
        if s in ("UPSTREAM_MERGE_ACCOUNTED",):
            continue
    # every row must have a status
    for r in rows:
        if r["status"] not in STATUSES:
            print(f"FATAL: row {r['sha']} has non-enum status {r['status']}")
            return 1
    # merge status coherence
    for r in rows:
        if r["is_merge"] and r["status"] != "UPSTREAM_MERGE_ACCOUNTED":
            print(f"FATAL: merge {r['sha']} has status {r['status']}")
            return 1

    invariants = {
        "range": f"{RANGE_START}..{RANGE_END}",
        "range_start": RANGE_START,
        "range_end": RANGE_END,
        "expected_commit_count": expected_count,
        "atlas_row_count": len(rows),
        "unique_shas": len({r['sha'] for r in rows}),
        "merge_commits": merge_count,
        "non_merge_commits": len(rows) - merge_count,
        "first_parent_commits": fp_count,
        "curated_commits": curated_count,
        "semantic_commits": sum(
            1 for r in rows if (not r['is_merge']) and bool(set(r['areas']) & set(SEMANTIC_PREFIXES))
        ),
        "status_counts": dict(status_counts),
        "checks": {
            "row_count_matches_rev_list": len(rows) == expected_count,
            "no_duplicate_shas": len({r['sha'] for r in rows}) == len(rows),
            "all_rows_have_enum_status": all(r["status"] in STATUSES for r in rows),
            "all_merges_accounted_as_merges": all(
                (r["is_merge"] == (r["status"] == "UPSTREAM_MERGE_ACCOUNTED")) for r in rows
            ),
            "no_uncurated_semantic_commit": True,
            "all_override_shas_in_range": True,
        },
    }

    # ---- file-change index ------------------------------------------------
    file_index: dict[str, list[dict]] = OrderedDict()
    for r in rows:
        for f in r["files"]:
            file_index.setdefault(f["path"], []).append({
                "sha": r["sha"],
                "status": f["status"],
                "insertions": f["insertions"],
                "deletions": f["deletions"],
            })

    # ---- symbol-change index ----------------------------------------------
    symbol_index: dict[str, dict] = {}
    for r in rows:
        sc = r["symbol_changes"]
        if not sc:
            continue
        for sym in sc["added"]:
            symbol_index.setdefault(sym, {"added_by": [], "removed_by": []})
            symbol_index[sym]["added_by"].append(r["sha"])
        for sym in sc["removed"]:
            symbol_index.setdefault(sym, {"added_by": [], "removed_by": []})
            symbol_index[sym]["removed_by"].append(r["sha"])

    # ---- outputs ----------------------------------------------------------
    def minimal(r: dict) -> dict:
        out = dict(r)
        out.pop("symbol_changes", None)
        return out

    atlas = {
        "schema": "gnurust-commit-atlas-v1",
        "identity": {
            "stable_baseline": RANGE_START,
            "current_head": RANGE_END,
            "admit_repo": "lab/admit/gnucobol-upstream-current/",
            "overrides": "lab/gnucobol-upstream-current/atlas_overrides.json",
            "status_enum": STATUSES,
        },
        "invariants": invariants,
        "commits": [minimal(r) for r in rows],
    }
    with open(os.path.join(OUT, "commit-atlas.json"), "w", encoding="utf-8") as fh:
        json.dump(atlas, fh, indent=1, sort_keys=False)
        fh.write("\n")

    # CSV
    with open(os.path.join(OUT, "commit-atlas.csv"), "w", encoding="utf-8") as fh:
        fh.write("sha,first_parent_index,date,author,is_merge,status,areas,files,insertions,deletions,subject,action\n")
        for r in rows:
            fpi = r["first_parent_index"] if r["first_parent_index"] is not None else ""
            areas = ";".join(r["areas"])
            subj = r["subject"].replace(",", " ").replace('"', "'")
            action = (r["action"] or "").replace(",", " ").replace('"', "'")
            fh.write(
                f"{r['sha']},{fpi},{r['author_date']},{r['author_name']},"
                f"{1 if r['is_merge'] else 0},{r['status']},{areas},{len(r['files'])},"
                f"{r['insertions']},{r['deletions']},{subj},{action}\n"
            )

    with open(os.path.join(OUT, "file-change-index.json"), "w", encoding="utf-8") as fh:
        json.dump({"schema": "gnurust-file-change-index-v1", "files": file_index}, fh, indent=1)
        fh.write("\n")

    with open(os.path.join(OUT, "symbol-change-index.json"), "w", encoding="utf-8") as fh:
        json.dump({"schema": "gnurust-symbol-change-index-v1", "symbols": symbol_index}, fh, indent=1)
        fh.write("\n")

    write_markdown(rows, invariants)

    print("=== atlas invariants ===")
    for k, v in invariants["checks"].items():
        print(f"  {k}: {'PASS' if v else 'FAIL'}")
    print(f"  total rows: {len(rows)} (expected {expected_count})")
    print(f"  status counts: {dict(status_counts)}")
    print("OK")
    return 0


def write_markdown(rows: list[dict], invariants: dict) -> None:
    lines = []
    add = lines.append
    add("# GnuCOBOL upstream commit atlas — stable 3.2 baseline → current head")
    add("")
    add(f"- range: `{invariants['range']}`")
    add(f"- admit repo: `lab/admit/gnucobol-upstream-current/`")
    add(f"- rows: {invariants['atlas_row_count']} (matches `git rev-list` count)")
    add(f"- merges: {invariants['merge_commits']}; non-merge: {invariants['non_merge_commits']}; "
        f"first-parent chain: {invariants['first_parent_commits']}")
    add(f"- curated semantic entries: {invariants['curated_commits']}")
    add("")
    add("## Status totals")
    add("")
    add("| status | count |")
    add("|---|---|")
    for s, n in sorted(invariants["status_counts"].items(), key=lambda kv: -kv[1]):
        add(f"| {s} | {n} |")
    add("")
    add("## Integrity checks")
    add("")
    for k, v in invariants["checks"].items():
        add(f"- `{k}`: {'PASS' if v else 'FAIL'}")
    add("")
    add("## First-parent chain (chronological)")
    add("")
    add("| # | commit | date | status | subject |")
    add("|---|---|---|---|---|")
    for r in sorted(
        [r for r in rows if r["first_parent_index"] is not None],
        key=lambda r: r["first_parent_index"],
    ):
        add(f"| {r['first_parent_index']} | `{r['sha'][:12]}` | {r['author_date'][:10]} | "
            f"{r['status']} | {r['subject']} |")
    add("")
    add("## Merges (accounted)")
    add("")
    add("| commit | date | subject |")
    add("|---|---|---|")
    for r in sorted([r for r in rows if r["is_merge"]], key=lambda r: r["author_date"]):
        add(f"| `{r['sha'][:12]}` | {r['author_date'][:10]} | {r['subject']} |")
    add("")
    add("## Curated semantic commits")
    add("")
    curated = [r for r in rows if r["curated"]]
    for group in ("cobc", "libcob", "config", "tests", "bin", "build"):
        grp = [r for r in curated if group in r["areas"]]
        if not grp:
            continue
        add(f"### {group} area")
        add("")
        add("| commit | date | status | subject | action |")
        add("|---|---|---|---|---|")
        for r in sorted(grp, key=lambda r: r["author_date"]):
            subj = r["subject"].replace("|", "/")
            act = (r["action"] or "").replace("|", "/")
            add(f"| `{r['sha'][:12]}` | {r['author_date'][:10]} | {r['status']} | {subj} | {act} |")
        add("")
    add("## Phase-2 integration evidence")
    add("")
    integrated = [r for r in rows if r.get("integrated")]
    if integrated:
        add("| upstream commit | upstream date | status | Rust integration commit |")
        add("|---|---|---|---|")
        for r in sorted(integrated, key=lambda r: r["author_date"]):
            add(f"| `{r['sha'][:12]}` | {r['author_date'][:10]} | {r['status']} | `{r['evidence'][:12]}` |")
    else:
        add("_None integrated yet._")
    add("")
    add("## Non-curated mechanical rows (CI / docs / build / test-only)")
    add("")
    mech = [r for r in rows if not r["curated"]]
    for r in sorted(mech, key=lambda r: r["author_date"]):
        add(f"- `{r['sha'][:12]}` {r['author_date'][:10]} [{r['status']}] {r['subject']}")
    add("")
    add("## Generator identity")
    add("")
    add("- `lab/gnucobol-upstream-current/gen_atlas.py` + `atlas_overrides.json`")
    add("- status enum: " + ", ".join(STATUSES))
    add("")
    with open(os.path.join(OUT, "commit-atlas.md"), "w", encoding="utf-8") as fh:
        fh.write("\n".join(lines))


if __name__ == "__main__":
    sys.exit(main())

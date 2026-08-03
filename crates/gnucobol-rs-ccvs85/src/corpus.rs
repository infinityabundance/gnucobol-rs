//! CCVS85 corpus spine handling: decompression, custody verification, unit splitting, and
//! content-derived dependency extraction.
//!
//! The corpus is the committed `lab/corpus/ccvs85/newcob.val.Z` (Unix `compress`/LZW, read via
//! `gzip -dc` — the same decompressor the `GNURUST.CCVS85.1` custody gate uses). Every step here
//! re-derives the committed custody facts first: a corpus whose hashes drift from the committed
//! receipt is a hard stop (the gate fails closed on corpus identity mismatch).

use crate::model::{Custody, MaterializedUnit, UnitIndexEntry};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

pub fn sha256_hex(data: &[u8]) -> String {
    let mut h = Sha256::new();
    h.update(data);
    format!("{:x}", h.finalize())
}

/// Decompress a `.Z` spine via `gzip -dc` (Unix compress/LZW mode), returning the raw bytes.
pub fn decompress(input: &Path) -> Option<Vec<u8>> {
    let out = std::process::Command::new("gzip")
        .arg("-dc")
        .arg(input)
        .output()
        .ok()?;
    if !out.status.success() || out.stdout.is_empty() {
        return None;
    }
    Some(out.stdout)
}

/// Derive the custody facts + the per-unit index from the compressed spine.
pub fn derive_custody(input: &Path) -> Option<(Custody, Vec<UnitIndexEntry>)> {
    let compressed = std::fs::read(input).ok()?;
    let decompressed = decompress(input)?;
    let text = String::from_utf8_lossy(&decompressed);
    let lines: Vec<&str> = text.lines().collect();

    let mut header_by_kind: BTreeMap<String, usize> = BTreeMap::new();
    let mut units: Vec<UnitIndexEntry> = Vec::new();
    let mut program_ids = 0usize;
    let mut end_of = 0usize;
    for (i, raw) in lines.iter().enumerate() {
        let line = raw.trim_end();
        if let Some(rest) = line.strip_prefix("*HEADER,") {
            let mut parts = rest.splitn(2, ',');
            let kind = parts.next().unwrap_or("").trim().to_string();
            let name = parts.next().unwrap_or("").trim().to_string();
            *header_by_kind.entry(kind.clone()).or_insert(0) += 1;
            if let Some(prev) = units.last_mut() {
                prev.end_line = i;
            }
            units.push(UnitIndexEntry {
                index: units.len(),
                kind,
                name,
                start_line: i + 1,
                end_line: lines.len(),
            });
        } else if line.contains("PROGRAM-ID") {
            program_ids += 1;
        }
        if line.starts_with("*END-OF") {
            end_of += 1;
        }
    }

    let custody = Custody {
        compressed_sha256: sha256_hex(&compressed),
        compressed_bytes: compressed.len() as u64,
        decompressed_sha256: sha256_hex(&decompressed),
        decompressed_bytes: decompressed.len() as u64,
        decompressed_lines: lines.len() as u64,
        unit_count: units.len(),
        header_by_kind,
    };
    let _ = (program_ids, end_of);
    Some((custody, units))
}

/// Parse a `*HEADER,...` line (or its post-`*HEADER,` remainder) into
/// (kind, main-name, subprogram-name).
///
/// The corpus has two subprogram shapes:
///   `*HEADER,COBOL,<NAME>`                          -> (COBOL, Some(NAME), None)
///   `*HEADER,COBOL,<MAIN>,SUBRTN,<SUB>`             -> (COBOL, Some(MAIN), Some(SUB))
///   `*HEADER,COBOL,<MAIN>,SUBPRG,<SUB>`             -> (COBOL, Some(MAIN), Some(SUB))  (SM units)
///   `*HEADER,CLBRY,<NAME>` / `*HEADER,DATA*,<NAME>` -> (kind, Some(NAME), None)
pub fn parse_header(line: &str) -> Option<(String, String, Option<String>, Option<String>)> {
    let rest = match line.strip_prefix("*HEADER,") {
        Some(r) => r,
        None => line,
    };
    let fields: Vec<&str> = rest.split(',').map(str::trim).collect();
    let kind = fields.first()?.to_string();
    let name = fields.get(1)?.to_string();
    if fields.len() >= 4
        && (fields[2].eq_ignore_ascii_case("SUBRTN") || fields[2].eq_ignore_ascii_case("SUBPRG"))
    {
        return Some((
            kind,
            name.clone(),
            Some(name),
            fields.get(3).map(|s| s.to_string()),
        ));
    }
    Some((kind, name, None, None))
}

/// The bytes of one unit: from the line AFTER its `*HEADER` line through `end_line` (inclusive),
/// stopping at the `*END-OF` marker. The committed index semantics are: `start_line` = the 1-based
/// line of the `*HEADER` line itself; the materialized file therefore contains 1-based lines
/// `start_line+1 ..= end_line`, minus the trailing `*END-OF` marker line. All other bytes are
/// preserved verbatim (fixed-format sequence columns, trailing tag columns, and final newline
/// handling included).
pub fn unit_bytes(lines: &[&str], entry: &UnitIndexEntry) -> Vec<u8> {
    let mut out = Vec::new();
    for i in (entry.start_line + 1)..=entry.end_line {
        let line = lines[i - 1];
        let is_end_of = line.trim_end().starts_with("*END-OF");
        if is_end_of {
            break;
        }
        out.extend_from_slice(line.as_bytes());
        out.push(b'\n');
    }
    out
}

/// Extract `PROGRAM-ID. <name>.` identifiers from unit source (case-insensitive).
pub fn program_ids(src: &str) -> Vec<String> {
    let up = src.to_ascii_uppercase();
    let mut out = Vec::new();
    let mut rest = up.as_str();
    while let Some(p) = rest.find("PROGRAM-ID") {
        rest = &rest[p + "PROGRAM-ID".len()..];
        let b = rest.as_bytes();
        let mut i = 0;
        while i < b.len() && (b[i].is_ascii_whitespace() || b[i] == b'.') {
            i += 1;
        }
        let start = i;
        while i < b.len() && (b[i].is_ascii_alphanumeric() || b[i] == b'-' || b[i] == b'_') {
            i += 1;
        }
        if i > start {
            let name = rest[start..i].to_string();
            if !out.contains(&name) {
                out.push(name);
            }
        }
    }
    out
}

/// Extract `COPY <name>.` / `COPY <name1> <name2>.` references from unit source.
///
/// The scan is fixed-format-aware: only the code area (columns 8-72) is scanned; full-line
/// comments (col-7 `*`/`/`) and free-format `*>` comments are dropped; string literals are masked
/// (including CCVS85 multi-line literals whose continuation lines carry a repeated quote marker
/// after the col-7 `-`); and the column-73-80 source tags are never scanned. Word boundaries are
/// enforced (`STATUS-COPY EQUAL TO` and `COPYRIGHT` never match).
pub fn copy_references(src: &str) -> Vec<String> {
    let code = mask_code_area(src);
    let up = code.to_ascii_uppercase();
    let b = up.as_bytes();
    let mut out = Vec::new();
    let mut i = 0usize;
    while i + 4 <= b.len() {
        if b[i..i + 4].eq_ignore_ascii_case(b"COPY") {
            let prev_ok = i == 0 || !(b[i - 1].is_ascii_alphanumeric() || b[i - 1] == b'-');
            // `COPY` must be followed by whitespace (a name) — `COPYRIGHT` has a letter right after.
            let next_is_ws = i + 4 < b.len() && b[i + 4].is_ascii_whitespace();
            if prev_ok && next_is_ws {
                let mut j = i + 4;
                loop {
                    while j < b.len() && b[j].is_ascii_whitespace() {
                        j += 1;
                    }
                    if j >= b.len() {
                        break;
                    }
                    let ns = j;
                    while j < b.len()
                        && (b[j].is_ascii_alphanumeric() || b[j] == b'-' || b[j] == b'_')
                    {
                        j += 1;
                    }
                    if j > ns {
                        let name = up[ns..j].to_string();
                        // A keyword after the first name (REPLACING/SUPPRESS/OF/IN/...) is not a
                        // copybook: REPLACING/SUPPRESS introduce trailing text; OF/IN name the
                        // LIBRARY the copybook comes from (COBOL-74 form).
                        if name == "REPLACING" || name == "SUPPRESS" || name == "OF" || name == "IN"
                        {
                            break;
                        }
                        if !out.contains(&name) {
                            out.push(name);
                        }
                    } else {
                        // a non-name token (e.g. `=` in a REPLACING pseudo-text): not a copybook
                        break;
                    }
                    if j < b.len() && b[j] == b'.' {
                        j += 1;
                        break;
                    }
                }
                i = j;
                continue;
            }
        }
        i += 1;
    }
    out
}

/// Produce the code-area bytes (columns 8-72 of every line) with string-literal content replaced
/// by spaces. Fixed-format aware:
///   - cols 1-7 (sequence + indicator) are masked;
///   - col-7 `*` / `/` lines are full comments (masked);
///   - col-7 `-` marks a literal continuation: the repeated quote marker at the start of the
///     continuation text is skipped, the literal stays open;
///   - quotes pair within and across lines; doubled quotes are literal quotes;
///   - columns 73+ (the CCVS85 source tag) are masked.
pub fn mask_code_area(src: &str) -> String {
    let mut out = String::new();
    let mut in_string: Option<u8> = None;
    for line in src.lines() {
        let chars: Vec<char> = line.chars().collect();
        let indicator = chars.get(6).copied().unwrap_or(' ');
        let is_comment = indicator == '*' || indicator == '/';
        // strip free-format `*>` inline comments within the code area
        let mut line = line.to_string();
        if let Some(p) = line.find("*>") {
            line.truncate(p);
        }
        let chars: Vec<char> = line.chars().collect();
        if is_comment {
            out.push('\n');
            continue;
        }
        // code area = 0-indexed [7, 72)
        let mut seg: Vec<char> = Vec::new();
        for (idx, c) in chars.iter().enumerate() {
            if idx < 7 || idx >= 72 {
                seg.push(' ');
            } else {
                seg.push(*c);
            }
        }
        if indicator == '-' {
            // literal continuation line: strip ONE leading quote marker when a literal is open
            // (the marker is the first non-blank char of the code area, not necessarily col 8)
            if let Some(q) = in_string {
                let first_nonspace = seg.iter().position(|c| !c.is_ascii_whitespace());
                if let Some(pos) = first_nonspace {
                    if seg[pos] == q as char {
                        seg[pos] = ' ';
                    }
                }
            }
        }
        let mut i = 0usize;
        while i < seg.len() {
            let c = seg[i];
            if let Some(q) = in_string {
                if c == q as char {
                    // doubled quote = literal quote
                    if i + 1 < seg.len() && seg[i + 1] == q as char {
                        seg[i] = ' ';
                        seg[i + 1] = ' ';
                        i += 2;
                        continue;
                    }
                    seg[i] = ' ';
                    in_string = None;
                } else {
                    seg[i] = ' ';
                }
                i += 1;
                continue;
            }
            if c == '"' || c == '\'' {
                in_string = Some(c as u8);
                seg[i] = ' ';
                i += 1;
                continue;
            }
            i += 1;
        }
        out.extend(seg.iter());
        out.push('\n');
    }
    out
}

/// Split + materialize every indexed unit, deriving content-based dependencies.
///
/// `lines` is the decompressed corpus split by line. `materialized` output dir receives:
///   COBOL -> `<NAME>.cob`        (uppercase, filesystem-safe)
///   CLBRY -> `copybooks/<NAME>.cpy`
///   DATA* -> `data/<NAME>.dat`
pub fn materialize(
    lines: &[&str],
    units: &[UnitIndexEntry],
    out_root: &Path,
) -> Vec<MaterializedUnit> {
    let clbry: BTreeSet<String> = units
        .iter()
        .filter(|u| u.kind == "CLBRY")
        .map(|u| {
            parse_header(&format!("*HEADER,{},{}", u.kind, u.name))
                .map(|(_, n, _, _)| n)
                .unwrap_or_else(|| u.name.clone())
        })
        .collect();
    let data: BTreeSet<String> = units
        .iter()
        .filter(|u| u.kind == "DATA*")
        .map(|u| {
            parse_header(&format!("*HEADER,{},{}", u.kind, u.name))
                .map(|(_, n, _, _)| n)
                .unwrap_or_else(|| u.name.clone())
        })
        .collect();

    let copy_dir = out_root.join("copybooks");
    let data_dir = out_root.join("data");
    std::fs::create_dir_all(&copy_dir).ok();
    std::fs::create_dir_all(&data_dir).ok();

    let mut out = Vec::new();
    for entry in units {
        let raw = unit_bytes(lines, entry);
        let (kind, hdr_name, main, sub) =
            parse_header(&format!("*HEADER,{},{}", entry.kind, entry.name))
                .unwrap_or_else(|| (entry.kind.clone(), entry.name.clone(), None, None));
        // SUBRTN units are subprograms: the FILE is named after the subprogram (so cobrun's
        // separate-CALL resolution finds `<sub>.cob` beside the main), and the manifest name is
        // the subprogram name.
        // The CCVS85 header name field may carry trailing whitespace-padding plus a second token
        // (e.g. `ST140A<48 spaces>TES00010` — a source-tape reference); the STABLE name is the
        // first whitespace-delimited token, so file names stay filesystem-safe.
        let first_token = |s: &str| s.split_whitespace().next().unwrap_or(s).to_string();
        let unit_name = if kind == "COBOL" && sub.is_some() {
            sub.clone().unwrap_or_default()
        } else {
            first_token(&hdr_name)
        };
        let src = String::from_utf8_lossy(&raw).to_string();
        let copys = copy_references(&src);
        let mut copy_deps = Vec::new();
        let mut missing = Vec::new();
        for c in copys {
            if clbry.contains(&c) {
                if !copy_deps.contains(&c) {
                    copy_deps.push(c);
                }
            } else if !missing.contains(&c) {
                missing.push(c);
            }
        }
        // A DATA* unit is consumed (fed on stdin) by the COBOL unit of the same name — the CCVS85
        // validation convention (the SSVG feeds each module's data cards to that module).
        let data_deps: Vec<String> = data.iter().filter(|d| **d == unit_name).cloned().collect();

        let (rel, _dir) = match kind.as_str() {
            "CLBRY" => (format!("copybooks/{unit_name}.cpy"), copy_dir.clone()),
            "DATA*" => (format!("data/{unit_name}.dat"), data_dir.clone()),
            _ => (format!("{unit_name}.cob"), out_root.to_path_buf()),
        };
        let path = out_root.join(&rel);
        std::fs::create_dir_all(path.parent().unwrap()).ok();
        std::fs::write(&path, &raw).ok();

        // Site-adapted execution copy (documented substitution table; original bytes preserved).
        // COBOL units and CLBRY copybooks are adapted (they are what the oracle/candidate compile
        // or cobc COPY-expands); DATA* units keep their original bytes as their only form.
        let (adapted_rel, adapted_sha) = if kind == "COBOL" || kind == "CLBRY" {
            let adapted = site_adapt(&raw);
            let adapted_rel = if kind == "CLBRY" {
                let adapt_dir = out_root.join("copybooks-adapted");
                std::fs::create_dir_all(&adapt_dir).ok();
                let r = format!("copybooks-adapted/{unit_name}.cpy");
                std::fs::write(adapt_dir.join(format!("{unit_name}.cpy")), &adapted).ok();
                r
            } else {
                let adapted_rel = format!("adapted/{unit_name}.cob");
                let adapted_path = out_root.join(&adapted_rel);
                std::fs::create_dir_all(adapted_path.parent().unwrap()).ok();
                std::fs::write(&adapted_path, &adapted).ok();
                adapted_rel
            };
            (adapted_rel, sha256_hex(&adapted))
        } else {
            (String::new(), String::new())
        };

        let is_exec = kind == "COBOL" && sub.is_none();
        out.push(MaterializedUnit {
            unit_index: entry.index,
            kind: kind.clone(),
            name: unit_name.clone(),
            header_raw: entry.name.clone(),
            main_program: main.clone(),
            subprogram: sub.clone(),
            source_path: rel.clone(),
            source_sha256: sha256_hex(&raw),
            adapted_path: adapted_rel.clone(),
            adapted_sha256: adapted_sha,
            start_line: entry.start_line,
            end_line: entry.end_line,
            program_ids: program_ids(&src),
            copy_dependencies: copy_deps,
            missing_copybooks: missing,
            data_dependencies: data_deps,
            is_executable_candidate: is_exec,
        });
    }
    out
}

/// The CCVS85 site-adaptation (documented; part of the corpus's own NIST SSVG methodology).
/// The corpus ships with `XXXXX0NN` site-parameter tokens AND column-7 site-adaptation markers
/// that the validation site resolves. This court's deterministic site applies:
///   `XXXXX084` -> `OMITTED`  (the FD `LABEL RECORDS` placeholder; GnuCOBOL requires
///                             `LABEL RECORDS [IS] OMITTED|STANDARD`)
///   `XXXXX030..043 / 063 / 064 / 081` -> `"X0NNXX"`-style literals (the CCVS85 VALUE-operand
///                             placeholders). The NIST site fills these with site values; this
///                             court uses deterministic, distinct values so the tests remain
///                             self-consistent and the oracle/candidate compare the SAME source.
///   `XXXXX053` -> the LINE IS DROPPED (the I-O-CONTROL `RERUN`-clause site card). RERUN is
///                             not implemented by GnuCOBOL and the bare placeholder token derails
///                             cobc's ENVIRONMENT-DIVISION parser ("missing file description" /
///                             "PROCEDURE DIVISION header missing" cascades), so this site drops
///                             the placeholder line exactly as it drops the Y/D/C/G-marked lines.
///   column-7 `Y` -> the line is DELETED (NIST deleted-test marker), `D` -> debug line dropped
///   column-7 `C`/`G` -> DELETED (the obsolete COBOL-74 FD clauses — LABEL RECORDS / VALUE OF /
///                             DATA RECORD — that GnuCOBOL does not accept; every C/G-marked line
///                             is part of those FD clause blocks)
///   column-7 other letters (P/S/X/A/J/...) -> the line is KEPT as code (the letter is not
///                             a GnuCOBOL indicator; this site keeps option-marked lines). The
///                             `P`-marked lines are the module's own CCVS1 prologue (the corpus's
///                             RAW-DATA harness routines); keeping them is this site's choice of
///                             prologue — the modules' test code PERFORMs CCVS1 paragraphs
///                             (WRITE-LINE / CCVS1-EXIT / BAIL-OUT ...), so they must be present.
///   `RECORD-KEY` -> `RECORD KEY` (GnuCOBOL spells the INDEXED key clause as two words; the
///                             hyphenated form is otherwise parsed as an identifier)
///   CCVS1-counter unification (definition-aware; the corpus ships two prologue generations):
///     - when a unit defines `NO-OF-TESTS` and not `C-NO-OF-TESTS`, every `C-NO-OF-TESTS`
///       reference (the other generation's name) is mapped to the unit's `NO-OF-TESTS`;
///     - when a unit defines `DELETE-COUNTER` and not `DELETE-CNT`, every `DELETE-COUNTER`
///       (that generation's own field) is mapped to `DELETE-CNT` (the prologue contract name).
///       Renaming the DEFINED field (not the reference) keeps the replacement shorter, so
///       width-preservation holds. Without this, the 25 SQ modules' own prologue text would
///       not compile (undefined data name), which is a corpus-internal template drift, not a
///       language-feature rejection.
/// Every substitution is WIDTH-PRESERVING (the replacement is space-padded to the token's width),
/// so the fixed-format column layout — including the column-73-80 source tags — is never shifted.
/// Substitutions apply ONLY in the code area (cols 8-72) of non-comment lines and are
/// word-boundary aware; string literals are left byte-identical (the many `XXXXX0NN` tokens
/// embedded in literals, e.g. indexed-file key images, are untouched). The ORIGINAL materialized
/// bytes are always preserved (the manifest records both hashes).
pub fn site_adaptation_table() -> Vec<(String, String)> {
    let mut v = vec![
        ("XXXXX084".to_string(), "OMITTED".to_string()),
        ("RECORD-KEY".to_string(), "RECORD KEY".to_string()),
        // Site device/value/PICTURE cards:
        //   X-51  = SWITCH-1 (SPECIAL-NAMES switch device; GnuCOBOL spells it `SWITCH-1`)
        //   X-56  = CONSOLE  (DISPLAY device mnemonic; GnuCOBOL accepts `CONSOLE`)
        //   X-57  = CONSOLE  (ACCEPT device mnemonic)
        //   X-73  = SYSIN    (I/O device mnemonic for ADVANCING; GnuCOBOL accepts `SYSIN`)
        //   X-65  = 100      (site file-record-count value placeholder, e.g. `MOVE X-65 TO
        //                     RECORDS-IN-FILE (1)`; the site's data-file record count)
        //   X-68  = 100      (the obsolete MEMORY SIZE value; cobc 3.2 rejects the MEMORY SIZE
        //                     clause itself, so this only sharpens the failure reason)
        //   X-86  = PIC 9(8) (the PICTURE-clause placeholder; SQ401M's VKEY is a RELATIVE key)
        ("XXXXX051".to_string(), "SWITCH-1".to_string()),
        ("XXXXX052".to_string(), "SWITCH-2".to_string()),
        ("XXXXX056".to_string(), "CONSOLE".to_string()),
        ("XXXXX057".to_string(), "CONSOLE".to_string()),
        ("XXXXX073".to_string(), "SYSIN".to_string()),
        ("XXXXX065".to_string(), "100".to_string()),
        ("XXXXX067".to_string(), "100".to_string()),
        ("XXXXX068".to_string(), "100".to_string()),
        ("XXXXX086".to_string(), "PIC 9(8)".to_string()),
        // X-90 / X-91: the CLASS character-set endpoints (NC174A/NC211A/NC254A use them as the
        // `A` and `D` bounds of the ordinal CLASS definitions).
        ("XXXXX090".to_string(), "\"A\"".to_string()),
        ("XXXXX091".to_string(), "\"D\"".to_string()),
    ];
    for n in [30, 31, 32, 33, 34, 35, 38, 39, 40, 41, 42, 43, 63, 64, 81] {
        v.push((format!("XXXXX{n:0>3}"), format!("\"X{n:0>3}XX\"")));
    }
    v
}

/// True when the line's code area (cols 8-72) is exactly a bare site file-name card (optionally
/// with a trailing period), with no other code on the line. Covers the three card families the
/// corpus uses: `XXXXX0NN`, `XXXXP0NN` and `XXXXD0NN` (the `P`/`D` variants are the optional-X-card
/// generations; the letter is in column 12).
fn is_bare_p_card(chars: &[char]) -> bool {
    let code: String = chars
        .iter()
        .enumerate()
        .filter(|(i, _)| (7..72).contains(i))
        .map(|(_, c)| *c)
        .collect();
    let t = code.trim();
    let t = t.strip_suffix('.').unwrap_or(t).trim_end();
    let b = t.as_bytes();
    b.len() == 8
        && b[..4] == *b"XXXX"
        && (b[4] == b'X' || b[4] == b'P' || b[4] == b'D')
        && b[5] == b'0'
        && b[6].is_ascii_digit()
        && b[7].is_ascii_digit()
}

/// True when the unit source defines the given record/WS field (e.g. `05 C-NO-OF-TESTS` or
/// `01 DELETE-COUNTER`) in the code area. Used by the definition-aware CCVS1 unification.
/// Matches a line whose code area (cols 8-72) starts with `<level> <field>` where `<level>` is
/// 01/05/07 etc. and `<field>` is followed by a non-name character (word boundary).
fn unit_defines_field(src: &str, field: &str) -> bool {
    src.lines().any(|line| {
        let chars: Vec<char> = line.chars().collect();
        if chars.len() < 7 {
            return false;
        }
        // code area = 0-indexed [7, 72); the fixed-format level number + field live there
        let code: String = chars
            .iter()
            .enumerate()
            .filter(|(i, _)| (7..72).contains(i))
            .map(|(_, c)| *c)
            .collect();
        let trimmed = code.trim_start();
        let name_start = trimmed
            .find(|c: char| !c.is_ascii_digit() && !c.is_ascii_whitespace())
            .unwrap_or(trimmed.len());
        let level: String = trimmed[..name_start]
            .chars()
            .filter(|c| c.is_ascii_digit())
            .collect();
        if level.is_empty() || level.len() > 2 {
            return false;
        }
        let after = trimmed[name_start..].trim_start();
        after
            .strip_prefix(field)
            .map(|r| {
                r.chars()
                    .next()
                    .map(|c| !(c.is_ascii_alphanumeric() || c == '-'))
                    .unwrap_or(true)
            })
            .unwrap_or(false)
    })
}

/// Apply [`site_adaptation_table`] + the column-7 site-adaptation markers to a unit's raw bytes
/// (fixed-format aware). Returns the adapted bytes.
pub fn site_adapt(raw: &[u8]) -> Vec<u8> {
    let text = String::from_utf8_lossy(raw);
    // Definition-aware CCVS1 unification flags (see [`site_adaptation_table`] docs).
    let src_for_defs = text.to_string();
    let defs_no_of_tests = unit_defines_field(&src_for_defs, "NO-OF-TESTS");
    let defs_c_no_of_tests = unit_defines_field(&src_for_defs, "C-NO-OF-TESTS");
    let defs_delete_counter = unit_defines_field(&src_for_defs, "DELETE-COUNTER");
    let defs_delete_cnt = unit_defines_field(&src_for_defs, "DELETE-CNT");
    let unify_no_of_tests = defs_no_of_tests && !defs_c_no_of_tests;
    let unify_delete_cnt = defs_delete_counter && !defs_delete_cnt;
    // Pre-scan for the optional-X-card sibling rule: when a line whose code area is a bare
    // `XXXXX0NN` / `XXXXP0NN` / `XXXXD0NN` file-name card is immediately adjacent (above or below)
    // to a site-option-marked (letter col-7) bare file-name card, the UNMARKED card is the
    // non-selected variant and is dropped (the marked one is the site-selected name). This
    // resolves the IX modules' `ASSIGN TO XXXXX024 / J XXXXX044` pairs (both orders occur in the
    // corpus) — keeping both would give cobc two file names in one ASSIGN.
    let all_lines: Vec<&str> = text.lines().collect();
    let mut drop_pairs: Vec<bool> = vec![false; all_lines.len()];
    for (i, line) in all_lines.iter().enumerate() {
        let chars: Vec<char> = line.chars().collect();
        let indicator = chars.get(6).copied().unwrap_or(' ');
        if !is_bare_p_card(&chars) || indicator.is_ascii_alphabetic() {
            continue;
        }
        let marked_neighbor = [i.checked_sub(1), i.checked_add(1)]
            .into_iter()
            .flatten()
            .any(|k| {
                all_lines
                    .get(k)
                    .map(|nl| {
                        let nc: Vec<char> = nl.chars().collect();
                        let nind = nc.get(6).copied().unwrap_or(' ');
                        nind.is_ascii_alphabetic() && nind != '-' && is_bare_p_card(&nc)
                    })
                    .unwrap_or(false)
            });
        if marked_neighbor {
            drop_pairs[i] = true;
        }
    }
    let mut out = Vec::new();
    for (li, line) in all_lines.iter().enumerate() {
        if drop_pairs[li] {
            continue;
        }
        let chars: Vec<char> = line.chars().collect();
        let indicator = chars.get(6).copied().unwrap_or(' ');
        // column-7 site-adaptation markers (not GnuCOBOL indicators):
        //   Y = NIST deleted-test line, D = debug line, C/G = obsolete FD clauses
        //   (LABEL RECORDS / VALUE OF / DATA RECORD) -> dropped by this site
        if indicator == 'Y' || indicator == 'D' || indicator == 'C' || indicator == 'G' {
            continue;
        }
        let is_code = chars.len() >= 7 && indicator != '*' && indicator != '/';
        if !is_code {
            out.extend_from_slice(line.as_bytes());
            out.push(b'\n');
            continue;
        }
        // The I-O-CONTROL `RERUN` site card (`XXXXX053`) is dropped by this site (GnuCOBOL has
        // no RERUN implementation; the bare token derails cobc's parser). Match the code area.
        {
            let code: String = chars
                .iter()
                .enumerate()
                .filter(|(i, _)| (7..72).contains(i))
                .map(|(_, c)| *c)
                .collect();
            let trimmed = code.trim();
            if trimmed == "XXXXX053" || trimmed == "XXXXX053." {
                continue;
            }
        }
        let mut chars = chars;
        // A kept site-option-marked line: the letter is NOT a GnuCOBOL indicator, so blank it
        // (the line stays as code). `-` continuation lines keep their indicator.
        if indicator.is_ascii_alphabetic() && indicator != '-' {
            chars[6] = ' ';
        }
        let mut i = 0usize;
        let mut line_out = String::new();
        let mut in_string: Option<char> = None;
        while i < chars.len() {
            let in_code_area = i >= 7 && i < 72;
            if in_code_area {
                // string-literal guard: never substitute inside a literal
                if let Some(q) = in_string {
                    if chars[i] == q {
                        if i + 1 < chars.len() && chars[i + 1] == q {
                            line_out.push(chars[i]);
                            line_out.push(chars[i + 1]);
                            i += 2;
                            continue;
                        }
                        in_string = None;
                    }
                    line_out.push(chars[i]);
                    i += 1;
                    continue;
                }
                if chars[i] == '\"' || chars[i] == '\'' {
                    in_string = Some(chars[i]);
                    line_out.push(chars[i]);
                    i += 1;
                    continue;
                }
                // try to match a substitution token at this position (word-boundary aware:
                // `XRECORD-KEY` never matches `RECORD-KEY`)
                let rest: String = chars[i..].iter().collect();
                let prev_ok = i == 0
                    || !(chars[i - 1].is_ascii_alphanumeric()
                        || chars[i - 1] == '-'
                        || chars[i - 1] == '_');
                let mut matched = false;
                if prev_ok {
                    for (tok, rep) in site_adaptation_table() {
                        if rest.starts_with(&tok) {
                            // right-boundary guard: the token must not be a prefix of a longer
                            // identifier — `RECORD-KEY-CONTENT` (a data name) must never become
                            // `RECORD KEY-CONTENT`; only the standalone clause token `RECORD-KEY`
                            // is split into `RECORD KEY`.
                            let after_ok = rest[tok.chars().count()..]
                                .chars()
                                .next()
                                .map(|c| !(c.is_ascii_alphanumeric() || c == '-' || c == '_'))
                                .unwrap_or(true);
                            if !after_ok {
                                continue;
                            }
                            // WIDTH-PRESERVING substitution: pad the replacement with spaces to the
                            // token's width so every later column (incl. the cols-73-80 source tag)
                            // stays in place — fixed-format column integrity is part of the corpus.
                            line_out.push_str(&rep);
                            let tok_w = tok.chars().count();
                            let rep_w = rep.chars().count();
                            for _ in rep_w..tok_w {
                                line_out.push(' ');
                            }
                            i += tok_w;
                            matched = true;
                            break;
                        }
                    }
                }
                if matched {
                    continue;
                }
                // Definition-aware CCVS1 unification (see [`site_adaptation_table`] docs): when
                // the unit's own prologue generation defines the OTHER name, rename to the unit's
                // name so the corpus's own prologue text compiles. Width-preserving (the
                // replacement is always shorter than the token).
                if (unify_no_of_tests || unify_delete_cnt) && i + 1 < chars.len() {
                    let prev_ok = i == 0
                        || !(chars[i - 1].is_ascii_alphanumeric()
                            || chars[i - 1] == '-'
                            || chars[i - 1] == '_');
                    if prev_ok {
                        let rest2: String = chars[i..].iter().collect();
                        for (tok, rep) in [
                            ("C-NO-OF-TESTS", "NO-OF-TESTS"),
                            ("DELETE-COUNTER", "DELETE-CNT"),
                        ] {
                            let use_tok = if tok == "C-NO-OF-TESTS" {
                                unify_no_of_tests
                            } else {
                                unify_delete_cnt
                            };
                            if use_tok && rest2.starts_with(tok) {
                                line_out.push_str(rep);
                                let tok_w = tok.chars().count();
                                let rep_w = rep.chars().count();
                                for _ in rep_w..tok_w {
                                    line_out.push(' ');
                                }
                                i += tok_w;
                                matched = true;
                                break;
                            }
                        }
                    }
                    if matched {
                        continue;
                    }
                }
            }
            line_out.push(chars[i]);
            i += 1;
        }
        out.extend_from_slice(line_out.as_bytes());
        out.push(b'\n');
    }
    out
}

/// The unit line-range index: 1-based lines of the decompressed file, mirroring the committed
/// `corpus-index.json` (which is the `GNURUST.CCVS85.1` split).
pub fn write_index_json(path: &Path, units: &[MaterializedUnit]) {
    let v: Vec<serde_json::Value> = units
        .iter()
        .map(|u| {
            serde_json::json!({
                "unit_index": u.unit_index,
                "kind": u.kind,
                "name": u.name,
                "header_raw": u.header_raw,
                "main_program": u.main_program,
                "subprogram": u.subprogram,
                "source_path": u.source_path,
                "source_sha256": u.source_sha256,
                "adapted_path": u.adapted_path,
                "adapted_sha256": u.adapted_sha256,
                "start_line": u.start_line,
                "end_line": u.end_line,
                "program_ids": u.program_ids,
                "copy_dependencies": u.copy_dependencies,
                "missing_copybooks": u.missing_copybooks,
                "data_dependencies": u.data_dependencies,
                "is_executable_candidate": u.is_executable_candidate,
            })
        })
        .collect();
    let _ = std::fs::write(path, serde_json::to_string_pretty(&v).unwrap() + "\n");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scanner_ignores_string_literals_and_tags() {
        // fixed-format lines: cols 1-6 sequence, col 7 indicator, cols 8-72 code, 73+ tag
        let src = "035600            \" COPY - NOT FOR DISTRIBUTION\".                       NC1074.2\n\
                   035700      COPY K101A.                                            NC1074.2\n\
                   037700            \"  COPYRIGHT   1985 \".                                NC1074.2\n\
                   035800      COPY K1WKA REPLACING ==A== BY ==B==.                  NC1074.2\n";
        let refs = copy_references(src);
        assert_eq!(
            refs,
            vec!["K101A".to_string(), "K1WKA".to_string()],
            "got {refs:?}"
        );
    }

    #[test]
    fn scanner_handles_ccvs85_full_lines() {
        // real NC107A-style lines (sequence + code + tag columns): literals only, no COPY stmts
        let src = "035600            \" COPY - NOT FOR DISTRIBUTION\".                       NC1074.2\n\
                   037700            \"  COPYRIGHT   1985 \".                                NC1074.2\n\
                   039900             \" COPYRIGHT 1985\".                                   NC1074.2\n";
        let refs = copy_references(src);
        assert!(refs.is_empty(), "got {refs:?}");
    }

    #[test]
    fn scanner_handles_multi_line_literals() {
        // CCVS85 continuation style: col-7 `-` lines repeat the quote marker
        let src = "005100     02  TABLE-A-VALUES   PICTURE X(20) VALUE \"1112223334441122334NC1324.2\n\
                   005200-    \"4\".                                                         NC1324.2\n\
                   010500     02 FILLER  PIC IS X(99)    VALUE IS \" FEATURE              PANC1324.2\n\
                   010600-    \"SS  PARAGRAPH-NAME                                          NC1324.2\n\
                   010700-    \"       REMARKS\".                                            NC1324.2\n\
                   013700            \" COPY - NOT FOR DISTRIBUTION\".                       NC1324.2\n";
        let refs = copy_references(src);
        assert!(refs.is_empty(), "got {refs:?}");
    }

    #[test]
    fn scanner_reads_name_on_next_line() {
        // the COPY keyword on one line, the name on the next (both in the code area)
        let src = "000100      COPY                                                                 ALTL14.2\n\
                   000200      ALTLB.                                                          ALTL14.2\n";
        let refs = copy_references(src);
        assert_eq!(refs, vec!["ALTLB".to_string()], "got {refs:?}");
    }

    #[test]
    fn scanner_two_books_in_one_copy() {
        // `COPY A B.` copies both A and B (KP001 SM2064 style, tag beyond col 72)
        let src =
            "033800      COPY                                                   KP001  SM2064.2\n";
        let refs = copy_references(src);
        // SM2064 is the column-73-80 tag (beyond the scanned code area) — must NOT appear
        assert_eq!(refs, vec!["KP001".to_string()], "got {refs:?}");
    }

    #[test]
    fn site_adapt_substitutes_only_code_position_tokens() {
        // code-area tokens are substituted; literal-embedded tokens are untouched
        let src = "003600     XXXXX084                                                     CM1014.2\n\
                   003700 77  PASSWORD1 PIC X(10) VALUE XXXXX031.                        CM1014.2\n\
                   006000     MOVE \"SSSSSTTTTT166WWWWWXXXXX060ALTKEY1FFFFFEEE\" TO X.    SQ1014.2\n";
        let adapted = String::from_utf8(site_adapt(src.as_bytes())).unwrap();
        assert!(adapted.contains("     OMITTED "), "got: {adapted}");
        assert!(adapted.contains("VALUE \"X031XX\""), "got: {adapted}");
        assert!(
            adapted.contains("XXXXX060"),
            "literal token must be untouched: {adapted}"
        );
        // width-preserving: the col-73-80 tag must remain at the same column
        for (orig_line, adapt_line) in src.lines().zip(adapted.lines()) {
            if orig_line.len() >= 73 {
                let orig_tag = &orig_line[72..];
                let adapt_tag = adapt_line.chars().skip(72).collect::<String>();
                assert_eq!(
                    adapt_tag.trim_end(),
                    orig_tag.trim_end(),
                    "tag shifted: {orig_tag:?} vs {adapt_tag:?}"
                );
            }
        }
    }

    #[test]
    fn site_adapt_word_boundary_and_literals() {
        // `XRECORD-KEY` must NOT be substituted; literal-embedded tokens must NOT be substituted;
        // the C/G-marked FD lines are dropped; Y-lines are dropped; S-marked lines are kept.
        let src = "012600           07 XRECORD-KEY          PIC X(29).                     RL1014.2\n\
                   012800      RECORD-KEY IS RAW-KEY.                                      RL1014.2\n\
                   006000     MOVE \"SSSSTTTTTT165WWWWXXXXXX063ALTKEY1FFFFEEEE\" TO X.    RL1014.2\n\
                   005400C    VALUE OF                                                     RL1014.2\n\
                   033500Y    IF RECORD-COUNT GREATER 50                                   DB1014.2\n\
                   027200S    EXIT PROGRAM.                                                DB1014.2\n";
        let adapted = String::from_utf8(site_adapt(src.as_bytes())).unwrap();
        assert!(adapted.contains("XRECORD-KEY"), "word-boundary: {adapted}");
        assert!(
            adapted.contains("RECORD KEY IS RAW-KEY"),
            "clause substituted: {adapted}"
        );
        assert!(
            adapted.contains("XXXXX063"),
            "literal token untouched: {adapted}"
        );
        assert!(!adapted.contains("VALUE OF"), "C-line dropped: {adapted}");
        assert!(
            !adapted.contains("RECORD-COUNT GREATER"),
            "Y-line dropped: {adapted}"
        );
        assert!(adapted.contains("EXIT PROGRAM"), "S-line kept: {adapted}");
    }

    #[test]
    fn unit_bytes_excludes_header_line() {
        let lines: Vec<&str> = vec![
            "CCVS85  VERSION 4.0   01 OCT 1992 0032",
            "*HEADER,CLBRY,ALTLB",
            "000100*    THIS TEXT MUST BE PLACED",
            "000200     MOVE SPACES TO RE-MARK.",
            "*END-OF,ALTLB",
            "*HEADER,COBOL,CM101M",
        ];
        let entry = UnitIndexEntry {
            index: 1,
            kind: "CLBRY".into(),
            name: "ALTLB".into(),
            start_line: 2,
            end_line: 4,
        };
        let bytes = unit_bytes(&lines, &entry);
        let text = String::from_utf8(bytes).unwrap();
        assert!(
            !text.contains("*HEADER"),
            "header must be excluded: {text:?}"
        );
        assert!(
            !text.contains("*END-OF"),
            "end-of must be excluded: {text:?}"
        );
        assert!(
            text.contains("000100*    THIS TEXT"),
            "content must start after the header: {text:?}"
        );
        assert!(
            text.contains("000200     MOVE SPACES"),
            "content must include the last line: {text:?}"
        );
    }
}

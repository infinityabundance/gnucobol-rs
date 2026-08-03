//! GNURUST.CCVS85.2 corpus-materialization tests — synthetic miniature CCVS-style fixtures.
//!
//! These cover: unit splitting, fixed-format byte preservation, sequence-column preservation,
//! dependency extraction, library-unit handling, data-unit handling, stable path generation,
//! stable hashing, and the site-adaptation rules (RERUN card, CCVS1 counter unification,
//! optional-X-card siblings, device-name cards, `RECORD-KEY` boundary). No full benchmark run and
//! no corpus spine is required — everything is self-contained synthetic fixtures.

use gnucobol_rs_ccvs85::corpus::{materialize, sha256_hex, site_adapt, unit_bytes};
use gnucobol_rs_ccvs85::model::UnitIndexEntry;
use std::path::PathBuf;

/// Build a fixed-format line: code in cols 8-72, tag at cols 73-80 (exactly 80 chars).
fn fl(seq: &str, marker: &str, code: &str, tag: &str) -> String {
    let mut l = String::new();
    l.push_str(seq);
    l.push_str(marker);
    l.push_str(code);
    while l.len() < 72 {
        l.push(' ');
    }
    l.push_str(tag);
    assert_eq!(l.len(), 80, "fixture line not 80 chars: {l:?}");
    l
}

/// A tiny synthetic corpus spine with every header shape the court must handle.
fn synthetic_corpus() -> String {
    [
        "CCVS85  VERSION 4.0   01 OCT 1992 0032".to_string(),
        "*HEADER,CLBRY,K1FDA".to_string(),
        fl("000100", " ", "LABEL RECORDS STANDARD", "K1FDA4.2"),
        fl("000200", "C", "VALUE OF", "K1FDA4.2"),
        fl("000300", "C", "XXXXX074", "K1FDA4.2"),
        fl("000700", " ", "DATA RECORD IS TST-TEST.", "K1FDA4.2"),
        "*END-OF,K1FDA".to_string(),
        "*HEADER,DATA*,NC109M".to_string(),
        "19920101".to_string(),
        "*END-OF,NC109M".to_string(),
        "*HEADER,COBOL,NC109M".to_string(),
        fl("000100", " ", "IDENTIFICATION DIVISION.", "NC1094.2"),
        fl("000200", " ", "PROGRAM-ID. NC109M.", "NC1094.2"),
        fl("000300", " ", "ENVIRONMENT DIVISION.", "NC1094.2"),
        fl("000400", " ", "DATA DIVISION.", "NC1094.2"),
        fl("000500", " ", "PROCEDURE DIVISION.", "NC1094.2"),
        fl("000600", " ", "STOP RUN.", "NC1094.2"),
        "*END-OF,NC109M".to_string(),
        "*HEADER,COBOL,IX101A,SUBRTN,IX102A".to_string(),
        fl("000100", " ", "IDENTIFICATION DIVISION.", "IX1024.2"),
        fl("000200", " ", "PROGRAM-ID. IX102A.", "IX1024.2"),
        fl("000300", " ", "PROCEDURE DIVISION.", "IX1024.2"),
        fl("000400", " ", "EXIT PROGRAM.", "IX1024.2"),
        "*END-OF,IX102A".to_string(),
        "*HEADER,COBOL,ST140A                                                    TES00010"
            .to_string(),
        fl("000100", " ", "IDENTIFICATION DIVISION.", "ST1404.2"),
        fl("000200", " ", "PROGRAM-ID. ST140A.", "ST1404.2"),
        fl("000300", " ", "PROCEDURE DIVISION.", "ST1404.2"),
        fl("000400", " ", "STOP RUN.", "ST1404.2"),
        "*END-OF,ST140A".to_string(),
    ]
    .join("\n")
}

fn index_from_corpus(text: &str) -> Vec<UnitIndexEntry> {
    let mut units: Vec<UnitIndexEntry> = Vec::new();
    for (i, raw) in text.lines().enumerate() {
        let line = raw.trim_end();
        if let Some(rest) = line.strip_prefix("*HEADER,") {
            let mut parts = rest.splitn(2, ',');
            let kind = parts.next().unwrap_or("").trim().to_string();
            let name = parts.next().unwrap_or("").trim().to_string();
            if let Some(prev) = units.last_mut() {
                prev.end_line = i;
            }
            units.push(UnitIndexEntry {
                index: units.len(),
                kind,
                name,
                start_line: i + 1,
                end_line: text.lines().count(),
            });
        }
    }
    units
}

fn write_fixture() -> (tempfile::TempDir, PathBuf) {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().to_path_buf();
    (dir, path)
}

// The crate's tests use `tempfile` only in tests/ — declared as a dev-dependency below via the
// tests' own Cargo usage; see Cargo.toml [dev-dependencies].

#[test]
fn splitting_excludes_header_and_end_of() {
    let text = synthetic_corpus();
    let lines: Vec<&str> = text.lines().collect();
    let units = index_from_corpus(&text);
    // NC109M is the third unit (index 2)
    let nc = units
        .iter()
        .find(|u| u.kind == "COBOL" && u.name == "NC109M")
        .unwrap();
    let bytes = unit_bytes(&lines, nc);
    let s = String::from_utf8(bytes).unwrap();
    assert!(!s.contains("*HEADER"), "header leaked: {s:?}");
    assert!(!s.contains("*END-OF"), "end-of leaked: {s:?}");
    assert!(s.contains("PROGRAM-ID. NC109M"), "content missing: {s:?}");
    assert!(s.contains("STOP RUN"), "content missing: {s:?}");
    // every line keeps its 6-char sequence column and the trailing source tag
    for line in s.lines() {
        assert!(line.len() >= 8, "short line {line:?}");
        assert!(
            line[..6].chars().all(|c| c.is_ascii_digit()),
            "sequence column broken: {line:?}"
        );
        assert!(line.len() >= 80, "source tag missing: {line:?}");
        assert_eq!(&line[72..], "NC1094.2", "tag not preserved: {line:?}");
    }
}

#[test]
fn splitting_preserves_fixed_format_bytes_verbatim() {
    // byte-for-byte: the materialized unit equals the source lines joined with '\n'
    let text = synthetic_corpus();
    let lines: Vec<&str> = text.lines().collect();
    let units = index_from_corpus(&text);
    let ix102 = units
        .iter()
        .find(|u| u.kind == "COBOL" && u.name.contains("IX102"))
        .unwrap();
    let bytes = unit_bytes(&lines, ix102);
    let expected = format!(
        "{}\n{}\n{}\n{}\n",
        fl("000100", " ", "IDENTIFICATION DIVISION.", "IX1024.2"),
        fl("000200", " ", "PROGRAM-ID. IX102A.", "IX1024.2"),
        fl("000300", " ", "PROCEDURE DIVISION.", "IX1024.2"),
        fl("000400", " ", "EXIT PROGRAM.", "IX1024.2"),
    );
    assert_eq!(String::from_utf8(bytes).unwrap(), expected);
}

#[test]
fn stable_hashing_is_deterministic() {
    let text = synthetic_corpus();
    let lines: Vec<&str> = text.lines().collect();
    let units = index_from_corpus(&text);
    let st = units.iter().find(|u| u.name.starts_with("ST140")).unwrap();
    let b1 = unit_bytes(&lines, st);
    let b2 = unit_bytes(&lines, st);
    assert_eq!(sha256_hex(&b1), sha256_hex(&b2));
    assert_eq!(sha256_hex(&b1).len(), 64);
}

#[test]
fn materialize_derives_stable_paths_kinds_and_dependencies() {
    let text = synthetic_corpus();
    let lines: Vec<&str> = text.lines().collect();
    let units = index_from_corpus(&text);
    let (_dir, work) = write_fixture();
    let materialized = materialize(&lines, &units, &work);

    // 5 units: 2 COBOL (NC109M, the IX101A main that maps to the IX102A subprogram file,
    // ST140A), 1 CLBRY, 1 DATA*.
    assert_eq!(materialized.len(), 5, "{materialized:#?}");

    let kind = |n: &str| {
        materialized
            .iter()
            .find(|u| u.name == n && u.kind != "DATA*")
            .unwrap()
            .kind
            .clone()
    };
    assert_eq!(kind("K1FDA"), "CLBRY");
    assert_eq!(kind("NC109M"), "COBOL");
    assert_eq!(kind("IX102A"), "COBOL");
    assert_eq!(kind("ST140A"), "COBOL");

    // ST140A's padded header name collapses to the first token -> a filesystem-safe file name.
    let st = materialized.iter().find(|u| u.name == "ST140A").unwrap();
    assert_eq!(st.source_path, "ST140A.cob");
    assert!(!st.source_path.contains(' '), "unstable path {st:?}");

    // The SUBRTN unit is named after the subprogram and binds to its main.
    let sub = materialized.iter().find(|u| u.name == "IX102A").unwrap();
    assert!(sub.subprogram.is_some());
    assert_eq!(sub.main_program.as_deref(), Some("IX101A"));
    assert!(!sub.is_executable_candidate);

    // CLBRY unit: raw copybook preserved under copybooks/ + site-adapted copy under
    // copybooks-adapted/ (the C-marked obsolete FD lines dropped).
    let cl = materialized.iter().find(|u| u.name == "K1FDA").unwrap();
    assert_eq!(cl.source_path, "copybooks/K1FDA.cpy");
    assert!(cl.adapted_path.starts_with("copybooks-adapted/"));
    let adapted = std::fs::read(work.join(&cl.adapted_path)).unwrap();
    let adapted_s = String::from_utf8_lossy(&adapted);
    assert!(
        !adapted_s.contains("VALUE OF"),
        "C-line not dropped: {adapted_s}"
    );
    assert!(adapted_s.contains("LABEL RECORDS STANDARD"));

    // DATA* unit: preserved verbatim under data/, and the same-named COBOL unit consumes it.
    let dat = materialized.iter().find(|u| u.kind == "DATA*").unwrap();
    assert_eq!(dat.source_path, "data/NC109M.dat");
    let nc = materialized.iter().find(|u| u.name == "NC109M").unwrap();
    assert_eq!(nc.data_dependencies, vec!["NC109M".to_string()]);

    // raw bytes are never modified: the materialized file hash matches the manifest hash
    let raw = std::fs::read(work.join(&nc.source_path)).unwrap();
    assert_eq!(sha256_hex(&raw), nc.source_sha256);
}

#[test]
fn dependency_extraction_finds_copy_and_marks_missing() {
    // A COBOL unit that COPYs an existing CLBRY (K1FDA) and a missing book (NOSUCH).
    let text = [
        "CCVS85  VERSION 4.0   01 OCT 1992 0032",
        "*HEADER,CLBRY,K1FDA",
        "000100     DATA RECORD IS TST-TEST.                               K1FDA4.2",
        "*END-OF,K1FDA",
        "*HEADER,COBOL,NC999A",
        "000100 IDENTIFICATION DIVISION.                                   NC9994.2",
        "000200 PROGRAM-ID. NC999A.                                        NC9994.2",
        "000300 DATA DIVISION.                                             NC9994.2",
        "000400 FILE SECTION.                                              NC9994.2",
        "000500 FD  TEST-FILE                                     COPY K1FDA.NC9994.2",
        "000600     COPY NOSUCH.                                           NC9994.2",
        "000700 PROCEDURE DIVISION.                                        NC9994.2",
        "000800     STOP RUN.                                              NC9994.2",
        "*END-OF,NC999A",
    ]
    .join("\n");
    let lines: Vec<&str> = text.lines().collect();
    let units = index_from_corpus(&text);
    let (_dir, work) = write_fixture();
    let materialized = materialize(&lines, &units, &work);
    let nc = materialized.iter().find(|u| u.name == "NC999A").unwrap();
    assert!(nc.copy_dependencies.contains(&"K1FDA".to_string()));
    assert!(nc.missing_copybooks.contains(&"NOSUCH".to_string()));
}

#[test]
fn site_adapt_drops_rerun_card_and_keeps_width() {
    let src = "002900 I-O-CONTROL.                                                     IX3024.2\n\
               003000     XXXXX053.                                                    IX3024.2\n\
               003100 DATA DIVISION.                                                   IX3024.2\n";
    let out = String::from_utf8(site_adapt(src.as_bytes())).unwrap();
    assert!(!out.contains("XXXXX053"), "RERUN card not dropped: {out}");
    assert!(out.contains("I-O-CONTROL."));
    assert!(out.contains("DATA DIVISION."));
    // width preservation: tags stay at column 73
    for (a, b) in src.lines().zip(out.lines()) {
        if a.len() >= 73 {
            assert_eq!(
                b.chars().skip(72).collect::<String>().trim_end(),
                a[72..].trim_end(),
                "tag shifted: {b}"
            );
        }
    }
}

#[test]
fn site_adapt_unifies_ccvs1_counters_per_definition() {
    // Generation A: defines C-NO-OF-TESTS -> references stay as-is.
    let gen_a = "008700P    05  C-NO-OF-TESTS       PIC 99.                              IX1014.2\n\
                 030200P    ADD 1 TO C-NO-OF-TESTS.                                      IX1014.2\n";
    let a = String::from_utf8(site_adapt(gen_a.as_bytes())).unwrap();
    assert!(a.contains("C-NO-OF-TESTS"), "{a}");

    // Generation B: defines NO-OF-TESTS but the prologue references C-NO-OF-TESTS -> unify to the
    // defined name, width-preserving.
    let gen_b = "008700P    05  NO-OF-TESTS         PIC 99.                              SQ1014.2\n\
                 043100P    ADD     1           TO C-NO-OF-TESTS.                        SQ1014.2\n";
    let b = String::from_utf8(site_adapt(gen_b.as_bytes())).unwrap();
    assert!(!b.contains("C-NO-OF-TESTS"), "{b}");
    assert!(b.contains("ADD     1           TO NO-OF-TESTS"), "{b}");
    for (orig, adapt) in gen_b.lines().zip(b.lines()) {
        if orig.len() >= 73 {
            assert_eq!(
                adapt.chars().skip(72).collect::<String>().trim_end(),
                orig[72..].trim_end()
            );
        }
    }
}

#[test]
fn site_adapt_unifies_delete_counter_when_defined_under_the_other_name() {
    // The module defines DELETE-COUNTER (not DELETE-CNT) while the prologue references DELETE-CNT:
    // this site renames the DEFINED field to the prologue contract name (shorter -> width-safe).
    let src = "033700 01  DELETE-COUNTER    PIC 999      VALUE ZERO.                   SQ1014.2\n\
               046100P    MOVE    DELETE-CNT    TO C-DELETED.                          SQ1014.2\n";
    let out = String::from_utf8(site_adapt(src.as_bytes())).unwrap();
    assert!(!out.contains("DELETE-COUNTER"), "{out}");
    assert!(out.contains("01  DELETE-CNT"), "{out}");
    // a module that defines DELETE-CNT is untouched
    let src2 = "014100 01  DELETE-CNT      PIC 999      VALUE ZERO.                    DB1014.2\n\
                046100P    MOVE    DELETE-CNT    TO C-DELETED.                          DB1014.2\n";
    let out2 = String::from_utf8(site_adapt(src2.as_bytes())).unwrap();
    assert!(out2.contains("DELETE-CNT"));
    assert!(!out2.contains("DELETE-COUNTER"));
}

#[test]
fn site_adapt_drops_unmarked_optional_x_card_sibling() {
    // `ASSIGN TO XXXXX024 / J XXXXX044` — the unmarked X-24 is the non-selected variant.
    let src = "006000     SELECT   IX-FS1 ASSIGN TO                                    IX1014.2\n\
               006100     XXXXX024                                                     IX1014.2\n\
               006200J    XXXXX044                                                     IX1014.2\n\
               006300     ORGANIZATION IS INDEXED                                      IX1014.2\n";
    let out = String::from_utf8(site_adapt(src.as_bytes())).unwrap();
    assert!(!out.contains("XXXXX024"), "unmarked sibling kept: {out}");
    assert!(out.contains("XXXXX044"), "selected card dropped: {out}");
    // reversed order (IX207A): marked first, unmarked second -> drop the unmarked one
    let src2 = "010000     ASSIGN TO                                                    IX2074.2\n\
                010100J    XXXXX044                                                     IX2074.2\n\
                010200     XXXXX024.                                                    IX2074.2\n";
    let out2 = String::from_utf8(site_adapt(src2.as_bytes())).unwrap();
    assert!(!out2.contains("XXXXX024"), "reversed sibling kept: {out2}");
    assert!(out2.contains("XXXXX044"));
}

#[test]
fn site_adapt_substitutes_device_cards_and_keeps_literals() {
    let src = "003700A    XXXXX051                                                     NC1084.2\n\
               003800A    IS ABBREV-SWITCH                                             NC1084.2\n\
               004100     XXXXX057                                                     NC2044.2\n\
               004200     IS ACCEPT-INPUT-DEVICE                                       NC2044.2\n\
               006000     MOVE \"GGGGHXXXXX052ALTKEY1\" TO X.                          NC1084.2\n";
    let out = String::from_utf8(site_adapt(src.as_bytes())).unwrap();
    assert!(out.contains("SWITCH-1"), "{out}");
    assert!(out.contains("CONSOLE"), "{out}");
    // literal-embedded token untouched
    assert!(out.contains("XXXXX052ALTKEY1"), "{out}");
}

#[test]
fn site_adapt_record_key_boundary_never_splits_data_names() {
    // `RECORD-KEY-CONTENT` is a data name and must stay hyphenated; the standalone clause token
    // `RECORD-KEY` is split into `RECORD KEY`.
    let src = "026400 01  RECORD-KEY-CONTENT.                                          IX1064.2\n\
               006900P          RECORD-KEY   IS RAW-DATA-KEY.                          IX1014.2\n";
    let out = String::from_utf8(site_adapt(src.as_bytes())).unwrap();
    assert!(out.contains("RECORD-KEY-CONTENT"), "data name split: {out}");
    assert!(out.contains("RECORD KEY   IS RAW-DATA-KEY"), "{out}");
}

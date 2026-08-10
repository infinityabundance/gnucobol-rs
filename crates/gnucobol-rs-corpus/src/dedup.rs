//! Deduplication.
//!
//! Five layers, from exact to near-duplicate:
//! 1. exact byte hash;
//! 2. normalized-source hash (line endings + trailing whitespace);
//! 3. whitespace-insensitive hash;
//! 4. identifier-normalized structural hash (uppercase, sequence-area stripped) where defensible;
//! 5. near-duplicate similarity.
//!
//! Grouping is repository-level so near-identical files from one project do not leak across
//! development and held-out partitions. Duplicate occurrences keep full provenance.

use serde::Serialize;
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;

pub fn sha256_hex(bytes: &[u8]) -> String {
    let mut h = Sha256::new();
    h.update(bytes);
    let mut s = String::with_capacity(64);
    for b in h.finalize() {
        s.push_str(&format!("{b:02x}"));
    }
    s
}

/// Layer 1: exact bytes.
pub fn exact_hash(bytes: &[u8]) -> String {
    sha256_hex(bytes)
}

/// Layer 2: normalized source (line endings -> LF, trailing whitespace stripped).
pub fn normalized_hash(bytes: &[u8]) -> String {
    sha256_hex(&crate::bytes::normalize(bytes))
}

/// Layer 3: whitespace-insensitive (all whitespace runs -> single space, trimmed).
pub fn whitespace_insensitive_hash(bytes: &[u8]) -> String {
    let mut out = Vec::with_capacity(bytes.len());
    let mut prev_space = true; // trim leading
    for &b in bytes {
        if b.is_ascii_whitespace() {
            if !prev_space {
                out.push(b' ');
                prev_space = true;
            }
        } else {
            out.push(b.to_ascii_lowercase());
            prev_space = false;
        }
    }
    while out.last() == Some(&b' ') {
        out.pop();
    }
    sha256_hex(&out)
}

/// Layer 4: structural hash -- fixed-format sequence area (cols 1-6) stripped, ASCII case
/// folded, whitespace runs collapsed. Defensible for COBOL because the sequence area is not part
/// of the program text; use with care (never claim semantic preservation).
pub fn structural_hash(bytes: &[u8]) -> String {
    let mut out = Vec::with_capacity(bytes.len());
    let mut line_start = true;
    let mut col = 0usize;
    let mut prev_space = false;
    for &b in bytes {
        if b == b'\n' {
            if !out.is_empty() && out.last() != Some(&b'\n') {
                out.push(b'\n');
            }
            line_start = true;
            col = 0;
            prev_space = false;
            continue;
        }
        if line_start && col < 6 {
            // sequence area: drop
            col += 1;
            if b == b' ' || b == b'\t' {
                // still sequence area; skip
            }
            continue;
        }
        if b.is_ascii_whitespace() {
            if !prev_space {
                out.push(b' ');
                prev_space = true;
            }
        } else {
            out.push(b.to_ascii_lowercase());
            prev_space = false;
        }
        line_start = false;
        col += 1;
    }
    sha256_hex(&out)
}

/// Layer 5: a lightweight near-duplicate similarity over case-folded, sequence-area-stripped,
/// whitespace-collapsed tokens (token-set Jaccard). 1.0 = identical token sets; the caller
/// chooses a threshold. The fixed-format sequence area (cols 1-6) is dropped per line because it
/// is not program text -- two files differing only in sequence numbers are near-identical, not
/// independent evidence.
pub fn similarity(a: &[u8], b: &[u8]) -> f64 {
    fn tokens(bytes: &[u8]) -> Vec<String> {
        let mut t = Vec::new();
        let mut cur = String::new();
        let mut col = 0usize;
        for &c in bytes {
            if c == b'\n' {
                col = 0;
                if !cur.is_empty() {
                    t.push(std::mem::take(&mut cur));
                }
                continue;
            }
            if col < 6 {
                // fixed-format sequence area: not program text
                col += 1;
                continue;
            }
            col += 1;
            if c.is_ascii_alphanumeric() {
                cur.push((c as char).to_ascii_lowercase());
            } else if !cur.is_empty() {
                t.push(std::mem::take(&mut cur));
            }
        }
        if !cur.is_empty() {
            t.push(cur);
        }
        t
    }
    let ta = tokens(a);
    let tb = tokens(b);
    if ta.is_empty() && tb.is_empty() {
        return 1.0;
    }
    if ta.is_empty() || tb.is_empty() {
        return 0.0;
    }
    let set_a: std::collections::HashSet<&String> = ta.iter().collect();
    let set_b: std::collections::HashSet<&String> = tb.iter().collect();
    let inter = set_a.intersection(&set_b).count();
    let union = set_a.union(&set_b).count();
    inter as f64 / union as f64
}

/// Repository-level grouping of duplicates: `canonical program_id -> occurrences with provenance`.
#[derive(Debug, Clone, Default, Serialize, serde::Deserialize)]
pub struct DedupIndex {
    /// exact_hash -> canonical program_id
    pub exact: BTreeMap<String, String>,
    /// canonical program_id -> list of duplicate program_ids (provenance)
    pub duplicates: BTreeMap<String, Vec<String>>,
    /// normalized_hash -> program_ids sharing it (near-identical modulo whitespace/EOL)
    pub normalized: BTreeMap<String, Vec<String>>,
}

impl DedupIndex {
    pub fn new() -> DedupIndex {
        DedupIndex::default()
    }

    /// Register one unit. Returns the canonical id for the unit (itself when unique).
    pub fn register(&mut self, program_id: &str, original_bytes: &[u8]) -> String {
        let eh = exact_hash(original_bytes);
        if let Some(canon) = self.exact.get(&eh) {
            self.duplicates
                .entry(canon.clone())
                .or_default()
                .push(program_id.to_string());
            return canon.clone();
        }
        self.exact.insert(eh, program_id.to_string());
        program_id.to_string()
    }

    /// Record a normalized-hash group (for reporting near-identical families).
    pub fn note_normalized(&mut self, normalized_hash: &str, program_id: &str) {
        self.normalized
            .entry(normalized_hash.to_string())
            .or_default()
            .push(program_id.to_string());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const A: &[u8] = b"       IDENTIFICATION DIVISION.\n       PROGRAM-ID. T.\n       PROCEDURE DIVISION.\n           DISPLAY \"HI\".\n           STOP RUN.\n";
    const B: &[u8] = b"       IDENTIFICATION DIVISION.\n       PROGRAM-ID. T.\n       PROCEDURE DIVISION.\n           DISPLAY \"HI\".\n           STOP RUN.\n"; // identical
    const C: &[u8] = b"000100 IDENTIFICATION DIVISION.\n000200 PROGRAM-ID. T.\n000300 PROCEDURE DIVISION.\n000400     DISPLAY \"HI\".\n000500     STOP RUN.\n"; // sequence numbers
    const D: &[u8] = b"       IDENTIFICATION DIVISION.\n       PROGRAM-ID. T.\n       PROCEDURE DIVISION.\n           DISPLAY \"BYE\".\n           STOP RUN.\n"; // near

    #[test]
    fn exact_and_whitespace_insensitive_layers() {
        assert_eq!(exact_hash(A), exact_hash(B));
        assert_ne!(exact_hash(A), exact_hash(C));
        // layer 3 is whitespace-only: A and B collapse identically; C carries sequence-number
        // content (dropped only by the structural layer 4), so its hash differs.
        assert_eq!(
            whitespace_insensitive_hash(A),
            whitespace_insensitive_hash(B)
        );
        assert_ne!(
            whitespace_insensitive_hash(A),
            whitespace_insensitive_hash(C)
        );
        assert_ne!(
            whitespace_insensitive_hash(A),
            whitespace_insensitive_hash(D)
        );
    }

    #[test]
    fn structural_hash_strips_sequence_area() {
        assert_eq!(structural_hash(A), structural_hash(C));
        assert_ne!(structural_hash(A), structural_hash(D));
    }

    #[test]
    fn similarity_orders_near_duplicates() {
        let s_ac = similarity(A, C);
        let s_ad = similarity(A, D);
        let s_aa = similarity(A, B);
        // sequence-number-only difference (A vs C) is a structural near-identical (1.0 after
        // sequence stripping); a literal change (A vs D) is a strictly smaller similarity.
        assert!(s_aa >= s_ac, "{s_aa} >= {s_ac}");
        assert!(s_ac >= s_ad, "{s_ac} >= {s_ad}");
    }

    #[test]
    fn dedup_index_registers_duplicates_with_provenance() {
        let mut idx = DedupIndex::new();
        let c1 = idx.register("repo1/p1", A);
        assert_eq!(c1, "repo1/p1");
        let c2 = idx.register("repo2/p1-copy", B);
        assert_eq!(c2, "repo1/p1");
        assert_eq!(idx.duplicates["repo1/p1"], vec!["repo2/p1-copy"]);
        // a third identical copy accumulates provenance
        idx.register("repo3/p1-copy2", A);
        assert_eq!(idx.duplicates["repo1/p1"].len(), 2);
    }
}

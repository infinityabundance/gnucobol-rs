//! `GNURUST.CCVS85.1` — external corpus custody/index for the NIST CCVS85 COBOL-85 validation bundle.
//!
//! This proves only CORPUS CUSTODY: the compressed spine's hash, a reproducible decompression, the
//! decompressed hash, and stable split/index metadata. It makes NO conformance claim — CCVS85 is admitted
//! as a historical regression gauntlet, not a byte-parity oracle. (Per-function byte parity stays with the
//! oracle sweeps; this corpus is broader and looser.)
//!
//! `ingest` decompresses, hashes, splits by `*HEADER`, and writes the index + receipt. `check` re-derives
//! the same metadata from the committed `.Z` and diffs it against the committed receipt (a freshness gate).

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Default committed location of the compressed corpus spine.
pub const DEFAULT_INPUT: &str = "lab/corpus/ccvs85/newcob.val.Z";
/// Default output directory for the generated index.
pub const DEFAULT_OUT: &str = "reports/ccvs85";
/// The generated, freshness-gated provenance receipt.
pub const RECEIPT_MD: &str = "reports/provenance/ccvs85-corpus-ingest-receipt.md";
pub const RECEIPT_JSON: &str = "reports/provenance/ccvs85-corpus-ingest-receipt.json";

/// The corpus-custody facts asserted by `GNURUST.CCVS85.1` (the receipt body).
#[derive(Serialize, Deserialize, PartialEq, Clone)]
pub struct Receipt {
    pub gate: String,
    pub corpus: String,
    pub dialect: String,
    pub conformance_claim: String,
    pub source_name: String,
    pub compressed_sha256: String,
    pub compressed_bytes: u64,
    pub decompressed_sha256: String,
    pub decompressed_bytes: u64,
    pub decompressed_lines: u64,
    pub decompressor: String,
    pub version_banner: String,
    pub header_count: usize,
    pub header_by_kind: BTreeMap<String, usize>,
    pub program_id_count: usize,
    pub end_of_count: usize,
    pub unit_count: usize,
}

/// One split unit (a `*HEADER,<kind>,<name>` section and the lines until the next header).
#[derive(Serialize)]
struct Unit {
    index: usize,
    kind: String,
    name: String,
    start_line: usize,
    end_line: usize,
}

// ---- minimal SHA-256 (FIPS 180-4) — no external crate, reproducible corpus hashing ----------------

const SHA_K: [u32; 64] = [
    0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5,
    0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174,
    0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
    0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7, 0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967,
    0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85,
    0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
    0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
    0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2,
];

fn sha256_hex(data: &[u8]) -> String {
    let mut h: [u32; 8] = [
        0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19,
    ];
    let bitlen = (data.len() as u64).wrapping_mul(8);
    let mut msg = data.to_vec();
    msg.push(0x80);
    while msg.len() % 64 != 56 {
        msg.push(0);
    }
    msg.extend_from_slice(&bitlen.to_be_bytes());
    for chunk in msg.chunks(64) {
        let mut w = [0u32; 64];
        for (i, word) in w.iter_mut().enumerate().take(16) {
            *word = u32::from_be_bytes([chunk[i * 4], chunk[i * 4 + 1], chunk[i * 4 + 2], chunk[i * 4 + 3]]);
        }
        for i in 16..64 {
            let s0 = w[i - 15].rotate_right(7) ^ w[i - 15].rotate_right(18) ^ (w[i - 15] >> 3);
            let s1 = w[i - 2].rotate_right(17) ^ w[i - 2].rotate_right(19) ^ (w[i - 2] >> 10);
            w[i] = w[i - 16].wrapping_add(s0).wrapping_add(w[i - 7]).wrapping_add(s1);
        }
        let (mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut hh) =
            (h[0], h[1], h[2], h[3], h[4], h[5], h[6], h[7]);
        for i in 0..64 {
            let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let ch = (e & f) ^ ((!e) & g);
            let t1 = hh.wrapping_add(s1).wrapping_add(ch).wrapping_add(SHA_K[i]).wrapping_add(w[i]);
            let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
            let maj = (a & b) ^ (a & c) ^ (b & c);
            let t2 = s0.wrapping_add(maj);
            hh = g;
            g = f;
            f = e;
            e = d.wrapping_add(t1);
            d = c;
            c = b;
            b = a;
            a = t1.wrapping_add(t2);
        }
        h[0] = h[0].wrapping_add(a);
        h[1] = h[1].wrapping_add(b);
        h[2] = h[2].wrapping_add(c);
        h[3] = h[3].wrapping_add(d);
        h[4] = h[4].wrapping_add(e);
        h[5] = h[5].wrapping_add(f);
        h[6] = h[6].wrapping_add(g);
        h[7] = h[7].wrapping_add(hh);
    }
    h.iter().map(|x| format!("{x:08x}")).collect()
}

// ---- corpus derivation --------------------------------------------------------------------------

/// Decompress the `.Z` spine via `gzip -dc` (which reads Unix `compress` data), returning the bytes and the
/// decompressor's identity string.
fn decompress(input: &Path) -> Option<(Vec<u8>, String)> {
    let out = Command::new("gzip").arg("-dc").arg(input).output().ok()?;
    if !out.status.success() || out.stdout.is_empty() {
        return None;
    }
    let ver = Command::new("gzip").arg("--version").output().ok()?;
    let ident = String::from_utf8_lossy(&ver.stdout).lines().next().unwrap_or("gzip").trim().to_string();
    Some((out.stdout, ident))
}

/// Derive the receipt + the per-unit index from the compressed spine. Returns `None` if the spine is absent.
fn derive(input: &Path) -> Option<(Receipt, Vec<Unit>)> {
    let compressed = std::fs::read(input).ok()?;
    let (decompressed, decompressor) = decompress(input)?;

    let text = String::from_utf8_lossy(&decompressed);
    let lines: Vec<&str> = text.lines().collect();

    let version_banner = lines.first().unwrap_or(&"").trim_end().to_string();
    let mut header_by_kind: BTreeMap<String, usize> = BTreeMap::new();
    let mut program_id_count = 0usize;
    let mut end_of_count = 0usize;
    let mut units: Vec<Unit> = Vec::new();

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
            units.push(Unit { index: units.len(), kind, name, start_line: i + 1, end_line: lines.len() });
        } else if line.contains("PROGRAM-ID") {
            program_id_count += 1;
        }
        if line.starts_with("*END-OF") {
            end_of_count += 1;
        }
    }

    let receipt = Receipt {
        gate: "GNURUST.CCVS85.1".to_string(),
        corpus: "CCVS85".to_string(),
        dialect: "COBOL-85 validation (NIST CCVS85, VERSION 4.0)".to_string(),
        conformance_claim: "NONE — corpus custody/index only; no COBOL-85 conformance, suite-pass, \
            compiler-replacement, or libcob behaviour-parity claim."
            .to_string(),
        source_name: input.file_name().map(|s| s.to_string_lossy().to_string()).unwrap_or_default(),
        compressed_sha256: sha256_hex(&compressed),
        compressed_bytes: compressed.len() as u64,
        decompressed_sha256: sha256_hex(&decompressed),
        decompressed_bytes: decompressed.len() as u64,
        decompressed_lines: lines.len() as u64,
        decompressor,
        version_banner,
        header_count: header_by_kind.values().sum(),
        header_by_kind,
        program_id_count,
        end_of_count,
        unit_count: units.len(),
    };
    Some((receipt, units))
}

fn receipt_md(r: &Receipt) -> String {
    let mut kinds = String::new();
    for (k, v) in &r.header_by_kind {
        kinds.push_str(&format!("  - `{k}`: {v}\n"));
    }
    format!(
        "# GNURUST.CCVS85.1 — CCVS85 corpus ingest receipt\n\n\
        **GENERATED** by `cargo run -p gnucobol-rs-port-index -- ccvs85 ingest` — do not edit by hand.\n\n\
        `GNURUST.CCVS85.1` admits the historical **{corpus}** COBOL-85 validation corpus as an external\n\
        regression gauntlet. It proves only **corpus custody**: the compressed spine's hash, a reproducible\n\
        decompression, the decompressed hash, and stable split/index metadata.\n\n\
        **Conformance claim:** {claim}\n\n\
        ## Custody\n\n\
        | fact | value |\n|---|---|\n\
        | source | `{source}` |\n\
        | compressed sha256 | `{csha}` |\n\
        | compressed bytes | {cbytes} |\n\
        | decompressor | {dtool} |\n\
        | decompressed sha256 | `{dsha}` |\n\
        | decompressed bytes | {dbytes} |\n\
        | decompressed lines | {dlines} |\n\
        | version banner | `{banner}` |\n\n\
        ## Index (no conformance claim)\n\n\
        - dialect: {dialect}\n\
        - split units (`*HEADER`): **{units}**\n\
        - `*HEADER` records: {hcount}\n{kinds}\
        - `PROGRAM-ID` lines: {pid}\n\
        - `*END-OF` records: {endof}\n\n\
        The per-unit index (kind, name, line range) is in `reports/ccvs85/corpus-index.json`.\n\n\
        ## Boundary\n\n\
        This milestone makes **no** COBOL-85 conformance or suite-pass claim. CCVS85 is broad and old; it\n\
        can expose missing compiler/runtime behaviour (work discovery), but per-function byte parity stays\n\
        with the oracle sweeps. Compile/run baselines are deferred to later tiered gates\n\
        (`GNURUST.CCVS85.2`/`.3`/`.4`).\n",
        corpus = r.corpus,
        claim = r.conformance_claim,
        source = r.source_name,
        csha = r.compressed_sha256,
        cbytes = r.compressed_bytes,
        dtool = r.decompressor,
        dsha = r.decompressed_sha256,
        dbytes = r.decompressed_bytes,
        dlines = r.decompressed_lines,
        banner = r.version_banner,
        dialect = r.dialect,
        units = r.unit_count,
        hcount = r.header_count,
        kinds = kinds,
        pid = r.program_id_count,
        endof = r.end_of_count,
    )
}

/// `ccvs85 ingest --input <.Z> --out <dir>`: derive + write the index, receipt JSON, and receipt MD.
pub fn ingest(root: &Path, input: Option<PathBuf>, out: Option<PathBuf>) -> i32 {
    let input = input.unwrap_or_else(|| root.join(DEFAULT_INPUT));
    let out = out.unwrap_or_else(|| root.join(DEFAULT_OUT));
    let (receipt, units) = match derive(&input) {
        Some(v) => v,
        None => {
            println!("GNURUST.CCVS85.1: corpus spine absent or gzip unavailable — ingest skipped");
            return 0;
        }
    };
    let _ = std::fs::create_dir_all(&out);
    let _ = std::fs::create_dir_all(root.join("reports/provenance"));
    let _ = std::fs::write(out.join("corpus-index.json"), serde_json::to_string_pretty(&units).unwrap() + "\n");
    let _ = std::fs::write(root.join(RECEIPT_JSON), serde_json::to_string_pretty(&receipt).unwrap() + "\n");
    let _ = std::fs::write(root.join(RECEIPT_MD), receipt_md(&receipt));
    println!(
        "GNURUST.CCVS85.1: {} units, {} headers, {} PROGRAM-ID; corpus sha256 {}",
        receipt.unit_count, receipt.header_count, receipt.program_id_count, &receipt.compressed_sha256[..16]
    );
    0
}

/// `ccvs85 check`: re-derive from the committed `.Z` and diff against the committed receipt JSON (freshness).
pub fn check(root: &Path) -> i32 {
    let input = root.join(DEFAULT_INPUT);
    let fresh = match derive(&input) {
        Some((r, _)) => r,
        None => {
            println!("GNURUST.CCVS85.1: corpus spine absent — check skipped");
            return 0;
        }
    };
    let committed: Option<Receipt> = std::fs::read_to_string(root.join(RECEIPT_JSON))
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok());
    match committed {
        Some(c) if c == fresh => {
            println!("GNURUST.CCVS85.1: corpus custody STABLE ({} units, corpus sha256 {})", fresh.unit_count, &fresh.compressed_sha256[..16]);
            0
        }
        Some(_) => {
            eprintln!("GNURUST.CCVS85.1 STALE: committed receipt != re-derived corpus metadata (run `ccvs85 ingest`)");
            1
        }
        None => {
            eprintln!("GNURUST.CCVS85.1 STALE: receipt JSON missing (run `ccvs85 ingest`)");
            1
        }
    }
}

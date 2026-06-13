//! Rust mirror for the line-sequential WRITE config-matrix sweep (`GNURUST.FILEIO.LINESEQ.1`).
//! Reads `<tag>=<value>` lines (the oracle's output-file bytes in hex, or a 2-char FILE STATUS) and
//! compares to [`gnucobol_rs::fileio`] for the same records under the same `COB_LS_*` config. PASS=n FAIL=n.
use gnucobol_rs::fileio::{write_line_sequential, LineSeqConfig};
use std::io::BufRead;

// Each config mirrors the exact COB_LS_* env the oracle program runs under.
const PLAIN: LineSeqConfig = LineSeqConfig { ls_fixed: false, ls_nulls: false, ls_validate: false }; // COB_LS_VALIDATE=0
const NULLS: LineSeqConfig = LineSeqConfig { ls_fixed: false, ls_nulls: true, ls_validate: false }; // VALIDATE=0 NULLS=1
const FIXED: LineSeqConfig = LineSeqConfig { ls_fixed: true, ls_nulls: false, ls_validate: true }; // COB_LS_FIXED=1 (validate default 1)
const FIXED_NULLS: LineSeqConfig = LineSeqConfig { ls_fixed: true, ls_nulls: true, ls_validate: false }; // VALIDATE=0 NULLS=1 FIXED=1

// The fixed-width PIC X(8) FD record areas (space-padded) the oracle program writes.
fn valid() -> Vec<&'static [u8]> {
    vec![b"AB      ", b"HELLO123", b"        ", b"XY      ", b"12345678"]
}
fn ctrl() -> Vec<&'static [u8]> {
    vec![b"AB      ", b"A\x09B     ", b"XY      "]
}
fn hex(v: &[u8]) -> String {
    v.iter().map(|b| format!("{b:02x}")).collect()
}

fn main() {
    let (mut pass, mut fail) = (0u32, 0u32);
    for line in std::io::stdin().lock().lines().map_while(Result::ok) {
        let Some((tag, oracle)) = line.split_once('=') else { continue };
        let oracle = oracle.trim();
        let mine = match tag {
            "valid_default" => hex(&write_line_sequential(&valid(), &LineSeqConfig::DEFAULT).0),
            "valid_plain" => hex(&write_line_sequential(&valid(), &PLAIN).0),
            "valid_fixed" => hex(&write_line_sequential(&valid(), &FIXED).0),
            "ctrl_plain" => hex(&write_line_sequential(&ctrl(), &PLAIN).0),
            "ctrl_nulls" => hex(&write_line_sequential(&ctrl(), &NULLS).0),
            "ctrl_fixednulls" => hex(&write_line_sequential(&ctrl(), &FIXED_NULLS).0),
            // a single bad-char record under default validate: nothing written, status 71.
            "ctrl_default_bytes" => hex(&write_line_sequential(&[b"A\x09B     "], &LineSeqConfig::DEFAULT).0),
            "ctrl_default_status" => write_line_sequential(&[b"A\x09B     "], &LineSeqConfig::DEFAULT).1.to_string(),
            _ => continue,
        };
        if mine == oracle {
            pass += 1;
        } else {
            println!("{tag} FAIL mine={mine} oracle={oracle}");
            fail += 1;
        }
    }
    println!("PASS={pass} FAIL={fail}");
}

//! Rust mirror for the line-sequential READ config-matrix sweep (`GNURUST.FILEIO.LINESEQ.2`).
//! Reads `<tag>=<hex>` (the oracle's per-READ log: concatenated 10-byte rows of FILE STATUS [2 ASCII]
//! + the 8-byte record area) and compares to [`gnucobol_rs::fileio::read_line_sequential`]. PASS=n FAIL=n.
use gnucobol_rs::fileio::{read_line_sequential, LineSeqConfig};
use std::io::BufRead;

const DEF: LineSeqConfig = LineSeqConfig::DEFAULT;
const PLAIN: LineSeqConfig = LineSeqConfig { ls_fixed: false, ls_nulls: false, ls_validate: false, ls_split: true };
const NULLS: LineSeqConfig = LineSeqConfig { ls_fixed: false, ls_nulls: true, ls_validate: false, ls_split: true };
const NOSPLIT: LineSeqConfig = LineSeqConfig { ls_fixed: false, ls_nulls: false, ls_validate: true, ls_split: false };

// (file bytes, config) for each tag — the same inputs the oracle program reads.
fn case(tag: &str) -> Option<(&'static [u8], LineSeqConfig)> {
    Some(match tag {
        "basic" => (b"AB\nCD\n", DEF),
        "crlf" => (b"AB\r\nCD\n", DEF),
        "lonecr_def" => (b"A\rB\n", DEF),
        "lonecr_plain" => (b"A\rB\n", PLAIN),
        "long_split" => (b"ABCDEFGHIJ\n", DEF),
        "long_nosplit" => (b"ABCDEFGHIJ\n", NOSPLIT),
        "exact8" => (b"ABCDEFGH\n", DEF),
        "nulls" => (b"A\x00\x09B\x00\x09B\x00\x00\n", NULLS),
        "tab_def" => (b"A\x09B\n", DEF),
        "plain_ctrl" => (b"A\x09B\n", PLAIN),
        "mid_empty" => (b"AB\n\nCD\n", DEF),
        "no_trail" => (b"AB", DEF),
        _ => return None,
    })
}

fn hex(v: &[u8]) -> String {
    v.iter().map(|b| format!("{b:02x}")).collect()
}

fn main() {
    let (mut pass, mut fail) = (0u32, 0u32);
    for line in std::io::stdin().lock().lines().map_while(Result::ok) {
        let Some((tag, oracle)) = line.split_once('=') else { continue };
        let Some((data, cfg)) = case(tag) else { continue };
        // build the 10-byte rows (status[2 ASCII] + record[8]) for each non-AT-END read
        let mut mine = String::new();
        for r in read_line_sequential(data, 8, &cfg) {
            if r.at_end {
                break;
            }
            mine.push_str(&hex(r.status.as_bytes()));
            mine.push_str(&hex(&r.record));
        }
        if mine == oracle.trim() {
            pass += 1;
        } else {
            println!("{tag} FAIL mine={mine} oracle={}", oracle.trim());
            fail += 1;
        }
    }
    println!("PASS={pass} FAIL={fail}");
}

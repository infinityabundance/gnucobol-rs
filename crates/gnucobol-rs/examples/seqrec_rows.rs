//! Rust mirror for the RECORD SEQUENTIAL sweep (`GNURUST.FILEIO.SEQ.1`). Compares
//! [`gnucobol_rs::fileio`] `sequential_write` / `sequential_read` to the oracle: variable-length WRITE
//! file bytes across `COB_VARSEQ_FORMAT` 0-3, fixed WRITE bytes, and variable READ (status+size). PASS=n FAIL=n.
use gnucobol_rs::fileio::{sequential_read, sequential_write};
use std::io::BufRead;

// variable-length records the oracle writes: (8-byte FD area, live size)
fn var_recs() -> Vec<(&'static [u8], usize)> {
    vec![(b"AB      ", 2), (b"HELLO   ", 5), (b"XYZ12678", 8)]
}
fn fixed_recs() -> Vec<&'static [u8]> {
    vec![b"AB      ", b"HELLO123", b"        "]
}
fn hex(v: &[u8]) -> String {
    v.iter().map(|b| format!("{b:02x}")).collect()
}

// concatenated variable WRITE file bytes for a format
fn var_write(ty: u8) -> Vec<u8> {
    let mut out = Vec::new();
    for (d, s) in var_recs() {
        out.extend_from_slice(&sequential_write(d, s, true, ty));
    }
    out
}

// variable READ back: "<st><len:02>" per record, concatenated
fn var_read(ty: u8) -> String {
    let file = var_write(ty);
    let mut buf = vec![b' '; 8];
    let mut pos = 0usize;
    let mut s = String::new();
    loop {
        let r = sequential_read(&file, &mut pos, &mut buf, 1, 8, ty);
        if r.at_end {
            break;
        }
        s.push_str(&format!("{}{:02}", r.status, r.size));
    }
    s
}

fn main() {
    let (mut pass, mut fail) = (0u32, 0u32);
    for line in std::io::stdin().lock().lines().map_while(Result::ok) {
        let Some((tag, oracle)) = line.split_once('=') else { continue };
        let oracle = oracle.trim();
        let mine = match tag {
            "vw0" => hex(&var_write(0)),
            "vw1" => hex(&var_write(1)),
            "vw2" => hex(&var_write(2)),
            "vw3" => hex(&var_write(3)),
            "fw" => {
                let mut out = Vec::new();
                for r in fixed_recs() {
                    out.extend_from_slice(&sequential_write(r, 8, false, 0));
                }
                hex(&out)
            }
            "vr0" => var_read(0),
            "vr1" => var_read(1),
            "vr2" => var_read(2),
            "vr3" => var_read(3),
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

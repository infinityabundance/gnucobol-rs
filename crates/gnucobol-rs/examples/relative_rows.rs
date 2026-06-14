//! Rust mirror for the RELATIVE sweep (`GNURUST.FILEIO.RELATIVE.1`). Replays the same keyed
//! WRITE/DELETE/READ sequence as the oracle and compares the final file bytes, the per-op FILE STATUS
//! string, and a READ NEXT scan to [`gnucobol_rs::fileio`]. PASS=n FAIL=n.
use gnucobol_rs::fileio::{
    relative_delete, relative_read, relative_read_next, relative_rewrite, relative_write,
};
use std::io::BufRead;

const RMAX: usize = 4;

fn hex(v: &[u8]) -> String {
    v.iter().map(|b| format!("{b:02x}")).collect()
}

// the scenario: write key1=AAAA, key3=CCCC, delete key1, write key5=EEEE, rewrite key3=ZZZZ
fn build() -> (Vec<u8>, String) {
    let mut sts = String::new();
    let w = relative_write(&[], b"AAAA", RMAX, RMAX, 1);
    sts.push_str(w.status);
    let w = relative_write(&w.file, b"CCCC", RMAX, RMAX, 3);
    sts.push_str(w.status);
    let d = relative_delete(&w.file, RMAX, 1);
    sts.push_str(d.status);
    let w = relative_write(&d.file, b"EEEE", RMAX, RMAX, 5);
    sts.push_str(w.status);
    let rw = relative_rewrite(&w.file, b"ZZZZ", RMAX, 3);
    sts.push_str(rw.status);
    (rw.file, sts)
}

fn main() {
    let (file, opstatus) = build();
    // read keys 1 (deleted), 2 (empty), 3 (active) -> statuses
    let mut readsts = String::new();
    for k in [1i64, 2, 3] {
        readsts.push_str(relative_read(&file, RMAX, k).status);
    }
    // READ NEXT scan -> the active record data, concatenated, then the terminal status
    let mut scan = String::new();
    let mut slot = 0usize;
    loop {
        let r = relative_read_next(&file, &mut slot, RMAX);
        if r.status != "00" {
            scan.push_str(r.status);
            break;
        }
        scan.push_str(&String::from_utf8_lossy(&r.data));
    }

    let (mut pass, mut fail) = (0u32, 0u32);
    for line in std::io::stdin().lock().lines().map_while(Result::ok) {
        let Some((tag, oracle)) = line.split_once('=') else { continue };
        let oracle = oracle.trim();
        let mine = match tag {
            "relfile" => hex(&file),
            "opstatus" => opstatus.clone(),
            "readsts" => readsts.clone(),
            "scan" => scan.clone(),
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

//! Rust mirror for the file-runtime OPEN/CLOSE sweep (`GNURUST.FILEIO.OPEN.1`). Replays the same
//! OPEN/WRITE/READ/CLOSE sequence as the oracle (on its own temp file) and compares the file image
//! bytes and the open/close FILE STATUS sequence to [`gnucobol_rs::fileio`]'s `CobFile` + `cob_open`/
//! `cob_close`. PASS=n FAIL=n.
use gnucobol_rs::fileio::{cob_close, cob_open, AccessMode, CobFile, OpenMode, Organization};
use std::io::BufRead;

fn run() -> (String, String) {
    let base = std::env::temp_dir().join("gnucobol_rs_open_sweep_mirror");
    let _ = std::fs::remove_dir_all(&base);
    std::fs::create_dir_all(&base).unwrap();
    let p = base.join("ls.dat");
    let ps = p.to_str().unwrap();
    let mut st: Vec<&str> = Vec::new();
    let mut f = CobFile::new(Organization::LineSequential, AccessMode::Sequential, 8, ps);
    st.push(cob_open(&mut f, OpenMode::Output));
    st.push(cob_open(&mut f, OpenMode::Output)); // 41 already open
    f.write_record(b"AB", 0);
    f.write_record(b"XY", 0);
    st.push(cob_close(&mut f, false));
    st.push(cob_close(&mut f, false)); // 42 not open
    st.push(cob_open(&mut f, OpenMode::Input));
    f.read_record();
    f.read_record();
    cob_close(&mut f, true); // close with lock (status not captured, matching the oracle)
    st.push(cob_open(&mut f, OpenMode::Input)); // 38 closed with lock
    // missing-file open -> 35
    let mut m = CobFile::new(Organization::LineSequential, AccessMode::Sequential, 8, base.join("none.dat").to_str().unwrap());
    st.push(cob_open(&mut m, OpenMode::Input));
    let image = std::fs::read(ps).unwrap_or_default();
    let _ = std::fs::remove_dir_all(&base);
    (st.join(","), image.iter().map(|b| format!("{b:02x}")).collect())
}

fn main() {
    let (statuses, image) = run();
    let (mut pass, mut fail) = (0u32, 0u32);
    for line in std::io::stdin().lock().lines().map_while(Result::ok) {
        let Some((tag, oracle)) = line.split_once('=') else { continue };
        let oracle = oracle.trim();
        let mine = match tag {
            "statuses" => statuses.clone(),
            "image" => image.clone(),
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

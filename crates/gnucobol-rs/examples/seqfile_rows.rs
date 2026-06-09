//! Rust mirror for the sequential-file sweep (`GNURUST.FILE.SEQUENTIAL.1`). Reads the gen_seqfile TSV + the
//! oracle's READ events, runs read_sequential, and compares the (record bytes, status) sequence. PASS=n FAIL=n.
use gnucobol_rs::file_seq::{read_sequential, FileOrg};
use std::io::BufRead;
fn unhex(s: &str) -> Vec<u8> {
    (0..s.len() / 2).map(|k| u8::from_str_radix(&s[k * 2..k * 2 + 2], 16).unwrap_or(0)).collect()
}
fn main() {
    let (mut pass, mut fail) = (0u32, 0u32);
    for line in std::io::stdin().lock().lines().map_while(Result::ok) {
        let f: Vec<&str> = line.split('\t').collect();
        if f.len() < 5 { continue; }
        let (label, org, reclen, filehex, events) = (f[0], f[1], f[2].parse::<usize>().unwrap_or(8), f[3], f[4]);
        let data = unhex(filehex);
        let o = if org == "RECORD" { FileOrg::RecordSequential } else { FileOrg::LineSequential };
        let mine = read_sequential(&data, o, reclen);
        // oracle events: "R:<recordhex>:<status>" or "E:<status>", ';'-joined
        let mut oracle: Vec<(Vec<u8>, String, bool)> = Vec::new();
        for ev in events.split(';').filter(|s| !s.is_empty()) {
            let p: Vec<&str> = ev.split(':').collect();
            if p[0] == "R" { oracle.push((unhex(p[1]), p[2].to_string(), false)); }
            else { oracle.push((Vec::new(), p[1].to_string(), true)); }
        }
        let ok = mine.len() == oracle.len()
            && mine.iter().zip(&oracle).all(|(m, (rec, st, end))| m.at_end == *end && &m.record == rec && m.status == st.as_str());
        if ok { pass += 1; } else {
            println!("{label} FAIL org={org} mine={:?} oracle={:?}",
                mine.iter().map(|m| (String::from_utf8_lossy(&m.record).into_owned(), m.status, m.at_end)).collect::<Vec<_>>(), oracle);
            fail += 1;
        }
    }
    println!("PASS={pass} FAIL={fail}");
}

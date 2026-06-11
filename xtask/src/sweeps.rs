//! Oracle-sweep glue replacing the inline-python in the byte-court *_sweep.sh scripts. These parse cobc's
//! marker-delimited output and append the extracted bytes; the byte COMPARISON stays in the Rust *_rows
//! examples. No court semantics change -- this is faithful binary parsing.
use std::path::Path;

fn hexd(b: &[u8]) -> String { let mut s = String::new(); for x in b { s.push_str(&format!("{x:02x}")); } s }
fn read(p: &str) -> String { std::fs::read_to_string(p).unwrap_or_default() }
fn readb(p: &str) -> Vec<u8> { std::fs::read(p).unwrap_or_default() }

fn find_from(buf: &[u8], marker: &[u8], cur: usize) -> Option<usize> {
    if marker.is_empty() || cur > buf.len() { return None; }
    buf[cur..].windows(marker.len()).position(|w| w == marker).map(|p| cur + p)
}

/// find `label[`, extract N bytes (N fixed or = cases column `col:K`) as hex, append to the full row.
pub fn extract(cases: &str, out: &str, bytes_spec: &str) -> i32 {
    let buf = readb(out);
    let mut cur = 0usize;
    for line in read(cases).lines() {
        let cols: Vec<&str> = line.split('\t').collect();
        let label = cols.first().copied().unwrap_or("");
        let n: usize = if let Some(k) = bytes_spec.strip_prefix("col:") {
            cols.get(k.parse::<usize>().unwrap_or(0)).and_then(|s| s.parse().ok()).unwrap_or(0)
        } else { bytes_spec.parse().unwrap_or(0) };
        let marker = format!("{label}[").into_bytes();
        let hex = match find_from(&buf, &marker, cur) {
            Some(m) => { let s = m + marker.len(); cur = s + n; hexd(&buf[s..(s + n).min(buf.len())]) }
            None => String::new(),
        };
        println!("{line}\t{hex}");
    }
    0
}

/// remainder: dual markers `labelQ[` (csz=col10) and `labelR[` (dsz=col13) -> full row + qhex + rhex.
pub fn remainder(cases: &str, out: &str) -> i32 {
    let buf = readb(out);
    let mut cur = 0usize;
    for line in read(cases).lines() {
        let cols: Vec<&str> = line.split('\t').collect();
        let label = cols.first().copied().unwrap_or("");
        let csz: usize = cols.get(10).and_then(|s| s.parse().ok()).unwrap_or(0);
        let dsz: usize = cols.get(13).and_then(|s| s.parse().ok()).unwrap_or(0);
        let mq = format!("{label}Q[").into_bytes();
        let qhex = match find_from(&buf, &mq, cur) { Some(m) => { let s = m + mq.len(); cur = s + csz; hexd(&buf[s..(s + csz).min(buf.len())]) } None => String::new() };
        let mr = format!("{label}R[").into_bytes();
        let rhex = match find_from(&buf, &mr, cur) { Some(m) => { let s = m + mr.len(); cur = s + dsz; hexd(&buf[s..(s + dsz).min(buf.len())]) } None => String::new() };
        println!("{line}\t{qhex}\t{rhex}");
    }
    0
}

/// ordchar: per-row op. ORD -> marker `label=`, 4 bytes latin1; else -> marker `label[`, 1 byte hex.
pub fn ordchar(cases: &str, out: &str) -> i32 {
    let buf = readb(out);
    let mut cur = 0usize;
    for line in read(cases).lines() {
        let cols: Vec<&str> = line.split('\t').collect();
        let (label, op, arg) = (cols.first().copied().unwrap_or(""), cols.get(1).copied().unwrap_or(""), cols.get(2).copied().unwrap_or(""));
        let val = if op == "ORD" {
            let m = format!("{label}=").into_bytes();
            match find_from(&buf, &m, cur) { Some(p) => { let s = p + m.len(); cur = s + 4; buf[s..(s + 4).min(buf.len())].iter().map(|&b| b as char).collect::<String>() } None => String::new() }
        } else {
            let m = format!("{label}[").into_bytes();
            match find_from(&buf, &m, cur) { Some(p) => { let s = p + m.len(); cur = s + 1; hexd(&buf[s..(s + 1).min(buf.len())]) } None => String::new() }
        };
        println!("{label}\t{op}\t{arg}\t{val}");
    }
    0
}

pub fn _unused(_: &Path) {}

use std::process::Command;
fn unhex2(s: &str) -> Vec<u8> {
    let b = s.as_bytes(); let mut o = Vec::new(); let mut i = 0;
    while i + 1 < b.len() { if let (Some(h), Some(l)) = ((b[i] as char).to_digit(16), (b[i+1] as char).to_digit(16)) { o.push((h*16+l) as u8); } i += 2; }
    o
}
fn compile_run(tmp: &str, label: &str, prog: &str) -> Option<Vec<u8>> {
    let cob = Path::new(tmp).join("p.cob");
    let exe = Path::new(tmp).join("p");
    let _ = std::fs::write(&cob, prog);
    match Command::new("cobc").args(["-free", "-x", "-o"]).arg(&exe).arg(&cob).output() {
        Ok(o) if o.status.success() => {}
        Ok(o) => { eprintln!("compile {label} failed: {}", String::from_utf8_lossy(&o.stderr).chars().take(200).collect::<String>()); return None; }
        Err(_) => return None,
    }
    Some(Command::new(&exe).output().map(|x| x.stdout).unwrap_or_default())
}

/// INITIALIZE: gen REC+REDEFINES, MOVE ALL "~", INITIALIZE, DISPLAY POST[REDF] -> reclen bytes hex.
pub fn initialize(cases: &str, tmp: &str) -> i32 {
    for line in read(cases).lines() {
        let c: Vec<&str> = line.split('\t').collect();
        if c.len() < 3 { continue; }
        let (label, reclen, lines_) = (c[0], c[1], c[2]);
        let decls = lines_.split('|').map(|d| format!("       {d}")).collect::<Vec<_>>().join("\n");
        let prog = format!("       >>SOURCE FORMAT FREE\nIDENTIFICATION DIVISION.\nPROGRAM-ID. INIT.\nDATA DIVISION.\nWORKING-STORAGE SECTION.\n01 REC.\n{decls}\n01 REDF REDEFINES REC PIC X({reclen}).\nPROCEDURE DIVISION.\n    MOVE ALL \"~\" TO REDF.\n    INITIALIZE REC.\n    DISPLAY \"POST[\" REDF \"]\".\n    STOP RUN.\n");
        let o = match compile_run(tmp, label, &prog) { Some(o) => o, None => continue };
        let n: usize = reclen.parse().unwrap_or(0);
        let posthex = match o.windows(5).position(|w| w == b"POST[") { Some(m) => { let s = m + 5; hexd(&o[s..(s + n).min(o.len())]) } None => String::new() };
        println!("{label}\t{reclen}\t{lines_}\t{posthex}");
    }
    0
}

/// INSPECT TALLYING/REPLACING/CONVERTING: gen per case, parse OUT[ -> cap bytes hex.
pub fn inspect(cases: &str, tmp: &str) -> i32 {
    let reg = |spec: &str| -> String {
        if let Some(s) = spec.strip_prefix("before:") { format!(" BEFORE INITIAL \"{s}\"") }
        else if let Some(s) = spec.strip_prefix("after:") { format!(" AFTER INITIAL \"{s}\"") }
        else { String::new() }
    };
    for line in read(cases).lines() {
        let c: Vec<&str> = line.split('\t').collect();
        if c.len() < 7 { continue; }
        let (label, op, target, mode, a1, a2, rspec) = (c[0], c[1], c[2], c[3], c[4], c[5], c[6]);
        let n = target.len();
        let (stmt, disp) = if op == "TALLY" {
            let m = match mode { "all" => format!("ALL \"{a1}\""), "leading" => format!("LEADING \"{a1}\""), _ => "CHARACTERS".to_string() };
            (format!("INSPECT T TALLYING C FOR {m}{}", reg(rspec)), "DISPLAY \"OUT[\" CX \"]\"")
        } else if op == "REPL" {
            let m = match mode { "all" => format!("ALL \"{a1}\" BY \"{a2}\""), "leading" => format!("LEADING \"{a1}\" BY \"{a2}\""), _ => format!("FIRST \"{a1}\" BY \"{a2}\"") };
            (format!("INSPECT T REPLACING {m}{}", reg(rspec)), "DISPLAY \"OUT[\" T \"]\"")
        } else {
            (format!("INSPECT T CONVERTING \"{a1}\" TO \"{a2}\"{}", reg(rspec)), "DISPLAY \"OUT[\" T \"]\"")
        };
        let prog = format!("       >>SOURCE FORMAT FREE\nIDENTIFICATION DIVISION.\nPROGRAM-ID. INSP.\nDATA DIVISION.\nWORKING-STORAGE SECTION.\n01 T PIC X({n}).\n01 C PIC 9(3).\n01 CX REDEFINES C PIC X(3).\nPROCEDURE DIVISION.\n    MOVE \"{target}\" TO T. MOVE 0 TO C.\n    {stmt}.\n    {disp}.\n    STOP RUN.\n");
        let o = match compile_run(tmp, label, &prog) { Some(o) => o, None => continue };
        let cap = if op == "TALLY" { 3 } else { n };
        let oraclehex = match o.windows(4).position(|w| w == b"OUT[") { Some(m) => { let s = m + 4; hexd(&o[s..(s + cap).min(o.len())]) } None => String::new() };
        println!("{label}\t{op}\t{target}\t{mode}\t{a1}\t{a2}\t{rspec}\t{oraclehex}");
    }
    0
}

/// sequential READ: write input file, run the pre-compiled org reader, parse REC[..][..]/END[..] events.
pub fn seqfile(cases: &str, tmp: &str) -> i32 {
    for line in read(cases).lines() {
        let c: Vec<&str> = line.split('\t').collect();
        if c.len() < 4 { continue; }
        let (label, org, reclen_s, filehex) = (c[0], c[1], c[2], c[3]);
        let reclen: usize = reclen_s.parse().unwrap_or(0);
        let fp = Path::new(tmp).join("in.dat");
        let _ = std::fs::write(&fp, unhex2(filehex));
        let prog = Path::new(tmp).join(if org == "RECORD" { "rec" } else { "lin" });
        let out = Command::new(&prog).env("DDIN", &fp).output().map(|x| x.stdout).unwrap_or_default();
        let mut events = Vec::new();
        let mut i = 0;
        while i < out.len() {
            if out[i..].starts_with(b"REC[") {
                let rec = &out[i + 4..(i + 4 + reclen).min(out.len())];
                let j = i + 4 + reclen;
                let st = &out[(j + 2).min(out.len())..(j + 4).min(out.len())];
                events.push(format!("R:{}:{}", hexd(rec), String::from_utf8_lossy(st)));
                i = j + 5;
            } else if out[i..].starts_with(b"END[") {
                let st = &out[(i + 4).min(out.len())..(i + 6).min(out.len())];
                events.push(format!("E:{}", String::from_utf8_lossy(st)));
                i += 7;
            } else { i += 1; }
        }
        println!("{label}\t{org}\t{reclen}\t{filehex}\t{}", events.join(";"));
    }
    0
}

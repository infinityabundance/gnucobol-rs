//! Rust mirror for the CBL_* system-routine sweep (`GNURUST.FILEIO.SYS.1`). Runs the same fixed
//! sequence of file/dir operations as the oracle program (on its own temp directory) and compares the
//! resulting status sequence to [`gnucobol_rs::fileio`]'s `cob_sys_*` routines. PASS=n FAIL=n.
use gnucobol_rs::fileio::{
    cob_sys_change_dir, cob_sys_close_file, cob_sys_create_dir, cob_sys_create_file, cob_sys_delete_dir,
    cob_sys_delete_file, cob_sys_get_current_dir, cob_sys_read_file, cob_sys_write_file,
};
use std::io::BufRead;

// CBL handle round-trip: create a file, write "HELLO", read it back; emit "<statuses>:<readhex>"
fn handle_roundtrip() -> String {
    let f = std::env::temp_dir().join("gnucobol_rs_handle_sweep_mirror.dat");
    let _ = std::fs::remove_file(&f);
    let fb = f.to_str().unwrap().as_bytes();
    let (cst, h) = cob_sys_create_file(fb, 3);
    let wst = cob_sys_write_file(h, 0, 5, b"HELLO");
    let mut buf = vec![0u8; 5];
    let (rst, _) = cob_sys_read_file(h, 0, 5, 0, &mut buf);
    let _ = cob_sys_close_file(h);
    let _ = std::fs::remove_file(&f);
    format!("{cst},{wst},{rst}:{}", buf.iter().map(|b| format!("{b:02x}")).collect::<String>())
}

fn my_sequence() -> String {
    let base = std::env::temp_dir().join("gnucobol_rs_sys_sweep_mirror");
    let _ = std::fs::remove_dir_all(&base);
    let b = base.to_str().unwrap().as_bytes();
    let missing = base.join("nofile").to_string_lossy().into_owned();
    let mut s: Vec<String> = Vec::new();
    s.push(cob_sys_create_dir(b).to_string()); // 0
    s.push(cob_sys_create_dir(b).to_string()); // 128 (exists)
    s.push(cob_sys_delete_file(missing.as_bytes()).to_string()); // 128 (missing)
    s.push(cob_sys_delete_dir(b).to_string()); // 0
    s.push(cob_sys_delete_dir(b).to_string()); // 128 (gone)
    s.push(cob_sys_change_dir(b"/gnucobol_rs_no_such_dir_zz").to_string()); // 128
    s.push(cob_sys_get_current_dir(0, 4096).0.to_string()); // 0
    s.join(",")
}

fn main() {
    let (mut pass, mut fail) = (0u32, 0u32);
    for line in std::io::stdin().lock().lines().map_while(Result::ok) {
        let Some((tag, oracle)) = line.split_once('=') else { continue };
        let mine = match tag {
            "statuses" => my_sequence(),
            "handle" => handle_roundtrip(),
            _ => continue,
        };
        let _ = &mine;
        if mine == oracle.trim() {
            pass += 1;
        } else {
            println!("{tag} FAIL mine={mine} oracle={}", oracle.trim());
            fail += 1;
        }
    }
    println!("PASS={pass} FAIL={fail}");
}

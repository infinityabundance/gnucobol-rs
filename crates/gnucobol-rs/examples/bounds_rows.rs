//! Oracle mirror for the runtime bounds-check messages (`GNURUST.COMMON.BOUNDCHECK.1`). Reads the
//! `error=<msg>` / `note=<hint>` lines a GnuCOBOL `-debug` program prints on a subscript / ref-mod / ODO
//! out-of-bounds, and checks the native `common::cob_check_*` reproduction is byte-identical. The same
//! lines are produced by both admitted oracles (3.1.2 + 3.2), so this is a version-stable claim. The
//! `case=<id>` line selects which native check to evaluate. Prints `PASS=n FAIL=n`.

use gnucobol_rs::common::{cob_check_odo, cob_check_ref_mod, cob_check_subscript};
use std::io::Read;

fn main() {
    let mut input = String::new();
    std::io::stdin().read_to_string(&mut input).unwrap();
    let mut case = String::new();
    let mut o_err = String::new();
    let mut o_note: Option<String> = None;
    for line in input.lines() {
        let l = line.trim();
        if let Some(v) = l.strip_prefix("case=") {
            case = v.trim().to_string();
        } else if let Some(v) = l.strip_prefix("error=") {
            o_err = v.trim().to_string();
        } else if let Some(v) = l.strip_prefix("note=") {
            o_note = Some(v.trim().to_string());
        }
    }

    // The native reproduction for the fixed case the sweep runs (E PIC X OCCURS 3, subscript 5; etc.).
    let (msg, hint): (Vec<u8>, Option<Vec<u8>>) = match case.as_str() {
        "subscript" => {
            let v = cob_check_subscript(5, 3, "E", false, false).unwrap();
            (v.message, v.hint)
        }
        "refmod" => {
            // F PIC X(5), reference F(2:9) -> length out of bounds.
            let v = &cob_check_ref_mod(2, 9, 5, "F")[0];
            (v.message.clone(), None)
        }
        "odo" => {
            // OCCURS 1 TO 5 DEPENDING ON N, with N = 7. The hint names the OCCURS element (E), the message
            // names the DEPENDING ON variable (N).
            let v = cob_check_odo(7, 1, 5, "E", "N").unwrap();
            (v.message, v.hint)
        }
        other => {
            println!("unknown case {other}");
            println!("PASS=0 FAIL=1");
            std::process::exit(1);
        }
    };

    let mut pass = 0;
    let mut fail = 0;
    if msg == o_err.as_bytes() {
        pass += 1;
    } else {
        fail += 1;
        println!("error FAIL: mine={:?} oracle={o_err:?}", String::from_utf8_lossy(&msg));
    }
    match (&hint, &o_note) {
        (Some(h), Some(n)) if h == n.as_bytes() => pass += 1,
        (None, None) => pass += 1,
        _ => {
            fail += 1;
            println!("note FAIL: mine={:?} oracle={o_note:?}", hint.as_ref().map(|h| String::from_utf8_lossy(h)));
        }
    }
    println!("PASS={pass} FAIL={fail}");
    if fail > 0 {
        std::process::exit(1);
    }
}

//! Oracle mirror for the CBL_ logic/bit builtins (`GNURUST.COMMON.CBL.1`). The sweep runs a COBOL program
//! that `CALL`s each `CBL_*` routine on fixed inputs and DISPLAYs the 1-byte result after an `op:` marker;
//! this mirror computes the same op with the native `common::cob_sys_*` and checks byte-identity. The same
//! results come from both admitted oracles (3.1.2 + 3.2). Prints `PASS=n FAIL=n`.

use gnucobol_rs::common::*;
use std::io::Read;

fn main() {
    let mut raw = Vec::new();
    std::io::stdin().read_to_end(&mut raw).unwrap();

    // Fixed inputs the sweep uses: A = 0xCC, B = 0xAA (1 byte), LEN = 1.
    let a = 0xccu8;
    let b = 0xaau8;

    // (marker, native result byte)
    let one = |f: &dyn Fn(&[u8], &mut [u8])| {
        let mut d = [b];
        f(&[a], &mut d);
        d[0]
    };
    let mut expect: Vec<(&str, u8)> = vec![
        ("and", one(&|s, d| { cob_sys_and(s, d, 1); })),
        ("or", one(&|s, d| { cob_sys_or(s, d, 1); })),
        ("xor", one(&|s, d| { cob_sys_xor(s, d, 1); })),
        ("nor", one(&|s, d| { cob_sys_nor(s, d, 1); })),
        ("imp", one(&|s, d| { cob_sys_imp(s, d, 1); })),
        ("nimp", one(&|s, d| { cob_sys_nimp(s, d, 1); })),
        ("eq", one(&|s, d| { cob_sys_eq(s, d, 1); })),
    ];
    // NOT (single buffer, on B): ~0xAA = 0x55
    {
        let mut d = [b];
        cob_sys_not(&mut d, 1);
        expect.push(("not", d[0]));
    }
    // TOUPPER / TOLOWER on a letter.
    {
        let mut d = *b"a";
        cob_sys_toupper(&mut d, 1);
        expect.push(("upper", d[0]));
        let mut e = *b"Z";
        cob_sys_tolower(&mut e, 1);
        expect.push(("lower", e[0]));
    }

    // Parse the oracle stream: each line "op:<byte>".
    let lines: Vec<&[u8]> = raw.split(|&c| c == b'\n').collect();
    let mut got: std::collections::HashMap<String, u8> = std::collections::HashMap::new();
    for l in lines {
        if let Some(p) = l.iter().position(|&c| c == b':') {
            let key = String::from_utf8_lossy(&l[..p]).trim().to_string();
            if p + 1 < l.len() {
                got.insert(key, l[p + 1]);
            }
        }
    }

    let (mut pass, mut fail) = (0u32, 0u32);
    for (k, v) in &expect {
        match got.get(*k) {
            Some(&ob) if ob == *v => pass += 1,
            other => {
                fail += 1;
                println!("FAIL {k}: native={v:#04x} oracle={other:?}");
            }
        }
    }
    println!("PASS={pass} FAIL={fail}");
    if fail > 0 || pass == 0 {
        std::process::exit(1);
    }
}

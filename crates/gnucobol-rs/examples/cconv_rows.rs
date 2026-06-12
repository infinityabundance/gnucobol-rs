//! Rust evaluator for the cconv.c differential. Mirrors `cconv_harness.c` exactly: `cob_toupper`/
//! `cob_tolower` over all 256 bytes, `cob_field_to_string` over the same fixed grid, and
//! `cob_load_collation` over the `.ttbl` paths passed as argv. Prints byte-identical lines so the two
//! streams diff. The static C helpers (hex/skip_blanks) are exercised transitively via load_collation.

use gnucobol_rs::cconv::{
    cob_field_to_string, cob_load_collation, cob_tolower, cob_toupper, CobCase, FieldRef,
};

fn put_hex(tag: &str, b: &[u8]) {
    let mut s = String::from(tag);
    for x in b {
        s.push_str(&format!("{x:02x}"));
    }
    println!("{s}");
}

fn f2s(label: &str, data: &[u8], size: usize, cm: CobCase) {
    let f = FieldRef { size, data: Some(data) };
    let mut out = [0u8; 64];
    let r = cob_field_to_string(Some(&f), &mut out, cm);
    let mut s = format!("F2S {label} {r} ");
    for &b in out.iter() {
        if b == 0 {
            break;
        }
        s.push_str(&format!("{b:02x}"));
    }
    println!("{s}");
}

fn main() {
    let mut up = [0u8; 256];
    let mut lo = [0u8; 256];
    for c in 0u16..256 {
        up[c as usize] = cob_toupper(c as u8);
        lo[c as usize] = cob_tolower(c as u8);
    }
    put_hex("TOUPPER ", &up);
    put_hex("TOLOWER ", &lo);

    f2s("none8", b"HeLLo   ", 8, CobCase::None);
    f2s("low8", b"HeLLo   ", 8, CobCase::Lower);
    f2s("up8", b"HeLLo   ", 8, CobCase::Upper);
    f2s("lowloc", b"HeLLo   ", 8, CobCase::LowerLocale);
    f2s("uploc", b"HeLLo   ", 8, CobCase::UpperLocale);
    f2s("blank", b"    ", 4, CobCase::None);
    f2s("trail0", b"AB\0\0", 4, CobCase::None);
    f2s("full", b"ABCDE", 5, CobCase::Upper);
    f2s("mixed", b"Ab9$X", 5, CobCase::Lower);
    f2s("one", b"Q", 1, CobCase::Lower);

    for path in std::env::args().skip(1) {
        let mut e2a = [0u8; 256];
        let mut a2e = [0u8; 256];
        let r = cob_load_collation(&path, Some(&mut e2a), Some(&mut a2e));
        let base = path.rsplit('/').next().unwrap_or(&path);
        println!("COLL {base} {r}");
        put_hex(&format!("E2A {base} "), &e2a);
        put_hex(&format!("A2E {base} "), &a2e);
    }
}

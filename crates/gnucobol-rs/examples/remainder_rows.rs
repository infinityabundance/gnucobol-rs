//! Rust mirror for the DIVIDE...REMAINDER sweep (`GNURUST.REMAINDER.1`). Reads the gen_remainder TSV + the
//! oracle quotient AND remainder bytes, builds operands via the sealed cob_move, runs cob_divide_remainder,
//! and compares BOTH receivers. PASS=n FAIL=n.
use gnucobol_rs::arith::cob_divide_remainder;
use gnucobol_rs::{
    build_field, cob_move, FieldAttr, Usage, COB_FLAG_HAVE_SIGN, COB_TYPE_NUMERIC_DISPLAY,
};
use std::io::BufRead;

fn usage(s: &str) -> Usage {
    if s == "COMP-3" { Usage::Comp3 } else { Usage::Display }
}
fn enc(pic: &str, u: Usage, value: &str) -> (Vec<u8>, FieldAttr) {
    let pf = build_field(pic, u, false, false).unwrap();
    let neg = value.starts_with('-');
    let t = value.trim_start_matches(['-', '+']);
    let (ip, fp) = t.split_once('.').unwrap_or((t, ""));
    let mut d: Vec<u8> = ip.bytes().chain(fp.bytes()).map(|b| b - b'0').collect();
    let (have, scale) = (fp.len() as i16, pf.attr.scale);
    if scale > have { d.resize(d.len() + (scale - have) as usize, 0); }
    while d.len() < pf.attr.digits as usize { d.insert(0, 0); }
    let extra = d.len().saturating_sub(pf.attr.digits as usize);
    d.drain(0..extra);
    let mut src: Vec<u8> = d.iter().map(|x| b'0' + x).collect();
    if neg {
        if let Some(l) = src.last_mut() { *l |= 0x40; }
    }
    let sattr = FieldAttr { field_type: COB_TYPE_NUMERIC_DISPLAY, digits: pf.attr.digits, scale: pf.attr.scale, flags: COB_FLAG_HAVE_SIGN };
    let mut out = vec![0u8; pf.size];
    cob_move(&src, &sattr, &mut out, &pf.attr).unwrap();
    (out, pf.attr)
}
fn unhex(s: &str) -> Vec<u8> {
    (0..s.len() / 2).map(|k| u8::from_str_radix(&s[k * 2..k * 2 + 2], 16).unwrap_or(0)).collect()
}
fn hex(b: &[u8]) -> String { b.iter().map(|x| format!("{x:02x}")).collect() }

fn main() {
    let (mut pass, mut fail) = (0u32, 0u32);
    for line in std::io::stdin().lock().lines().map_while(Result::ok) {
        let f: Vec<&str> = line.split('\t').collect();
        if f.len() < 16 { continue; }
        let (label, form, a, a_pic, a_use, b, b_pic, b_use, c_pic, c_use, d_pic, d_use, qhex, rhex) =
            (f[0], f[1], f[2], f[3], f[4], f[5], f[6], f[7], f[8], f[9], f[11], f[12], f[14], f[15]);
        let (ab, aattr) = enc(a_pic, usage(a_use), a);
        let (bb, battr) = enc(b_pic, usage(b_use), b);
        let cattr = build_field(c_pic, usage(c_use), false, false).unwrap().attr;
        let dattr = build_field(d_pic, usage(d_use), false, false).unwrap().attr;
        // form BY: q := a/b ; form INTO: q := b/a
        let (lhs, la, rhs, ra) = if form == "BY" { (&ab, &aattr, &bb, &battr) } else { (&bb, &battr, &ab, &aattr) };
        let mine = cob_divide_remainder(lhs, la, rhs, ra, &cattr, &dattr).ok();
        let (qo, ro) = (unhex(qhex), unhex(rhex));
        let ok = matches!(&mine, Some((q, r)) if q.as_slice() == qo.as_slice() && r.as_slice() == ro.as_slice());
        if ok {
            pass += 1;
        } else {
            println!("{label} FAIL form={form} {a}/{b} -> q={c_pic}/{c_use} r={d_pic}/{d_use} mine={:?} oracle=q:{qhex} r:{rhex}",
                     mine.map(|(q, r)| format!("q:{} r:{}", hex(&q), hex(&r))));
            fail += 1;
        }
    }
    println!("PASS={pass} FAIL={fail}");
}

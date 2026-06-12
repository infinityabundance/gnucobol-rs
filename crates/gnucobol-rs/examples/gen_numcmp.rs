//! Generator for the numeric comparison sweep (GNURUST.NUMCMP.1). Row:
//! label t1 d1 sc1 fl1 sz1 hex1 t2 d2 sc2 fl2 sz2 hex2
const DISPLAY: u32 = 16; const PACKED: u32 = 18; const SIGN: u32 = 1;
fn hex(b: &[u8]) -> String { b.iter().map(|x| format!("{x:02x}")).collect() }
fn enc(ut: u32, digits: &[u8], neg: bool) -> (Vec<u8>, usize) {
    if ut == DISPLAY {
        let mut o: Vec<u8> = digits.iter().map(|d| b'0' + d).collect();
        if neg { if let Some(l) = o.last_mut() { *l |= 0x40; } }
        (o, digits.len())
    } else {
        let mut n: Vec<u8> = digits.to_vec();
        n.push(if neg { 0x0D } else { 0x0C });
        if n.len() % 2 == 1 { n.insert(0, 0); }
        (n.chunks(2).map(|c| (c[0] << 4) | c[1]).collect(), digits.len() / 2 + 1)
    }
}
struct L(u64);
impl L { fn n(&mut self) -> u64 { self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(1); self.0 >> 16 } }
fn main() {
    let mut r = L(0xC0FFEE);
    let mut id = 0u64;
    let vals: &[(&[u8], i16)] = &[
        (&[1,2,3], 0), (&[1,2,3], 1), (&[1,2,3], 2), (&[0,1,2,3,0], 2),
        (&[9,9,9], 0), (&[0,0,0], 0), (&[5], 0), (&[1,0,0,0,0,0], 3),
    ];
    for &t1 in &[DISPLAY, PACKED] {
        for &t2 in &[DISPLAY, PACKED] {
            for &(d1, s1) in vals {
                for &(d2, s2) in vals {
                    for n1 in [false, true] {
                        for n2 in [false, true] {
                            let _ = r.n();
                            let (b1, sz1) = enc(t1, d1, n1);
                            let (b2, sz2) = enc(t2, d2, n2);
                            println!("c{id} {t1} {} {s1} {SIGN} {sz1} {} {t2} {} {s2} {SIGN} {sz2} {}",
                                d1.len(), hex(&b1), d2.len(), hex(&b2));
                            id += 1;
                        }
                    }
                }
            }
        }
    }
}

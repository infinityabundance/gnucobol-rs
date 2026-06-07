//! Deterministic generator of VALUE-court sweep specs (`GNURUST.8`). Each line is one record case:
//! `LABEL|field|field|...` where a field is `LEVEL:NAME:PIC:USAGE:VALUE`:
//!   USAGE ∈ {G(group,no pic), D(display), C(comp-3)};
//!   VALUE ∈ "" (unvalued) | N<lit> (numeric) | A<lit> (alnum) | Z (ZERO) | S (SPACE).
//! Both the libcob program-oracle (builds a .cob) and the Rust mirror parse this. Test infra.

struct Lcg(u64);
impl Lcg {
    fn step(&mut self) -> u64 {
        self.0 = self
            .0
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        self.0 >> 16
    }
    fn below(&mut self, n: u64) -> u64 {
        self.step() % n.max(1)
    }
}

fn num_value(rng: &mut Lcg, digits: usize, scale: usize, signed: bool) -> String {
    let intd = digits - scale;
    let mut s = String::new();
    if signed && rng.below(2) == 0 {
        s.push('-');
    }
    let id = (rng.below(intd as u64) + 1) as usize; // 1..intd integer digits (no leading zero pad needed)
    for _ in 0..id {
        s.push((b'0' + rng.below(10) as u8) as char);
    }
    if scale > 0 {
        s.push('.');
        for _ in 0..scale {
            s.push((b'0' + rng.below(10) as u8) as char);
        }
    }
    s
}

fn main() {
    let mut rng = Lcg(0x7A1u64.wrapping_add(0x9E37_79B9_7F4A_7C15));
    let mut id = 0u64;
    // Field "shapes": (pic, usage-char, signed, digits, scale, is_alpha)
    let shapes: &[(&str, char, bool, usize, usize)] = &[
        ("9(3)", 'D', false, 3, 0),
        ("S9(3)", 'D', true, 3, 0),
        ("9(4)V99", 'D', false, 6, 2),
        ("S9(4)V99", 'D', true, 6, 2),
        ("S9(3)V99", 'C', true, 5, 2),
        ("9(5)", 'C', false, 5, 0),
        ("S9(7)V9(3)", 'C', true, 10, 3),
    ];
    let alnums: &[&str] = &["X(4)", "X(1)", "X(8)"];

    for shape in shapes {
        for valued in [true, false] {
            for _rep in 0..28 {
                let (pic, usage, signed, digits, scale) = *shape;
                let mut fields = vec!["1:REC::G:".to_string()];
                // a leading alnum, the numeric under test, a trailing alnum (mixed record).
                let alead = alnums[rng.below(alnums.len() as u64) as usize];
                let avalued = rng.below(2) == 0;
                fields.push(format!(
                    "5:LEAD:{alead}:D:{}",
                    if avalued {
                        format!("AAB{}", rng.below(9))
                    } else {
                        String::new()
                    }
                ));

                let v = if valued {
                    if rng.below(5) == 0 {
                        "Z".to_string()
                    } else {
                        format!("N{}", num_value(&mut rng, digits, scale, signed))
                    }
                } else {
                    String::new()
                };
                fields.push(format!("5:NUM:{pic}:{usage}:{v}"));

                let atrail = alnums[rng.below(alnums.len() as u64) as usize];
                fields.push(format!(
                    "5:TAIL:{atrail}:D:{}",
                    if rng.below(2) == 0 {
                        "AHELLO".to_string()
                    } else {
                        String::new()
                    }
                ));

                println!("x{id}|{}", fields.join("|"));
                id += 1;
            }
        }
    }
}

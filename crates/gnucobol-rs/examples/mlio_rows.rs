//! Rust mirror for the XML/JSON GENERATE sweep (`GNURUST.MLIO.GENERATE.1`). The oracle runs `XML GENERATE`
//! and `JSON GENERATE` over a fixed record and prints `xml=<bytes>` / `json=<bytes>`; this mirror builds the
//! same `cob_ml_tree` and serializes it natively ([`gnucobol_rs::mlio::cob_xml_generate_new`] /
//! `cob_json_generate_new`), asserting byte-identity. PASS=n FAIL=n.
use gnucobol_rs::mlio::{cob_json_generate_new, cob_xml_generate_new, MlContent, MlTree};
use std::io::BufRead;

fn num(name: &str, digits: &[u8], scale: usize, neg: bool) -> MlTree {
    MlTree { name: name.into(), attrs: vec![], content: MlContent::Numeric { digits: digits.to_vec(), scale, negative: neg }, is_suppressed: false, children: vec![] }
}
fn alx(name: &str, v: &[u8]) -> MlTree {
    MlTree { name: name.into(), attrs: vec![], content: MlContent::Alnum(v.to_vec()), is_suppressed: false, children: vec![] }
}
fn grp(name: &str, children: Vec<MlTree>) -> MlTree {
    MlTree { name: name.into(), attrs: vec![], content: MlContent::None, is_suppressed: false, children }
}

/// The `cob_ml_tree` the sweep's COBOL `01 G` record produces (must stay in lock-step with ml_generate_sweep.sh).
fn tree() -> MlTree {
    grp(
        "G",
        vec![
            num("NEG", b"042", 0, true),
            num("DEC", b"01250", 2, false),
            alx("SPC", b"a<b&c"),
            grp("GRP", vec![alx("X", b"hi"), num("Y", b"7", 0, false)]),
        ],
    )
}

fn main() {
    let (mut pass, mut fail) = (0u32, 0u32);
    let xml = cob_xml_generate_new(&tree());
    let json = cob_json_generate_new(&tree());
    for line in std::io::stdin().lock().lines().map_while(Result::ok) {
        let Some((tag, oracle)) = line.split_once('=') else { continue };
        let oracle = oracle.trim().as_bytes();
        let mine: &[u8] = match tag {
            "xml" => &xml,
            "json" => &json,
            _ => continue,
        };
        if mine == oracle {
            pass += 1;
        } else {
            println!("{tag} FAIL\n  mine  ={}\n  oracle={}", String::from_utf8_lossy(mine), String::from_utf8_lossy(oracle));
            fail += 1;
        }
    }
    println!("PASS={pass} FAIL={fail}");
}

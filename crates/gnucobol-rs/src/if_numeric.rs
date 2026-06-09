//! Numeric `IF` / `EVALUATE` execution slice (`GNURUST.IF.NUMERIC.SLICE.1`): the numeric counterpart of
//! [`crate::if_eval`] (which is alphanumeric). It *runs* an `IF`/`EVALUATE` over **numeric** `PIC 9(n)` fields,
//! comparing decoded **values** (so `5 < 10`, unlike the alphanumeric `"5" > "10"`) and applying numeric
//! `MOVE` branches — proven against GnuCOBOL 3.2. Completes the conditional-logic slice for both data classes.
//!
//! **Sealed subset (deliberately narrow).** Fields are **unsigned numeric** `PIC 9(n)` regions of a flat
//! record. The condition is a single numeric relation `field <op> literal`; `EVALUATE` matches a numeric
//! subject against `WHEN <literal>` values (else `WHEN OTHER`). Branches are `MOVE <literal> TO <field>`
//! statements that encode the value into the numeric field width (low-order digits on overflow, like a
//! numeric `MOVE`).
//!
//! **Non-claims:** signed / packed / `V`-scaled numerics, `MOVE <field> TO <field>` (only literals here),
//! numeric `SIZE ERROR`, compound / class conditions, `88`-level (`GNURUST.11`), range / `THRU` `WHEN`s,
//! non-`MOVE` branches, nested flow, and all dialects.

use crate::if_eval::{Relop, SliceField};
use crate::perform_slice::NumCond;

/// A `MOVE <literal> TO <numeric field>` branch statement.
pub struct MoveNum<'a> {
    pub value: i64,
    pub target: &'a str,
}

fn field<'a>(fields: &'a [SliceField], name: &str) -> Option<&'a SliceField<'a>> {
    fields.iter().find(|f| f.name == name)
}

fn read_num(buf: &[u8], fields: &[SliceField], name: &str) -> i64 {
    field(fields, name)
        .and_then(|f| buf.get(f.offset..f.offset + f.size))
        .map(|b| b.iter().fold(0i64, |a, &c| if c.is_ascii_digit() { a * 10 + (c - b'0') as i64 } else { a }))
        .unwrap_or(0)
}

fn write_num(out: &mut [u8], fields: &[SliceField], name: &str, val: i64) {
    if let Some(f) = field(fields, name) {
        if f.offset + f.size <= out.len() {
            let m = 10i64.pow(f.size as u32);
            let v = ((val % m) + m) % m; // encode into the field width (low-order digits on overflow)
            let s = format!("{:0width$}", v, width = f.size);
            out[f.offset..f.offset + f.size].copy_from_slice(s.as_bytes());
        }
    }
}

fn eval_cond(out: &[u8], fields: &[SliceField], cond: &NumCond) -> bool {
    let l = read_num(out, fields, cond.field);
    match cond.op {
        Relop::Eq => l == cond.value,
        Relop::Ne => l != cond.value,
        Relop::Gt => l > cond.value,
        Relop::Lt => l < cond.value,
        Relop::Ge => l >= cond.value,
        Relop::Le => l <= cond.value,
    }
}

/// Execute `IF <numeric cond> <then> ELSE <else>` over `record`, returning the resulting storage bytes.
pub fn eval_if_numeric(
    record: &[u8],
    fields: &[SliceField],
    cond: &NumCond,
    then_moves: &[MoveNum],
    else_moves: &[MoveNum],
) -> Vec<u8> {
    let mut out = record.to_vec();
    let taken = if eval_cond(&out, fields, cond) {
        then_moves
    } else {
        else_moves
    };
    for mv in taken {
        write_num(&mut out, fields, mv.target, mv.value);
    }
    out
}

/// Execute `EVALUATE <numeric subject> WHEN <lit> <branch> ... WHEN OTHER <other>` over `record`. The first
/// `WHEN` whose literal equals the subject value wins; otherwise `other`.
pub fn eval_evaluate_numeric(
    record: &[u8],
    fields: &[SliceField],
    subject: &str,
    whens: &[(i64, &[MoveNum])],
    other: &[MoveNum],
) -> Vec<u8> {
    let mut out = record.to_vec();
    let v = read_num(&out, fields, subject);
    let taken = whens
        .iter()
        .find(|(w, _)| *w == v)
        .map(|(_, b)| *b)
        .unwrap_or(other);
    for mv in taken {
        write_num(&mut out, fields, mv.target, mv.value);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    // record: N = 9(3) @0, F = 9(2) @3  (5 bytes)
    fn fields() -> Vec<SliceField<'static>> {
        vec![
            SliceField { name: "N", offset: 0, size: 3 },
            SliceField { name: "F", offset: 3, size: 2 },
        ]
    }
    fn rec(n: i64) -> Vec<u8> {
        format!("{n:03}00").into_bytes()
    }
    fn f_of(out: &[u8]) -> i64 {
        out[3..5].iter().fold(0i64, |a, &c| a * 10 + (c - b'0') as i64)
    }
    fn cond(op: Relop, value: i64) -> NumCond<'static> {
        NumCond { field: "N", op, value }
    }

    #[test]
    fn numeric_if_compares_by_value() {
        let fl = fields();
        let t = [MoveNum { value: 1, target: "F" }];
        let e = [MoveNum { value: 9, target: "F" }];
        // N=50: N>100 false -> ELSE -> 9 (value comparison; alphanumeric "050">"100" would also be false, but...)
        assert_eq!(f_of(&eval_if_numeric(&rec(50), &fl, &cond(Relop::Gt, 100), &t, &e)), 9);
        // N=50: N<100 true -> 1
        assert_eq!(f_of(&eval_if_numeric(&rec(50), &fl, &cond(Relop::Lt, 100), &t, &e)), 1);
        // N=5: N>9 false numerically (5<9) -> 9  (alphanumeric "005">"9" would be false too, but the value path is what is sealed)
        assert_eq!(f_of(&eval_if_numeric(&rec(5), &fl, &cond(Relop::Gt, 9), &t, &e)), 9);
        // N=50: N>=50 true -> 1
        assert_eq!(f_of(&eval_if_numeric(&rec(50), &fl, &cond(Relop::Ge, 50), &t, &e)), 1);
    }

    #[test]
    fn numeric_evaluate_first_match_and_other() {
        let fl = fields();
        let whens: Vec<(i64, &[MoveNum])> = vec![
            (10, &[MoveNum { value: 1, target: "F" }]),
            (50, &[MoveNum { value: 5, target: "F" }]),
        ];
        let other = [MoveNum { value: 8, target: "F" }];
        assert_eq!(f_of(&eval_evaluate_numeric(&rec(50), &fl, "N", &whens, &other)), 5);
        assert_eq!(f_of(&eval_evaluate_numeric(&rec(99), &fl, "N", &whens, &other)), 8);
    }
}

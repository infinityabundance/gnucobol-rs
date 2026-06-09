//! `PERFORM` execution slice (`GNURUST.PERFORM.SLICE.1`): the second execution slice — a narrow interpreter
//! that *runs* a `PERFORM` loop over numeric counter fields and returns the resulting storage, proven against
//! GnuCOBOL 3.2. Builds on [`crate::if_eval`] (it reuses the field model and relational operators) and the
//! `PROCEDURE.FLOW.ATLAS` observations, but *executes* the loop rather than observing it.
//!
//! **Sealed subset (deliberately narrow).** Counter fields are **unsigned numeric** `PIC 9(n)` DISPLAY regions
//! of a flat record. The loop body is a sequence of `ADD <literal> TO <field>` ops. Three loop forms, with the
//! oracle-witnessed semantics:
//! - **`PERFORM n TIMES`** — the body runs `n` times (`n <= 0` → zero iterations).
//! - **`PERFORM UNTIL <cond>`** — **test before** each iteration; loop while the condition is false.
//! - **`PERFORM VARYING v FROM a BY b UNTIL <cond>`** — set `v = a`, then test-before: while the condition is
//!   false run the body and add `b` to `v`. The control variable ends **one step past** the limit when the
//!   loop ran (`FROM 2 BY 3 UNTIL I>10` ends `I=11`), or at `a` if the condition was true immediately.
//!
//! The condition is a single numeric relation `field <op> literal`. Field values are read/written as the
//! zoned `9(n)` digits; an increment that would exceed `10^n` wraps (overflow / `SIZE ERROR` is a non-claim).
//!
//! **Non-claims:** signed / packed / binary counters, numeric `SIZE ERROR` on the body, non-`ADD` body
//! statements, compound / class conditions, `PERFORM ... THRU` / out-of-line paragraph performs, `WITH TEST
//! AFTER`, nested `PERFORM`, `GO TO`, and all dialects.

use crate::if_eval::{Relop, SliceField};

/// One body operation: `ADD <amount> TO <field>`.
pub struct AddOp<'a> {
    pub target: &'a str,
    pub amount: i64,
}

/// A single numeric relation `field <op> value`.
pub struct NumCond<'a> {
    pub field: &'a str,
    pub op: Relop,
    pub value: i64,
}

/// The `PERFORM` loop form.
pub enum PerformForm<'a> {
    Times(i64),
    Until(NumCond<'a>),
    Varying {
        var: &'a str,
        from: i64,
        by: i64,
        until: NumCond<'a>,
    },
}

fn field<'a>(fields: &'a [SliceField], name: &str) -> Option<&'a SliceField<'a>> {
    fields.iter().find(|f| f.name == name)
}

fn read_num(record: &[u8], fields: &[SliceField], name: &str) -> i64 {
    field(fields, name)
        .and_then(|f| record.get(f.offset..f.offset + f.size))
        .map(|b| {
            let mut v: i64 = 0;
            for &c in b {
                if c.is_ascii_digit() {
                    v = v * 10 + (c - b'0') as i64;
                }
            }
            v
        })
        .unwrap_or(0)
}

fn write_num(out: &mut [u8], fields: &[SliceField], name: &str, val: i64) {
    if let Some(f) = field(fields, name) {
        if f.offset + f.size <= out.len() {
            let m = 10i64.pow(f.size as u32);
            let v = ((val % m) + m) % m; // wrap to fit the field width (overflow is a non-claim)
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

fn run_body(out: &mut [u8], fields: &[SliceField], body: &[AddOp]) {
    for op in body {
        let v = read_num(out, fields, op.target) + op.amount;
        write_num(out, fields, op.target, v);
    }
}

/// Execute a `PERFORM` loop over `record`, returning the resulting storage bytes.
pub fn eval_perform(record: &[u8], fields: &[SliceField], form: &PerformForm, body: &[AddOp]) -> Vec<u8> {
    let mut out = record.to_vec();
    match form {
        PerformForm::Times(n) => {
            for _ in 0..(*n).max(0) {
                run_body(&mut out, fields, body);
            }
        }
        PerformForm::Until(cond) => {
            while !eval_cond(&out, fields, cond) {
                run_body(&mut out, fields, body);
            }
        }
        PerformForm::Varying { var, from, by, until } => {
            write_num(&mut out, fields, var, *from);
            while !eval_cond(&out, fields, until) {
                run_body(&mut out, fields, body);
                let v = read_num(&out, fields, var) + by;
                write_num(&mut out, fields, var, v);
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    // record: C = 9(3) @0, I = 9(3) @3  (6 bytes)
    fn fields() -> Vec<SliceField<'static>> {
        vec![
            SliceField { name: "C", offset: 0, size: 3 },
            SliceField { name: "I", offset: 3, size: 3 },
        ]
    }
    fn rec(c: i64, i: i64) -> Vec<u8> {
        format!("{c:03}{i:03}").into_bytes()
    }
    fn nums(out: &[u8]) -> (i64, i64) {
        let f = fields();
        (read_num(out, &f, "C"), read_num(out, &f, "I"))
    }
    fn add_c() -> Vec<AddOp<'static>> {
        vec![AddOp { target: "C", amount: 1 }]
    }

    #[test]
    fn perform_times() {
        let f = fields();
        assert_eq!(nums(&eval_perform(&rec(0, 0), &f, &PerformForm::Times(3), &add_c())).0, 3);
        assert_eq!(nums(&eval_perform(&rec(0, 0), &f, &PerformForm::Times(0), &add_c())).0, 0);
    }

    #[test]
    fn perform_until_tests_before() {
        let f = fields();
        // C=0 UNTIL C>=5 -> 5
        assert_eq!(nums(&eval_perform(&rec(0, 0), &f, &PerformForm::Until(NumCond { field: "C", op: Relop::Ge, value: 5 }), &add_c())).0, 5);
        // C=7 already satisfies -> body never runs -> 7
        assert_eq!(nums(&eval_perform(&rec(7, 0), &f, &PerformForm::Until(NumCond { field: "C", op: Relop::Ge, value: 5 }), &add_c())).0, 7);
    }

    #[test]
    fn perform_varying_ends_one_past_limit() {
        let f = fields();
        // VARYING I FROM 1 BY 1 UNTIL I>4 -> body 4x (C=4), I=5
        let out = eval_perform(&rec(0, 0), &f, &PerformForm::Varying { var: "I", from: 1, by: 1, until: NumCond { field: "I", op: Relop::Gt, value: 4 } }, &add_c());
        assert_eq!(nums(&out), (4, 5));
        // FROM 2 BY 3 UNTIL I>10 -> I=2,5,8 body 3x (C=3), I=11
        let out = eval_perform(&rec(0, 0), &f, &PerformForm::Varying { var: "I", from: 2, by: 3, until: NumCond { field: "I", op: Relop::Gt, value: 10 } }, &add_c());
        assert_eq!(nums(&out), (3, 11));
        // FROM 5 BY 1 UNTIL I>2 -> condition true immediately, body never runs, I stays 5
        let out = eval_perform(&rec(0, 0), &f, &PerformForm::Varying { var: "I", from: 5, by: 1, until: NumCond { field: "I", op: Relop::Gt, value: 2 } }, &add_c());
        assert_eq!(nums(&out), (0, 5));
    }
}

#[cfg(kani)]
mod kani_proofs {
    use super::*;
    use crate::if_eval::SliceField;
    // KANIFOR: GNURUST.PERFORM.SLICE.1
    /// A bounded PERFORM n TIMES never changes the record length.
    #[kani::proof]
    #[kani::unwind(6)]
    fn perform_times_preserves_length() {
        let rec: [u8; 6] = kani::any();
        let fields = [SliceField { name: "C", offset: 0, size: 3 }, SliceField { name: "I", offset: 3, size: 3 }];
        let n: i64 = kani::any();
        kani::assume((0..=4).contains(&n));
        let body = [AddOp { target: "C", amount: 1 }];
        assert_eq!(eval_perform(&rec, &fields, &PerformForm::Times(n), &body).len(), 6);
    }
}

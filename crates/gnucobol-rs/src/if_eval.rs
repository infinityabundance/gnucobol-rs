//! `IF` / `EVALUATE` execution slice (`GNURUST.IF.EVALUATE.SLICE.1`): the **first execution slice** — a narrow
//! interpreter that *runs* an `IF`/`EVALUATE` fragment over a record's bytes and returns the resulting storage,
//! proven against GnuCOBOL 3.2. The `PROCEDURE.FLOW.ATLAS` only *observed* control flow; this *executes* a
//! tightly-bounded fragment by composing the sealed alphanumeric compare and MOVE semantics.
//!
//! **Sealed subset (deliberately narrow).** Fields are **alphanumeric** (`PIC X(n)`) regions of a flat record.
//! - **Condition:** a single relation `left <op> right` where each operand is a field or a literal and `op` is
//!   `= / NOT = / > / < / >= / <=`. Comparison is the COBOL alphanumeric rule: pad the shorter operand with
//!   spaces to the longer length, then compare byte-by-byte in the native (ASCII) collating sequence.
//! - **Branch:** zero or more `MOVE <field|literal> TO <field>` statements (alphanumeric MOVE — left-justify,
//!   space-pad / truncate to the target width).
//! - **`EVALUATE` subject:** first `WHEN <literal>` whose value equals the subject wins; otherwise `WHEN OTHER`.
//!
//! **Non-claims:** numeric / packed comparison and numeric `MOVE`, compound conditions (`AND`/`OR`/`NOT`),
//! class conditions (`NUMERIC`/`ALPHABETIC`), `88`-level condition names (those are `GNURUST.11`), non-`MOVE`
//! branch statements, reference modification / subscripts, nested `IF` / `PERFORM` / `GO TO`, range/`THRU`
//! `WHEN`s, and all dialects.

use std::cmp::Ordering;

/// One alphanumeric field: its name and `[offset, offset+size)` region in the record.
pub struct SliceField<'a> {
    pub name: &'a str,
    pub offset: usize,
    pub size: usize,
}

/// A condition/MOVE operand: a named field or an inline literal.
#[derive(Clone, Copy)]
pub enum Operand<'a> {
    Field(&'a str),
    Literal(&'a [u8]),
}

/// A relational operator.
#[derive(Clone, Copy)]
pub enum Relop {
    Eq,
    Ne,
    Gt,
    Lt,
    Ge,
    Le,
}

/// A single relation `left <op> right`.
pub struct Condition<'a> {
    pub left: Operand<'a>,
    pub op: Relop,
    pub right: Operand<'a>,
}

/// A `MOVE <source> TO <target>` statement (alphanumeric).
pub struct MoveStmt<'a> {
    pub source: Operand<'a>,
    pub target: &'a str,
}

fn field<'a>(fields: &'a [SliceField], name: &str) -> Option<&'a SliceField<'a>> {
    fields.iter().find(|f| f.name == name)
}

fn operand_bytes(record: &[u8], fields: &[SliceField], op: &Operand) -> Vec<u8> {
    match *op {
        Operand::Literal(b) => b.to_vec(),
        Operand::Field(name) => field(fields, name)
            .and_then(|f| record.get(f.offset..f.offset + f.size))
            .map(|s| s.to_vec())
            .unwrap_or_default(),
    }
}

/// COBOL alphanumeric comparison: pad the shorter to the longer with spaces, compare byte-by-byte.
fn alnum_cmp(a: &[u8], b: &[u8]) -> Ordering {
    let n = a.len().max(b.len());
    for i in 0..n {
        let av = a.get(i).copied().unwrap_or(b' ');
        let bv = b.get(i).copied().unwrap_or(b' ');
        match av.cmp(&bv) {
            Ordering::Equal => continue,
            o => return o,
        }
    }
    Ordering::Equal
}

/// Evaluate a single relation over the record.
pub fn eval_condition(record: &[u8], fields: &[SliceField], cond: &Condition) -> bool {
    let l = operand_bytes(record, fields, &cond.left);
    let r = operand_bytes(record, fields, &cond.right);
    let ord = alnum_cmp(&l, &r);
    match cond.op {
        Relop::Eq => ord == Ordering::Equal,
        Relop::Ne => ord != Ordering::Equal,
        Relop::Gt => ord == Ordering::Greater,
        Relop::Lt => ord == Ordering::Less,
        Relop::Ge => ord != Ordering::Less,
        Relop::Le => ord != Ordering::Greater,
    }
}

fn apply_move(out: &mut [u8], fields: &[SliceField], mv: &MoveStmt) {
    let src = operand_bytes(out, fields, &mv.source);
    if let Some(f) = field(fields, mv.target) {
        if f.offset + f.size <= out.len() {
            let mut v: Vec<u8> = src.iter().take(f.size).copied().collect();
            v.resize(f.size, b' ');
            out[f.offset..f.offset + f.size].copy_from_slice(&v);
        }
    }
}

/// Execute `IF <cond> <then_branch> ELSE <else_branch>` over `record`, returning the resulting storage bytes.
pub fn eval_if(
    record: &[u8],
    fields: &[SliceField],
    cond: &Condition,
    then_branch: &[MoveStmt],
    else_branch: &[MoveStmt],
) -> Vec<u8> {
    let mut out = record.to_vec();
    let taken = if eval_condition(&out, fields, cond) {
        then_branch
    } else {
        else_branch
    };
    for mv in taken {
        apply_move(&mut out, fields, mv);
    }
    out
}

/// Execute `EVALUATE <subject> WHEN <lit> <branch> ... WHEN OTHER <other>` over `record`. The first `WHEN`
/// whose literal equals the subject (alphanumeric) wins; otherwise `other`.
pub fn eval_evaluate(
    record: &[u8],
    fields: &[SliceField],
    subject: &str,
    whens: &[(&[u8], &[MoveStmt])],
    other: &[MoveStmt],
) -> Vec<u8> {
    let mut out = record.to_vec();
    let subj = operand_bytes(&out, fields, &Operand::Field(subject));
    let taken = whens
        .iter()
        .find(|(lit, _)| alnum_cmp(&subj, lit) == Ordering::Equal)
        .map(|(_, b)| *b)
        .unwrap_or(other);
    for mv in taken {
        apply_move(&mut out, fields, mv);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::Operand::*;
    use super::*;

    // record: A = X(3) @0, T = X(4) @3  (7 bytes)
    fn fields() -> Vec<SliceField<'static>> {
        vec![
            SliceField { name: "A", offset: 0, size: 3 },
            SliceField { name: "T", offset: 3, size: 4 },
        ]
    }
    fn rec(a: &str, t: &str) -> Vec<u8> {
        let mut av = a.as_bytes().to_vec();
        av.resize(3, b' ');
        let mut tv = t.as_bytes().to_vec();
        tv.resize(4, b' ');
        av.extend_from_slice(&tv);
        av
    }
    fn t_of(out: &[u8]) -> String {
        String::from_utf8_lossy(&out[3..7]).into_owned()
    }

    #[test]
    fn if_eq_then_branch() {
        let f = fields();
        let out = eval_if(
            &rec("BBB", "----"),
            &f,
            &Condition { left: Field("A"), op: Relop::Eq, right: Literal(b"BBB") },
            &[MoveStmt { source: Literal(b"YES"), target: "T" }],
            &[MoveStmt { source: Literal(b"NO"), target: "T" }],
        );
        assert_eq!(t_of(&out), "YES "); // left-justified, padded
    }

    #[test]
    fn if_gt_alphanumeric() {
        let f = fields();
        let out = eval_if(
            &rec("BBB", "----"),
            &f,
            &Condition { left: Field("A"), op: Relop::Gt, right: Literal(b"AAA") },
            &[MoveStmt { source: Literal(b"GT"), target: "T" }],
            &[MoveStmt { source: Literal(b"LE"), target: "T" }],
        );
        assert_eq!(t_of(&out), "GT  ");
    }

    #[test]
    fn if_else_moves_field_to_field() {
        let f = fields();
        let out = eval_if(
            &rec("BBB", "----"),
            &f,
            &Condition { left: Field("A"), op: Relop::Lt, right: Literal(b"AAA") },
            &[MoveStmt { source: Literal(b"Y"), target: "T" }],
            &[MoveStmt { source: Field("A"), target: "T" }],
        );
        assert_eq!(t_of(&out), "BBB "); // A (X3) into T (X4) -> "BBB "
    }

    #[test]
    fn evaluate_first_match_and_other() {
        let f = fields();
        let whens: Vec<(&[u8], &[MoveStmt])> = vec![
            (b"A", &[MoveStmt { source: Literal(b"AAA"), target: "T" }]),
            (b"B", &[MoveStmt { source: Literal(b"BEE"), target: "T" }]),
        ];
        let other = [MoveStmt { source: Literal(b"OTH"), target: "T" }];
        assert_eq!(t_of(&eval_evaluate(&rec("B", "----"), &f, "A", &whens, &other)), "BEE ");
        assert_eq!(t_of(&eval_evaluate(&rec("Z", "----"), &f, "A", &whens, &other)), "OTH ");
    }
}

//! Table (`OCCURS`) `PERFORM VARYING` execution slice (`GNURUST.TABLE.PERFORM.SLICE.1`): the core of COBOL
//! table processing — *running* a `PERFORM VARYING I … TABLE(I) …` loop over a subscripted `OCCURS` table and
//! returning the accumulated result, proven against GnuCOBOL 3.2. Deepens the execution slices with
//! **1-based subscript** access, the construct that touches almost every real COBOL program.
//!
//! **Sealed subset (deliberately narrow).** A single-dimension `OCCURS` table of fixed unsigned `9(n)`
//! elements in a flat record. `PERFORM VARYING I FROM <a> BY <b> UNTIL I > <limit>` (test-before; `I` ends one
//! step past the limit) accumulates `TABLE(I)` — **`TABLE(I)` is 1-based**: element `I` lives at
//! `base + (I-1)*elem_size`. An optional per-element filter (`IF TABLE(I) <op> <literal>`) gates the
//! accumulation. The result is the **sum** and **count** of the accumulated elements.
//!
//! **Non-claims:** multi-dimensional / nested `OCCURS`, `OCCURS DEPENDING ON` (variable length), subscript
//! **out-of-bounds** behavior, `INDEXED BY` / `SEARCH` / `SET` index semantics, signed / packed / `V`-scaled
//! elements, numeric `SIZE ERROR` on the accumulator, non-sum per-element bodies, and all dialects.

use crate::if_eval::Relop;

/// A single-dimension `OCCURS` table: where it starts, each element's byte width, and how many elements.
pub struct Table {
    pub base_offset: usize,
    pub elem_size: usize,
    pub occurs: usize,
}

/// The result of a table accumulation loop.
pub struct TableLoopResult {
    pub sum: i64,
    pub count: usize,
    /// the control variable `I` after the loop (one step past the limit when the loop ran).
    pub final_index: i64,
}

/// `TABLE(i)` — the decoded unsigned value of the **1-based** element `i` (`0` if out of `1..=occurs`).
pub fn table_elem(record: &[u8], table: &Table, i: usize) -> i64 {
    if i < 1 || i > table.occurs {
        return 0;
    }
    let off = table.base_offset + (i - 1) * table.elem_size;
    record
        .get(off..off + table.elem_size)
        .map(|b| b.iter().fold(0i64, |a, &c| if c.is_ascii_digit() { a * 10 + (c - b'0') as i64 } else { a }))
        .unwrap_or(0)
}

fn relop_int(a: i64, op: &Relop, b: i64) -> bool {
    match op {
        Relop::Eq => a == b,
        Relop::Ne => a != b,
        Relop::Gt => a > b,
        Relop::Lt => a < b,
        Relop::Ge => a >= b,
        Relop::Le => a <= b,
    }
}

/// Execute `PERFORM VARYING I FROM <from> BY <by> UNTIL I > <limit>` accumulating `TABLE(I)`, optionally only
/// where `TABLE(I) <op> <value>`. Test-before; only in-bounds indices contribute.
pub fn eval_table_loop(
    record: &[u8],
    table: &Table,
    from: i64,
    by: i64,
    limit: i64,
    filter: Option<(Relop, i64)>,
) -> TableLoopResult {
    let mut sum = 0i64;
    let mut count = 0usize;
    let mut i = from;
    while i <= limit {
        if i >= 1 && (i as usize) <= table.occurs {
            let v = table_elem(record, table, i as usize);
            let pass = match &filter {
                None => true,
                Some((op, val)) => relop_int(v, op, *val),
            };
            if pass {
                sum += v;
                count += 1;
            }
        }
        i += by;
    }
    TableLoopResult { sum, count, final_index: i }
}

#[cfg(test)]
mod tests {
    use super::*;

    // T-ELEM 9(3) OCCURS 5 = "100025050200007"
    fn table() -> Table {
        Table { base_offset: 0, elem_size: 3, occurs: 5 }
    }

    #[test]
    fn sum_whole_table() {
        let r = eval_table_loop(b"100025050200007", &table(), 1, 1, 5, None);
        assert_eq!(r.sum, 382); // 100+25+50+200+7
        assert_eq!(r.count, 5);
        assert_eq!(r.final_index, 6); // one past the limit
    }

    #[test]
    fn count_filtered_elements() {
        let r = eval_table_loop(b"100025050200007", &table(), 1, 1, 5, Some((Relop::Ge, 50)));
        assert_eq!(r.count, 3); // 100, 50, 200
        assert_eq!(r.sum, 350);
    }

    #[test]
    fn subscript_is_one_based() {
        // TABLE(1) is the first element (100), not TABLE(0)
        assert_eq!(table_elem(b"100025050200007", &table(), 1), 100);
        assert_eq!(table_elem(b"100025050200007", &table(), 5), 7);
        assert_eq!(table_elem(b"100025050200007", &table(), 0), 0); // out of range
        assert_eq!(table_elem(b"100025050200007", &table(), 6), 0); // out of range
    }
}

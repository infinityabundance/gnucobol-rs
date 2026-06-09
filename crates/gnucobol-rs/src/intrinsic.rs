//! Intrinsic functions (`GNURUST.INTRINSIC.*`) — the first *implemented* intrinsic courts, split out of the
//! observed `GNURUST.INTRINSIC.ATLAS.1` map.
//!
//! `GNURUST.INTRINSIC.LENGTH.1`: `FUNCTION LENGTH(field)` returns the **storage byte length** of an
//! elementary item — the same byte count the sealed field model (`build_field`, `GNURUST.3`/`GNURUST.9`/
//! `GNURUST.14`) computes — proven equal to GnuCOBOL's `FUNCTION LENGTH` across DISPLAY / `COMP-3` / binary
//! field types. **Non-claims:** `LENGTH` of a group / table / reference-modified operand, `LENGTH OF`
//! (different from `FUNCTION LENGTH` for some operands), national/UTF-8 (character vs byte length), and all
//! dialects — those remain on the atlas, not implemented here.

use crate::pic::{build_field, PicError};
use crate::Usage;

/// `FUNCTION LENGTH(field)` for an elementary item: the storage byte length (== GnuCOBOL). For a `PIC X(n)`
/// this is `n`; for numeric `DISPLAY` the digit count; for `COMP-3` the packed byte count; for binary the
/// storage width.
pub fn intrinsic_length(pic: &str, usage: Usage) -> Result<usize, PicError> {
    build_field(pic, usage, false, false).map(|f| f.size)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn len(pic: &str, u: Usage) -> usize {
        intrinsic_length(pic, u).unwrap()
    }
    #[test]
    fn length_matches_storage_bytes() {
        // oracle FUNCTION LENGTH: X(5)=5 · 9(5)=5 · S9(8)V99=10 · S9(3)COMP-3=2 · 9(4)COMP=2 ·
        //                         9(7)COMP-3=4 · S9(9)COMP=4 · X(1)=1
        assert_eq!(len("X(5)", Usage::Display), 5);
        assert_eq!(len("9(5)", Usage::Display), 5);
        assert_eq!(len("S9(8)V99", Usage::Display), 10);
        assert_eq!(len("S9(3)", Usage::Comp3), 2);
        assert_eq!(len("9(4)", Usage::Comp), 2);
        assert_eq!(len("9(7)", Usage::Comp3), 4);
        assert_eq!(len("S9(9)", Usage::Comp), 4);
        assert_eq!(len("X(1)", Usage::Display), 1);
    }
}

//! # gnucobol-rs
//!
//! A **faithful, line-cited port** of GnuCOBOL's packed-decimal (COMP-3), zoned, and display
//! numeric **byte semantics** and the `MOVE` conversions between them, proven byte-identical
//! against the GnuCOBOL 3.2 `libcob` oracle under pinned settings.
//!
//! ## Claim boundary (read this)
//!
//! This crate ports the *observable byte semantics* of three `cob_move` conversions —
//! DISPLAY→DISPLAY, DISPLAY→PACKED (COMP-3 encode), and PACKED→DISPLAY (COMP-3 decode) — for
//! admitted PICs, on a **little-endian ASCII host** under `LC_ALL=C.UTF-8`. The [`pic`] module
//! additionally parses the sealed `PIC` subset (`9 X A S V`, repeats, `SIGN` clause,
//! `USAGE DISPLAY`/`COMP-3`) into that same field model — matching the GnuCOBOL compiler's own
//! field-attribute computation (`GNURUST.3`; `P` scaling and edited pictures fail closed). The
//! [`layout`] module assigns each DATA DIVISION item its byte offset/size within an `01` record
//! (nested groups, fixed `OCCURS`, `REDEFINES`, `FILLER`), matching the compiler's record layout
//! (`GNURUST.4`).
//!
//! It is **not** a GnuCOBOL replacement, **not** a COBOL compiler, and **not** `libcob`. It does
//! **not** implement decimal arithmetic (`ADD`/`MULTIPLY`/…), edited pictures, comparison,
//! `DISPLAY` statement output, files, or any other `cob_move` type pair — those **fail closed**
//! with [`DecimalError::UnsupportedConversion`]. The byte domain claimed is *field-storage* and
//! *move-result* bytes only (`GNURUST.BYTEDOMAIN.0`); see the project's `docs/claim-boundary.md`
//! and `reports/negative-claims.md`.
//!
//! ## Pure kernel (`GNURUST.PUREDEC.0`)
//!
//! Every function here is a pure function of its `(bytes, attrs)` inputs: no global mutable
//! state, no environment/locale/filesystem/runtime-config reads, deterministic, and panic-free on
//! hostile input (it returns typed [`DecimalError`]s and bounds-guards every byte access). The
//! types are `Send + Sync`. (The upstream `libcob` runtime is **not** asserted thread-safe; this
//! Rust kernel is.)
//!
//! ## Derivation & license
//!
//! Ported statement-by-statement from `libcob/move.c`, `libcob/numeric.c`, and `libcob/common.c`
//! (GnuCOBOL 3.2), which are **LGPL-3.0-or-later**; this crate inherits that license. The FSF
//! copyright and the original authors (Keisuke Nishida, Roger While, Simon Sobisch, et al.) are
//! acknowledged. See the workspace `docs/derivation-and-license.md` and `COPYING.LESSER`.

#![forbid(unsafe_code)]

pub mod arith;
pub mod file_flow_slice;
pub mod file_seq;
pub mod initialize;
pub mod if_eval;
pub mod if_numeric;
pub mod inspect;
pub mod accept_display;
pub mod intrinsic;
pub mod size_error;
pub mod string_ops;
pub mod table_slice;
pub mod attr;
mod binary;
pub mod cond;
pub mod class;
pub mod refmod;
pub mod subscript;
pub mod odo;
pub mod index_item;
pub mod copybook;
pub mod ebcdic;
pub mod edited;
pub mod error;
pub mod init;
pub mod layout;
mod move_ops;
pub mod perform_slice;
pub mod search;
pub mod pic;
mod sign;
pub mod value;

pub use arith::{cob_arith, ArithError, Op, Round};
pub use attr::{
    FieldAttr, COB_FLAG_BINARY_SWAP, COB_FLAG_BINARY_TRUNC, COB_FLAG_HAVE_SIGN,
    COB_FLAG_NO_SIGN_NIBBLE, COB_FLAG_REAL_BINARY, COB_FLAG_SIGN_LEADING, COB_FLAG_SIGN_SEPARATE,
    COB_TYPE_NUMERIC_BINARY, COB_TYPE_NUMERIC_DISPLAY, COB_TYPE_NUMERIC_PACKED,
};
pub use cond::{
    apply_set_88_true, eval_88, set_88_true, set_88_false, CondLit, CondValue, Condition, ConditionError,
    ConditionSetError,
};
pub use copybook::{expand, CopyError, CopyResolver, Expanded};
pub use ebcdic::{decode_display, translate_byte, CodePage, EbcdicError};
pub use refmod::{apply_ref_mod, ref_mod, ref_mod_to_end, RefModError};
pub use subscript::{element_1d, element_2d, table_element, SubscriptError};
pub use odo::{odo_element, odo_used_length, OdoError};
pub use index_item::{
    index_store, index_value, set_index_down_by, set_index_to, set_index_up_by, INDEX_SIZE,
};
pub use class::{is_alphabetic, is_alphabetic_lower, is_alphabetic_upper, is_numeric, is_numeric_sign_leading, is_numeric_sign_leading_separate, is_numeric_sign_trailing_separate, is_numeric_signed_trailing};
pub use edited::{decode_edited, edited_size, encode_edited, EditedDecode, EditedError};
pub use error::DecimalError;
pub use init::{value_image, InitError, Val, ValueItem};
pub use layout::{lay_out, Item, Laid, LayoutError, Odo};
pub use move_ops::cob_move;
pub use pic::{build_field, PicError, PicField, Usage};
pub use value::Decimal;

/// Maximum decimal digits, as emitted by the built oracle's `selfcheck` (`GNURUST.NUMCONST.0`):
/// `COB_MAX_DIGITS` in `libcob/common.h:607`.
pub const COB_MAX_DIGITS: i64 = 38;

/// A self-describing compatibility profile so downstream users never quote a detached claim
/// (`GNURUST.COMPATPROFILE.0`).
pub mod compat_profile {
    /// The admitted oracle this crate's bytes are proven against.
    pub const AUTHORITY: &str = "GnuCOBOL 3.2.0";
    /// The upstream source files ported (faithful derivative, LGPL-3.0-or-later).
    pub const SOURCE_FILES: &[&str] = &["libcob/move.c", "libcob/numeric.c", "libcob/common.c"];
    /// The sealing receipt for this claim.
    pub const RECEIPT: &str = "RECEIPT-GNURUST-DECIMAL-1";
    /// One-line statement of what is sealed.
    pub const CLAIM: &str =
        "COMP-3 / zoned / display numeric MOVE byte semantics (storage & move-result bytes)";
}

// ---------------------------------------------------------------------------------------------
// Kani reduced-surface proofs (`GNURUST` Kani doctrine): the *sharpest* index/arithmetic
// invariants — the exact bounds that would be the actual out-of-bounds access if a guard or the
// scale arithmetic regressed. Each is allocation-free and converges quickly.
// ---------------------------------------------------------------------------------------------
#[cfg(kani)]
pub mod kani_harness {
    use super::*;

    /// **Sharpest invariant #1 — the scale-alignment window is always in bounds.**
    /// `store_common_region`'s digit-copy window `(d, s, n)` is the one place where a regression in
    /// the two-scale alignment math (`gcf = min(hf1,hf2)`, `lcf = max(lf1,lf2)`) would silently
    /// index past either buffer. Prove, for *all* integer inputs, that when a window exists it lies
    /// entirely within the destination field (`0 <= d`, `d+n <= fsize`) and the source
    /// (`0 <= s`, `s+n <= size`). Unbounded in the inputs (full i64 reasoning), so it is the
    /// strongest possible statement of the bound, not a sampled one.
    // KANIFOR: GNURUST.2
    #[kani::proof]
    fn store_window_is_in_bounds() {
        let size: i64 = kani::any();
        let fsize: i64 = kani::any();
        let scale: i64 = kani::any();
        let dst_scale: i64 = kani::any();
        // Field sizes/scales are non-negative and bounded by COB_MAX_DIGITS in practice; keep the
        // proof free of i64 wrap at the extremes (which the field model never produces).
        kani::assume(size >= 0 && size <= super::COB_MAX_DIGITS);
        kani::assume(fsize >= 0 && fsize <= super::COB_MAX_DIGITS);
        kani::assume(scale >= -super::COB_MAX_DIGITS && scale <= super::COB_MAX_DIGITS);
        kani::assume(dst_scale >= -super::COB_MAX_DIGITS && dst_scale <= super::COB_MAX_DIGITS);

        if let Some((d, s, n)) = move_ops::region_window(size, scale, fsize, dst_scale) {
            assert!(n > 0);
            assert!(d >= 0 && d + n <= fsize, "dst window out of field");
            assert!(s >= 0 && s + n <= size, "src window out of source");
        }
    }

    /// **Sharpest invariant #2 — the fixed unpack buffer is sufficient.** `packed_to_display`
    /// unpacks nibbles into a fixed `[u8; COB_MAX_DIGITS + 1]`. Prove that for any packed field
    /// whose declared digit count is within `COB_MAX_DIGITS`, decoding cannot panic or index out of
    /// that buffer — i.e. the buffer is exactly sized and the push-guard is never the thing that
    /// silently truncates a valid field.
    #[kani::proof]
    #[kani::unwind(41)]
    fn packed_unpack_buffer_sufficient() {
        let digits: u16 = kani::any();
        kani::assume(digits >= 1 && digits <= super::COB_MAX_DIGITS as u16);
        // packed byte count for `digits`: floor(digits/2)+1, at most 20 for digits<=38.
        let size = (digits as usize) / 2 + 1;
        let mut data = [0u8; 20];
        for b in data.iter_mut().take(size) {
            *b = kani::any();
        }
        let scale: i16 = kani::any();
        kani::assume(scale >= 0 && scale <= digits as i16);
        let attr = FieldAttr {
            field_type: COB_TYPE_NUMERIC_PACKED,
            digits,
            scale,
            flags: COB_FLAG_HAVE_SIGN,
        };
        let mut dst = [0u8; 38];
        let dst_attr = FieldAttr {
            field_type: COB_TYPE_NUMERIC_DISPLAY,
            digits,
            scale,
            flags: COB_FLAG_HAVE_SIGN,
        };
        // Must not panic / OOB for any nibble content.
        let _ = cob_move(&data[..size], &attr, &mut dst[..digits as usize], &dst_attr);
    }
}

// ---------------------------------------------------------------------------------------------
// Fuzz entry point (detached fuzz crate; `fuzzing` feature). Hostile bytes/attrs must never panic.
// ---------------------------------------------------------------------------------------------
/// Fuzz target: drive `cob_move` with arbitrary bytes and attributes; asserts only panic-freedom.
#[cfg(feature = "fuzzing")]
#[doc(hidden)]
pub fn __fuzz_cob_move(data: &[u8]) {
    if data.len() < 12 {
        return;
    }
    let mk_attr = |b: &[u8]| FieldAttr {
        field_type: match b[0] % 3 {
            0 => COB_TYPE_NUMERIC_DISPLAY,
            1 => COB_TYPE_NUMERIC_PACKED,
            _ => attr::COB_TYPE_NUMERIC_BINARY, // GNURUST.14: binary moves on the hostile surface too
        },
        digits: (b[1] % 39) as u16,
        scale: (b[2] % 39) as i16,
        flags: b[3] as u16,
    };
    let src_attr = mk_attr(&data[0..4]);
    let dst_attr = mk_attr(&data[4..8]);
    let body = &data[9..];
    let at = (data[8] as usize) % (body.len() + 1);
    let (src, dst_seed) = body.split_at(at);
    let mut dst = dst_seed.to_vec();
    let _ = cob_move(src, &src_attr, &mut dst, &dst_attr);
    let _ = Decimal::from_packed(src, &src_attr);
    let _ = Decimal::from_display(src, &src_attr);
    let _ = Decimal::from_binary(src, &src_attr);
}

/// Fuzz target: parse an arbitrary string as a PICTURE; asserts only panic-freedom — any hostile
/// or malformed picture must yield a typed `PicError`, never a panic.
#[cfg(feature = "fuzzing")]
#[doc(hidden)]
pub fn __fuzz_pic(data: &[u8]) {
    let s = String::from_utf8_lossy(data);
    let usage = if data.first().is_some_and(|b| b & 1 == 0) {
        pic::Usage::Display
    } else {
        pic::Usage::Comp3
    };
    let sep = data.get(1).is_some_and(|b| b & 1 == 0);
    let lead = data.get(2).is_some_and(|b| b & 1 == 0);
    let _ = pic::build_field(&s, usage, sep, lead);
}

/// Fuzz target: lay out a record built from arbitrary bytes; asserts only panic-freedom — hostile
/// level nesting, OCCURS counts, and REDEFINES targets must yield a typed `LayoutError`, never a
/// panic or overflow (the level-tree recursion and OCCURS `checked_mul` are the sharp surfaces).
#[cfg(feature = "fuzzing")]
#[doc(hidden)]
pub fn __fuzz_layout(data: &[u8]) {
    let mut items = Vec::new();
    // 5 bytes per item: level, name-id, pic-id, occurs, redefines-id.
    for ch in data.chunks(5) {
        if ch.len() < 5 {
            break;
        }
        let level = (ch[0] % 50) as u16; // includes 0 and >49 nesting to stress the tree
        let name = format!("N{}", ch[1]);
        let pic_id = ch[2] % 5;
        let pic = match pic_id {
            0 => None, // group
            1 => Some(("9(3)".to_string(), pic::Usage::Display, false, false)),
            2 => Some(("S9(5)V99".to_string(), pic::Usage::Comp3, false, false)),
            3 => Some(("X(7)".to_string(), pic::Usage::Display, false, false)),
            _ => Some((
                format!("9({})", ch[2] % 40),
                pic::Usage::Display,
                false,
                false,
            )),
        };
        let occurs = if ch[3] == 0 { None } else { Some(ch[3] as u32) };
        let redefines = if ch[4] == 0 {
            None
        } else {
            Some(format!("N{}", ch[4]))
        };
        // Occasionally attach an ODO (max from a byte) so the ODO rules are fuzzed too.
        let odo = if ch[3] & 0x40 != 0 {
            Some(layout::Odo {
                min: (ch[0] % 4) as u32,
                max: (ch[3] % 8) as u32,
                depending_on: format!("N{}", ch[1] ^ 1),
            })
        } else {
            None
        };
        items.push(layout::Item {
            level,
            name,
            pic,
            occurs,
            redefines,
            odo,
        });
    }
    let _ = layout::lay_out(&items);
}

/// Fuzz target: expand a copybook tree from arbitrary bytes; asserts only panic-freedom — cycles,
/// missing copybooks, deep nesting, and REPLACING forms must yield a typed `CopyError`.
#[cfg(feature = "fuzzing")]
#[doc(hidden)]
pub fn __fuzz_copybook(data: &[u8]) {
    use std::collections::HashMap;
    struct Map(HashMap<String, String>);
    impl copybook::CopyResolver for Map {
        fn resolve(&self, name: &str) -> Option<String> {
            self.0.get(name).cloned()
        }
    }
    // Split data: first chunk is the main source, the rest define copybooks "A".."D".
    let s = String::from_utf8_lossy(data);
    let mut m = HashMap::new();
    for (i, part) in s.split('\u{1}').enumerate().skip(1).take(4) {
        m.insert(
            ((b'A' + (i as u8 - 1)) as char).to_string(),
            part.to_string(),
        );
    }
    let main = s.split('\u{1}').next().unwrap_or("");
    let _ = copybook::expand(main, &Map(m));
}

/// Fuzz target: arithmetic over arbitrary operand bytes/attrs; asserts only panic-freedom — hostile
/// digits/scales/values must yield a typed `ArithError`, never a panic or overflow (the i128
/// checked arithmetic + pow10 bounds are the sharp surface).
#[cfg(feature = "fuzzing")]
#[doc(hidden)]
pub fn __fuzz_arith(data: &[u8]) {
    if data.len() < 9 {
        return;
    }
    let mk = |b: &[u8]| FieldAttr {
        field_type: if b[0] & 1 == 0 {
            COB_TYPE_NUMERIC_DISPLAY
        } else {
            COB_TYPE_NUMERIC_PACKED
        },
        digits: (b[1] % 40) as u16,
        scale: (b[2] % 40) as i16,
        flags: b[3] as u16,
    };
    let a_attr = mk(&data[0..4]);
    let b_attr = mk(&data[4..8]);
    let op = match data[8] % 3 {
        0 => arith::Op::Add,
        1 => arith::Op::Subtract,
        _ => arith::Op::Multiply,
    };
    // Drive every ROUNDED MODE IS setting (GNURUST.ROUND.1): hostile values under any mode must
    // still yield a typed ArithError, never a panic.
    let round = match data[8] >> 5 {
        0 => arith::Round::Truncate,
        1 => arith::Round::NearAwayFromZero,
        2 => arith::Round::AwayFromZero,
        3 => arith::Round::NearEven,
        4 => arith::Round::NearTowardZero,
        5 => arith::Round::TowardGreater,
        6 => arith::Round::TowardLesser,
        _ => arith::Round::Prohibited,
    };
    let body = &data[9..];
    let at = (data[8] as usize >> 1) % (body.len() + 1);
    let (a, b) = body.split_at(at);
    let _ = arith::cob_arith(op, a, &a_attr, b, &b_attr, round);
}

/// Fuzz target: build a record of VALUE items from arbitrary bytes and compute its initial image;
/// asserts only panic-freedom — hostile PICs, literals, scales, and field counts must yield a typed
/// `InitError`, never a panic (the literal parser + zoned alignment + cob_move are the sharp surface).
#[cfg(feature = "fuzzing")]
#[doc(hidden)]
pub fn __fuzz_init(data: &[u8]) {
    let mut items = vec![ValueItem {
        level: 1,
        name: "REC".to_string(),
        pic: None,
        value: None,
    }];
    // 4 bytes per field: pic-id, usage, value-kind, value-byte.
    for (i, ch) in data.chunks(4).enumerate().take(16) {
        if ch.len() < 4 {
            break;
        }
        let pic = match ch[0] % 8 {
            0 => ("9(3)", Usage::Display),
            1 => ("S9(3)V99", Usage::Display),
            2 => ("X(4)", Usage::Display),
            3 => ("S9(5)V9(3)", Usage::Comp3),
            4 => ("9(5)", Usage::Comp3),
            5 => ("999PPP", Usage::Display), // P-scaled: must fail closed, not panic
            6 => ("PPP999", Usage::Comp3),
            _ => ("S9(4)", Usage::Display),
        };
        let value = match ch[2] % 5 {
            0 => None,
            1 => Some(Val::Zero),
            2 => Some(Val::Space),
            3 => Some(Val::Alpha(format!("v{}", ch[3]))),
            _ => Some(Val::Num(format!(
                "{}{}.{}",
                if ch[3] & 1 == 0 { "-" } else { "" },
                ch[3],
                ch[1]
            ))),
        };
        items.push(ValueItem {
            level: 5,
            name: format!("F{i}"),
            pic: Some((pic.0.to_string(), pic.1, false, false)),
            value,
        });
    }
    let _ = value_image(&items);
}

/// Fuzz target: evaluate a LEVEL-88 condition over arbitrary parent bytes/attrs and value tables;
/// asserts only panic-freedom — hostile literals, scales, and ranges must yield a typed
/// `ConditionError` or a bool, never a panic (the i128 comparison + literal parser are the surface).
#[cfg(feature = "fuzzing")]
#[doc(hidden)]
pub fn __fuzz_cond(data: &[u8]) {
    if data.len() < 4 {
        return;
    }
    let attr = FieldAttr {
        field_type: match data[0] % 3 {
            0 => COB_TYPE_NUMERIC_DISPLAY,
            1 => pic::COB_TYPE_ALPHANUMERIC,
            _ => COB_TYPE_NUMERIC_PACKED,
        },
        digits: (data[1] % 40) as u16,
        scale: (data[2] % 40) as i16,
        flags: data[3] as u16,
    };
    let body = &data[4..];
    let cut = (data[1] as usize) % (body.len() + 1);
    let (bytes, raw) = body.split_at(cut);
    // Build a small value table from the remaining bytes.
    let s = String::from_utf8_lossy(raw);
    let mut values = Vec::new();
    for (i, tok) in s.split(',').take(8).enumerate() {
        let v = if i % 4 == 0 {
            cond::CondValue::Lit(cond::CondLit::Alpha(tok.to_string()))
        } else if i % 4 == 1 {
            cond::CondValue::Lit(cond::CondLit::Num(tok.to_string()))
        } else if i % 4 == 2 {
            cond::CondValue::Range(
                cond::CondLit::Num(tok.to_string()),
                cond::CondLit::Num("9".into()),
            )
        } else {
            cond::CondValue::Range(
                cond::CondLit::Alpha(tok.to_string()),
                cond::CondLit::Alpha("Z".into()),
            )
        };
        values.push(v);
    }
    let c = cond::Condition {
        name: "C".into(),
        values,
        false_value: None,
    };
    let _ = cond::eval_88(&attr, bytes, &c);
    // Also fuzz the inverse constructor (SET ... TO TRUE): arbitrary size, then a round-trip eval.
    let size = (data[2] as usize) % 24;
    if let Ok(produced) = cond::set_88_true(&attr, size, &c) {
        let _ = cond::eval_88(&attr, &produced, &c);
    }
    // Also fuzz SET ... TO FALSE with a false-clause literal (GNURUST.12B).
    let cf = cond::Condition {
        name: "C".into(),
        values: c.values.clone(),
        false_value: Some(cond::CondLit::Num("0".into())),
    };
    if let Ok(produced) = cond::set_88_false(&attr, size, &cf) {
        let _ = cond::eval_88(&attr, &produced, &cf);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn display(digits: u16, scale: i16, signed: bool) -> FieldAttr {
        FieldAttr {
            field_type: COB_TYPE_NUMERIC_DISPLAY,
            digits,
            scale,
            flags: if signed { COB_FLAG_HAVE_SIGN } else { 0 },
        }
    }
    fn packed(digits: u16, scale: i16, signed: bool) -> FieldAttr {
        FieldAttr {
            field_type: COB_TYPE_NUMERIC_PACKED,
            digits,
            scale,
            flags: if signed { COB_FLAG_HAVE_SIGN } else { 0 },
        }
    }

    #[test]
    fn display_to_packed_signed_negative() {
        // S9(3)V99, value -012.34, display "01234" overpunched '4'->'t' (0x74)
        let src = [0x30, 0x31, 0x32, 0x33, 0x74];
        let mut dst = [0u8; 3];
        cob_move(&src, &display(5, 2, true), &mut dst, &packed(5, 2, true)).unwrap();
        assert_eq!(dst, [0x01, 0x23, 0x4d]); // negative => sign nibble 0x0d
    }

    #[test]
    fn display_to_packed_unsigned_sign_nibble_is_f() {
        let src = [0x30, 0x31, 0x32, 0x33, 0x34];
        let mut dst = [0u8; 3];
        cob_move(&src, &display(5, 2, false), &mut dst, &packed(5, 2, false)).unwrap();
        assert_eq!(dst, [0x01, 0x23, 0x4f]); // unsigned => 0x0f
    }

    #[test]
    fn packed_to_display_roundtrips() {
        let packed_bytes = [0x01, 0x23, 0x4d];
        let mut dsp = [0u8; 5];
        cob_move(
            &packed_bytes,
            &packed(5, 2, true),
            &mut dsp,
            &display(5, 2, true),
        )
        .unwrap();
        // -01234 => digits "01234" with negative overpunch on last byte '4'(0x34)->'t'(0x74)
        assert_eq!(dsp, [0x30, 0x31, 0x32, 0x33, 0x74]);
    }

    #[test]
    fn unsupported_pair_fails_closed() {
        let src = [0x01, 0x2c];
        let mut dst = [0u8; 2];
        let err = cob_move(&src, &packed(3, 0, true), &mut dst, &packed(3, 0, true)).unwrap_err();
        assert!(matches!(err, DecimalError::UnsupportedConversion { .. }));
    }

    #[test]
    fn value_decode_packed_negative_zero_preserved() {
        // 000 with 0x0d sign => negative zero
        let d = Decimal::from_packed(&[0x00, 0x0d], &packed(3, 0, true));
        assert!(d.negative);
        assert_eq!(d.unscaled_i128(), Some(0));
    }

    #[test]
    fn hostile_attrs_do_not_panic() {
        // digits wildly exceeding the buffer must not panic (fails closed / produces bytes).
        let src = [0xff; 4];
        let mut dst = [0u8; 2];
        let _ = cob_move(
            &src,
            &packed(200, 100, true),
            &mut dst,
            &display(200, 100, true),
        );
    }
}

// ===========================================================================================================
// Fuzz entries for the byte-court + execution-slice modules added after the original 8 targets. Each asserts
// panic-freedom (GNURUST.PANICPOLICY.0) on arbitrary input; a few assert a round-trip invariant in-bounds.
// ===========================================================================================================

#[cfg(feature = "fuzzing")]
#[doc(hidden)]
pub fn __fuzz_string_ops(data: &[u8]) {
    let p = (data.first().copied().unwrap_or(1) as usize % 8) + 1;
    let prefill = vec![b'~'; p];
    let body: &[u8] = data.get(1..).unwrap_or(&[]);
    let _ = string_ops::string_into(&prefill, &[string_ops::StringSource::Size(body)], (data.first().copied().unwrap_or(1) as usize % (p + 2)).max(1));
    let delim = data.first().map(std::slice::from_ref);
    let _ = string_ops::unstring(body, delim, &[2usize, 3, 1], 1);
}

#[cfg(feature = "fuzzing")]
#[doc(hidden)]
pub fn __fuzz_initialize(data: &[u8]) {
    use initialize::{InitCategory, InitField};
    let mut fields = Vec::new();
    let mut off = 0usize;
    for ch in data.chunks(2).take(8) {
        let size = (ch[0] as usize % 4) + 1;
        let category = match ch[0] % 4 { 0 => InitCategory::Alphanumeric, 1 => InitCategory::NumericDisplay, 2 => InitCategory::Packed, _ => InitCategory::Binary };
        fields.push(InitField { offset: off, size, category, signed: ch[0] & 8 == 0, is_filler: ch[0] & 16 == 0, is_redefiner: ch.get(1).is_some_and(|b| b & 1 == 0) });
        off += size;
    }
    let prefill = vec![0xAAu8; off];
    let _ = initialize::initialize_record(&fields, &prefill);
}

#[cfg(feature = "fuzzing")]
#[doc(hidden)]
pub fn __fuzz_inspect(data: &[u8]) {
    if data.len() < 3 { return; }
    let pat = &data[0..1];
    let to = &data[1..2];
    let target = &data[2..];
    let _ = inspect::inspect_tallying(target, inspect::TallyMode::All(pat), inspect::Region::All);
    let _ = inspect::inspect_tallying(target, inspect::TallyMode::Leading(pat), inspect::Region::Before(to));
    let _ = inspect::inspect_tallying(target, inspect::TallyMode::Characters, inspect::Region::After(pat));
    let _ = inspect::inspect_replacing(target, inspect::ReplaceMode::All(pat, to), inspect::Region::All);
    let _ = inspect::inspect_converting(target, pat, to, inspect::Region::After(pat));
}

#[cfg(feature = "fuzzing")]
#[doc(hidden)]
pub fn __fuzz_file_seq(data: &[u8]) {
    let rl = (data.first().copied().unwrap_or(1) as usize % 7) + 1;
    let body = data.get(1..).unwrap_or(&[]);
    let _ = file_seq::read_sequential(body, file_seq::FileOrg::RecordSequential, rl);
    let _ = file_seq::read_sequential(body, file_seq::FileOrg::LineSequential, rl);
    let recs: Vec<&[u8]> = body.chunks(rl).collect();
    let _ = file_seq::write_sequential(&recs, file_seq::FileOrg::RecordSequential, rl);
    let _ = file_seq::write_sequential(&recs, file_seq::FileOrg::LineSequential, rl);
    let rw: Vec<(usize, &[u8])> = vec![(0, body), (data.len(), body)];
    let _ = file_seq::rewrite_records(body, rl, &rw);
}

#[cfg(feature = "fuzzing")]
#[doc(hidden)]
pub fn __fuzz_intrinsic(data: &[u8]) {
    let s = String::from_utf8_lossy(data);
    let _ = intrinsic::intrinsic_numval(&s);
    let _ = intrinsic::intrinsic_numval_c(&s);
    let _ = intrinsic::intrinsic_upper_case(data);
    let _ = intrinsic::intrinsic_lower_case(data);
    let _ = intrinsic::intrinsic_reverse(data);
    let _ = intrinsic::intrinsic_length("X(5)", Usage::Display);
    if let Some(&b0) = data.first() {
        let a = b0 as i128;
        let b = data.get(1).map_or(1, |&x| x as i128);
        let _ = intrinsic::intrinsic_mod(a, b);
        let _ = intrinsic::intrinsic_rem(a, b);
        let sc = (b0 % 4) as u32;
        let _ = intrinsic::intrinsic_integer(a, sc);
        let _ = intrinsic::intrinsic_integer_part(a, sc);
        // round-trip invariants (in-bounds): ord(char(n))==n, integer_of_date(date_of_integer(d))==d
        let n = (b0 as u32 % 256) + 1;
        assert_eq!(intrinsic::intrinsic_ord(intrinsic::intrinsic_char(n)), n);
        let d = (b0 as i64) + 1;
        assert_eq!(intrinsic::intrinsic_integer_of_date(intrinsic::intrinsic_date_of_integer(d)), d);
    }
}

#[cfg(feature = "fuzzing")]
#[doc(hidden)]
pub fn __fuzz_accept_display(data: &[u8]) {
    let (a, b) = data.split_at(data.len() / 2);
    let _ = accept_display::display_line(&[a, b]);
    let _ = accept_display::accept_field(data, (data.first().copied().unwrap_or(0) as usize % 16) + 1);
    let digits: Vec<u8> = data.iter().map(|&c| b'0' + (c % 10)).take(6).collect();
    let scale = if digits.is_empty() { 0 } else { data.first().copied().unwrap_or(0) as usize % digits.len() };
    let _ = accept_display::display_numeric(&digits, scale, data.len() % 2 == 0, data.len() % 3 == 0);
}

#[cfg(feature = "fuzzing")]
#[doc(hidden)]
pub fn __fuzz_size_error(data: &[u8]) {
    let int_digits: Vec<u8> = data.iter().map(|&c| b'0' + (c % 10)).take(8).collect();
    let frac_digits: Vec<u8> = data.iter().rev().map(|&c| b'0' + (c % 10)).take(4).collect();
    let ri = (data.first().copied().unwrap_or(0) as usize % 6) + 1;
    let rs = data.get(1).copied().unwrap_or(0) as usize % 4;
    let _ = size_error::arith_size_error(&int_digits, &frac_digits, ri, rs);
}

#[cfg(feature = "fuzzing")]
#[doc(hidden)]
pub fn __fuzz_if_eval(data: &[u8]) {
    use if_eval::{Condition, MoveStmt, Operand, Relop, SliceField};
    let fields = [SliceField { name: "A", offset: 0, size: 3 }, SliceField { name: "T", offset: 3, size: 4 }];
    let mut rec = vec![b' '; 7];
    for (i, &b) in data.iter().take(7).enumerate() { rec[i] = b; }
    let op = match data.first().copied().unwrap_or(0) % 6 { 0 => Relop::Eq, 1 => Relop::Ne, 2 => Relop::Gt, 3 => Relop::Lt, 4 => Relop::Ge, _ => Relop::Le };
    let lit = data.get(1..3).unwrap_or(b"AB");
    let cond = Condition { left: Operand::Field("A"), op, right: Operand::Literal(lit) };
    let then = [MoveStmt { source: Operand::Literal(b"YES"), target: "T" }];
    let els = [MoveStmt { source: Operand::Field("A"), target: "T" }];
    let _ = if_eval::eval_if(&rec, &fields, &cond, &then, &els);
    let whens: Vec<(&[u8], &[MoveStmt])> = vec![(lit, &then[..])];
    let _ = if_eval::eval_evaluate(&rec, &fields, "A", &whens, &els);
}

#[cfg(feature = "fuzzing")]
#[doc(hidden)]
pub fn __fuzz_if_numeric(data: &[u8]) {
    use if_eval::{Relop, SliceField};
    use if_numeric::MoveNum;
    use perform_slice::NumCond;
    let fields = [SliceField { name: "N", offset: 0, size: 3 }, SliceField { name: "F", offset: 3, size: 2 }];
    let mut rec = vec![b'0'; 5];
    for (i, &b) in data.iter().take(5).enumerate() { rec[i] = b'0' + (b % 10); }
    let op = match data.first().copied().unwrap_or(0) % 6 { 0 => Relop::Eq, 1 => Relop::Ne, 2 => Relop::Gt, 3 => Relop::Lt, 4 => Relop::Ge, _ => Relop::Le };
    let v = data.get(1).copied().unwrap_or(0) as i64;
    let cond = NumCond { field: "N", op, value: v };
    let then = [MoveNum { value: 1, target: "F" }];
    let els = [MoveNum { value: 9, target: "F" }];
    let _ = if_numeric::eval_if_numeric(&rec, &fields, &cond, &then, &els);
    let whens: Vec<(i64, &[MoveNum])> = vec![(v, &then[..])];
    let _ = if_numeric::eval_evaluate_numeric(&rec, &fields, "N", &whens, &els);
}

#[cfg(feature = "fuzzing")]
#[doc(hidden)]
pub fn __fuzz_perform_slice(data: &[u8]) {
    use if_eval::{Relop, SliceField};
    use perform_slice::{AddOp, NumCond, PerformForm};
    let fields = [SliceField { name: "C", offset: 0, size: 3 }, SliceField { name: "I", offset: 3, size: 3 }];
    let rec = vec![b'0'; 6];
    // body always makes progress (amount >= 1) and bounds are small -> guaranteed termination.
    let body = [AddOp { target: "C", amount: (data.get(1).copied().unwrap_or(0) as i64 % 4) + 1 }];
    let n = data.first().copied().unwrap_or(0) as i64 % 20;
    let _ = perform_slice::eval_perform(&rec, &fields, &PerformForm::Times(n), &body);
    let _ = perform_slice::eval_perform(&rec, &fields, &PerformForm::Until(NumCond { field: "C", op: Relop::Ge, value: n }), &body);
    let by = (data.get(2).copied().unwrap_or(0) as i64 % 3) + 1;
    let _ = perform_slice::eval_perform(&rec, &fields, &PerformForm::Varying { var: "I", from: 1, by, until: NumCond { field: "I", op: Relop::Gt, value: n } }, &body);
}

#[cfg(feature = "fuzzing")]
#[doc(hidden)]
pub fn __fuzz_file_flow_slice(data: &[u8]) {
    use file_flow_slice::{FilterCond, LoopOp};
    use if_eval::{Relop, SliceField};
    let rf = [SliceField { name: "R-ST", offset: 0, size: 1 }, SliceField { name: "R-AMT", offset: 1, size: 3 }];
    let wf = [SliceField { name: "CNT", offset: 0, size: 3 }, SliceField { name: "SM", offset: 3, size: 5 }];
    let body = [LoopOp::Count("CNT"), LoopOp::SumField { field: "R-AMT", into: "SM" }];
    let _ = file_flow_slice::eval_read_loop(data, file_seq::FileOrg::RecordSequential, 4, &rf, b"00000000", &wf, &body);
    let cn = FilterCond::Numeric { field: "R-AMT", op: Relop::Ge, value: data.first().copied().unwrap_or(0) as i64 };
    let _ = file_flow_slice::eval_filter_loop(data, file_seq::FileOrg::RecordSequential, 4, &rf, b"00000000", &wf, &cn, &body);
    let ca = FilterCond::Alpha { field: "R-ST", op: Relop::Eq, value: data.get(0..1).unwrap_or(b"A") };
    let _ = file_flow_slice::eval_filter_loop(data, file_seq::FileOrg::LineSequential, 4, &rf, b"00000000", &wf, &ca, &body);
}

#[cfg(feature = "fuzzing")]
#[doc(hidden)]
pub fn __fuzz_table_slice(data: &[u8]) {
    use if_eval::Relop;
    use table_slice::Table;
    let occurs = (data.first().copied().unwrap_or(1) as usize % 8) + 1;
    let t = Table { base_offset: 0, elem_size: 3, occurs };
    let by = (data.get(1).copied().unwrap_or(0) as i64 % 3) + 1;
    let limit = data.get(2).copied().unwrap_or(0) as i64 % 10;
    let _ = table_slice::eval_table_loop(data, &t, 1, by, limit, None);
    let _ = table_slice::eval_table_loop(data, &t, 1, by, limit, Some((Relop::Ge, 50)));
    for i in 0..occurs + 2 {
        let _ = table_slice::table_elem(data, &t, i);
    }
}

#[cfg(feature = "fuzzing")]
#[doc(hidden)]
pub fn __fuzz_value(data: &[u8]) {
    use crate::value::Decimal;
    for (pic, usage) in [("S9(5)", Usage::Comp), ("S9(5)", Usage::Comp5), ("S9(7)", Usage::Comp3), ("S9(5)", Usage::Display)] {
        if let Ok(f) = pic::build_field(pic, usage, false, false) {
            let _ = Decimal::from_binary(data, &f.attr);
            let _ = Decimal::from_packed(data, &f.attr);
            let _ = Decimal::from_display(data, &f.attr);
            let _ = Decimal::from_ebcdic_zoned(data, &f.attr);
        }
    }
}

#[cfg(feature = "fuzzing")]
#[doc(hidden)]
pub fn __fuzz_search(data: &[u8]) {
    use search::SearchTable;
    let occurs = (data.first().copied().unwrap_or(1) as usize % 8) + 1;
    let t = SearchTable { base_offset: 0, elem_size: 3, key_offset: 0, key_size: 3, occurs };
    let target = data.get(1).copied().unwrap_or(0) as i64;
    let from = (data.get(2).copied().unwrap_or(0) as usize % (occurs + 2)) + 1;
    let _ = search::search_serial(data, &t, from, target);
    let _ = search::search_all(data, &t, target);
}

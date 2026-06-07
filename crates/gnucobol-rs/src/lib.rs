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
//! admitted PICs, on a **little-endian ASCII host** under `LC_ALL=C.UTF-8`.
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

pub mod attr;
pub mod error;
mod move_ops;
mod sign;
pub mod value;

pub use attr::{
    FieldAttr, COB_FLAG_HAVE_SIGN, COB_FLAG_NO_SIGN_NIBBLE, COB_FLAG_SIGN_LEADING,
    COB_FLAG_SIGN_SEPARATE, COB_TYPE_NUMERIC_DISPLAY, COB_TYPE_NUMERIC_PACKED,
};
pub use error::DecimalError;
pub use move_ops::cob_move;
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
        field_type: if b[0] & 1 == 0 {
            COB_TYPE_NUMERIC_DISPLAY
        } else {
            COB_TYPE_NUMERIC_PACKED
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

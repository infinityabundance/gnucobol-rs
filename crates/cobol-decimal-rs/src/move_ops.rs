//! Faithful port of the GnuCOBOL `MOVE` byte conversions for the sealed type pairs:
//! DISPLAY→DISPLAY, DISPLAY→PACKED (COMP-3 encode), PACKED→DISPLAY (COMP-3 decode).
//!
//! Citations: `cob_move` dispatch (`move.c:1446`), `store_common_region` (`move.c:147`),
//! `cob_move_display_to_display` (`move.c:372`), `cob_move_display_to_packed` (`move.c:477`),
//! `cob_move_packed_to_display` (`move.c:582`). Control flow mirrors upstream, including its
//! one-byte-past read in the packing loop (cleaned by the trailing `&= 0xf0`, faithfully
//! reproduced) — here every such read is **bounds-guarded** to a `0`, which yields the identical
//! result because that nibble is always cleaned. Evidence kind: **source_port**.

use crate::attr::{FieldAttr, COB_TYPE_NUMERIC_DISPLAY, COB_TYPE_NUMERIC_PACKED};
use crate::error::DecimalError;
use crate::sign;

/// Bounds-guarded read: out-of-range index yields `0` (never panics on hostile attrs).
#[inline]
fn rd(buf: &[u8], idx: i64) -> u8 {
    if idx >= 0 && (idx as usize) < buf.len() {
        buf[idx as usize]
    } else {
        0
    }
}

/// Bounds-guarded write: out-of-range index is dropped (the C never writes past `f->size`;
/// this only differs for self-inconsistent attrs, where it fails closed instead of corrupting).
#[inline]
fn wr(buf: &mut [u8], idx: i64, val: u8) {
    if idx >= 0 && (idx as usize) < buf.len() {
        buf[idx as usize] = val;
    }
}

/// Guarded write into a `fdata` region that starts at `off` within `full`.
#[inline]
fn wr_off(full: &mut [u8], off: usize, didx: i64, fsize: i64, val: u8) {
    if didx >= 0 && didx < fsize {
        let abs = off as i64 + didx;
        wr(full, abs, val);
    }
}

/// The scale-aligned copy window of `store_common_region` (`move.c:147-164`), as a pure function
/// so its index bounds can be Kani-proved as the single source of truth. Returns
/// `(dst_start, src_start, len)` for the digit-copy, or `None` when the scales do not overlap.
///
/// **Sharp invariant** (proved in [`crate::kani_harness::store_window_is_in_bounds`]): whenever
/// this returns `Some((d, s, n))`, the window stays inside both buffers for *all* integer inputs —
/// `0 <= d` and `d + n <= fsize`, and `0 <= s` and `s + n <= size`. This holds because
/// `gcf = min(hf1, hf2)` and `lcf = max(lf1, lf2)`, so `d + n = hf2 - lcf <= hf2 - lf2 = fsize`
/// and `s + n = hf1 - lcf <= hf1 - lf1 = size`. It is the exact bound that would be violated if the
/// scale arithmetic regressed.
pub(crate) fn region_window(
    size: i64,
    scale: i64,
    fsize: i64,
    dst_scale: i64,
) -> Option<(i64, i64, i64)> {
    let lf1 = -scale;
    let lf2 = -dst_scale;
    let lcf = lf1.max(lf2); // cob_max_int (move.c:155)
    let hf1 = size + lf1;
    let hf2 = fsize + lf2;
    let gcf = hf1.min(hf2); // cob_min_int (move.c:159)
    if gcf > lcf {
        Some((hf2 - gcf, hf1 - gcf, gcf - lcf))
    } else {
        None
    }
}

/// `store_common_region` (`move.c:147`): copy `size` source digit-bytes into the destination
/// field's data region via the scale-aligned [`region_window`]; `verified` selects the raw-copy
/// path (`move.c:172`) vs the digit-cleaning path (`move.c:174-187`).
fn store_common_region(
    dst_full: &mut [u8],
    dst_attr: &FieldAttr,
    data: &[u8],
    size: i64,
    scale: i64,
    verified: bool,
) {
    let off = dst_attr.data_offset();
    let fsize = dst_attr.data_size(dst_full.len()) as i64;

    // pre-set the whole target to '0' (move.c:166)
    for k in 0..fsize {
        wr_off(dst_full, off, k, fsize, b'0');
    }

    if let Some((d0, s0, n)) = region_window(size, scale, fsize, dst_attr.scale as i64) {
        for j in 0..n {
            let didx = d0 + j;
            let sidx = s0 + j;
            let sd = rd(data, sidx);
            if verified {
                wr_off(dst_full, off, didx, fsize, sd);
            } else if sd == b' ' || sd == 0 {
                // already '0' (move.c:185)
            } else {
                wr_off(dst_full, off, didx, fsize, sign::i2d(sd.wrapping_sub(b'0')));
            }
        }
    }
}

/// `cob_move_display_to_display` (`move.c:372`): strip the source sign, store the clean digits,
/// re-apply the sign to the destination. (Upstream also restores `f1`; that is invisible to the
/// destination bytes, so the port works on a copy of the source.)
fn display_to_display(
    src_full: &[u8],
    src_attr: &FieldAttr,
    dst_full: &mut [u8],
    dst_attr: &FieldAttr,
) {
    let mut tmp = src_full.to_vec();
    let s = sign::display_get_sign_strip(&mut tmp, src_attr); // COB_GET_SIGN (move.c:374)
    let s_off = src_attr.data_offset();
    let s_size = src_attr.data_size(src_full.len()) as i64;
    store_common_region(
        dst_full,
        dst_attr,
        tmp.get(s_off..).unwrap_or(&[]), // fail closed on degenerate size < offset
        s_size,
        src_attr.scale as i64,
        false,
    );
    sign::display_put_sign(dst_full, dst_attr, s); // COB_PUT_SIGN (f2) (move.c:380)
}

/// `cob_move_display_to_packed` (`move.c:477`): pack zoned/display digits into COMP-3 nibbles,
/// then set the sign nibble. PACKED fields have no leading-separate offset, so the destination is
/// addressed via the full slice (`f2->data`, `f2->size`), exactly as upstream.
fn display_to_packed(src_full: &[u8], src_attr: &FieldAttr, dst: &mut [u8], dst_attr: &FieldAttr) {
    let s_off = src_attr.data_offset();
    let data1 = src_full.get(s_off..).unwrap_or(&[]); // fail closed on degenerate size < offset
    let s = sign::display_get_sign_adjust_readonly(src_full, src_attr); // COB_GET_SIGN_ADJUST (move.c:480)

    let scale1 = src_attr.scale as i64;
    let scale2 = dst_attr.scale as i64;
    let tnsn = dst_attr.no_sign_nibble();

    let digits1 = if scale1 >= 0 {
        src_attr.digits as i64
    } else {
        src_attr.digits as i64 + scale1 // 99P (move.c:497)
    };
    let digits2 = if scale2 >= 0 {
        dst_attr.digits as i64
    } else {
        dst_attr.digits as i64 + scale2
    };

    let mut i: i64 = if tnsn { digits2 % 2 } else { 1 - digits2 % 2 }; // move.c:501-505

    // p = data1 + (digits1 - scale1) - (digits2 - scale2)  (move.c:511)
    let mut p: i64 = (digits1 - scale1) - (digits2 - scale2);
    while p < 0 {
        p += 1;
        i += 1; // skip not-available positions (move.c:513-515)
    }

    // zero out target (move.c:518)
    for b in dst.iter_mut() {
        *b = 0;
    }

    let i_end: i64 = dst.len() as i64;
    let p_end: i64 = digits1;
    let mut q: i64 = i / 2;

    if i % 2 == 1 {
        wr(dst, q, sign::d2i(rd(data1, p))); // *q++ = COB_D2I(*p++) (move.c:525)
        q += 1;
        p += 1;
        i += 1;
    }
    i /= 2; // i now counts target bytes (move.c:533)

    if i_end - i < (p_end - p + 1) / 2 {
        // truncating loop, bounded by target (move.c:543-550)
        while i < i_end {
            let hi = rd(data1, p) << 4;
            let lo = sign::d2i(rd(data1, p + 1));
            wr(dst, q, hi.wrapping_add(lo));
            q += 1;
            i += 1;
            p += 2;
        }
    } else {
        // consume all source digits (move.c:551-557)
        while p < p_end {
            let hi = rd(data1, p) << 4;
            let lo = sign::d2i(rd(data1, p + 1));
            wr(dst, q, hi.wrapping_add(lo));
            q += 1;
            p += 2;
        }
    }
    if p > p_end {
        // clean bottom nibble of the extra-packed digit (move.c:560)
        wr(dst, q - 1, rd(dst, q - 1) & 0xf0);
    }

    // COB_PUT_SIGN_ADJUSTED is a no-op on an ASCII host (move.c:563).
    if tnsn {
        return; // COMP-6 has no sign nibble (move.c:566)
    }
    let last = dst.len() as i64 - 1;
    let cur = rd(dst, last);
    if !dst_attr.have_sign() {
        wr(dst, last, cur | 0x0F); // move.c:574
    } else if s < 0 {
        wr(dst, last, (cur & 0xF0) | 0x0D); // move.c:576
    } else {
        wr(dst, last, (cur & 0xF0) | 0x0C); // move.c:578
    }
}

/// `cob_move_packed_to_display` (`move.c:582`): unpack COMP-3 / COMP-6 nibbles to display digits
/// (skipping leading zeros exactly as upstream), store, then set the destination sign.
fn packed_to_display(src: &[u8], src_attr: &FieldAttr, dst_full: &mut [u8], dst_attr: &FieldAttr) {
    let mut buff = [0u8; (crate::COB_MAX_DIGITS + 1) as usize];
    let mut b: usize = 0;
    let mut di: i64 = 0;
    let d_end: i64 = src.len() as i64 - 1; // d + f1->size - 1 (move.c:587)
    let scale = src_attr.scale as i64;
    let mut digits = if scale >= 0 {
        src_attr.digits as i64
    } else {
        src_attr.digits as i64 + scale // 99P (move.c:596)
    };

    let push = |buff: &mut [u8], b: &mut usize, v: u8| {
        if *b < buff.len() {
            buff[*b] = v;
            *b += 1;
        }
    };

    if src_attr.no_sign_nibble() {
        // COMP-6 (move.c:604-635)
        let offset = digits % 2;
        if offset == 1 {
            let start = rd(src, di) & 0x0F;
            di += 1;
            if start != 0 {
                push(&mut buff, &mut b, sign::i2d(start));
            } else {
                digits -= 1;
                while di <= d_end && rd(src, di) == 0 {
                    digits -= 2;
                    di += 1;
                }
            }
        } else {
            while di <= d_end && rd(src, di) == 0 {
                digits -= 2;
                di += 1;
            }
        }
        while di <= d_end {
            push(&mut buff, &mut b, sign::i2d(rd(src, di) >> 4));
            push(&mut buff, &mut b, sign::i2d(rd(src, di) & 0x0F));
            di += 1;
        }
        store_common_region(dst_full, dst_attr, &buff[..b], digits, scale, true);
        sign::display_put_sign(dst_full, dst_attr, 0); // COB_PUT_SIGN (f2, 0) (move.c:633)
    } else {
        // PACKED-DECIMAL / COMP-3 (move.c:637-668)
        let offset = 1 - digits % 2;
        if offset == 1 {
            let start = rd(src, di) & 0x0F;
            di += 1;
            if start != 0 {
                push(&mut buff, &mut b, sign::i2d(start));
            } else {
                digits -= 1;
                while di < d_end && rd(src, di) == 0 {
                    digits -= 2;
                    di += 1;
                }
            }
        } else {
            while di < d_end && rd(src, di) == 0 {
                digits -= 2;
                di += 1;
            }
        }
        while di < d_end {
            push(&mut buff, &mut b, sign::i2d(rd(src, di) >> 4));
            push(&mut buff, &mut b, sign::i2d(rd(src, di) & 0x0F));
            di += 1;
        }
        push(&mut buff, &mut b, sign::i2d(rd(src, di) >> 4)); // last high nibble (move.c:663)
        store_common_region(dst_full, dst_attr, &buff[..b], digits, scale, true);
        let s = if (rd(src, di) & 0x0F) == 0x0D { -1 } else { 1 }; // (move.c:666)
        sign::display_put_sign(dst_full, dst_attr, s);
    }
}

/// `cob_move` (`move.c:1446`), restricted to the sealed elementary type pairs. Any other pair
/// **fails closed** with [`DecimalError::UnsupportedConversion`] rather than guessing.
///
/// `src` / `dst` are the full field byte storages; the field "size" is the slice length (as
/// `cob_field.size` in upstream). `dst` is zeroed/overwritten in place.
pub fn cob_move(
    src: &[u8],
    src_attr: &FieldAttr,
    dst: &mut [u8],
    dst_attr: &FieldAttr,
) -> Result<(), DecimalError> {
    if dst.is_empty() {
        return Ok(()); // dst->size == 0 (move.c:1455)
    }
    match (src_attr.field_type, dst_attr.field_type) {
        (COB_TYPE_NUMERIC_DISPLAY, COB_TYPE_NUMERIC_DISPLAY) => {
            display_to_display(src, src_attr, dst, dst_attr);
            Ok(())
        }
        (COB_TYPE_NUMERIC_DISPLAY, COB_TYPE_NUMERIC_PACKED) => {
            display_to_packed(src, src_attr, dst, dst_attr);
            Ok(())
        }
        (COB_TYPE_NUMERIC_PACKED, COB_TYPE_NUMERIC_DISPLAY) => {
            packed_to_display(src, src_attr, dst, dst_attr);
            Ok(())
        }
        _ => Err(DecimalError::UnsupportedConversion {
            src_type: src_attr.field_type,
            dst_type: dst_attr.field_type,
        }),
    }
}

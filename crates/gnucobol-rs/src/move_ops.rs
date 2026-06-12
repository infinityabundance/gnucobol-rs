//! Faithful port of the GnuCOBOL `MOVE` byte conversions for the sealed type pairs:
//! DISPLAY→DISPLAY, DISPLAY→PACKED (COMP-3 encode), PACKED→DISPLAY (COMP-3 decode).
//!
//! Citations: `cob_move` dispatch (`move.c:1446`), `store_common_region` (`move.c:147`),
//! `cob_move_display_to_display` (`move.c:372`), `cob_move_display_to_packed` (`move.c:477`),
//! `cob_move_packed_to_display` (`move.c:582`). Control flow mirrors upstream, including its
//! one-byte-past read in the packing loop (cleaned by the trailing `&= 0xf0`, faithfully
//! reproduced) — here every such read is **bounds-guarded** to a `0`, which yields the identical
//! result because that nibble is always cleaned. Evidence kind: **source_port**.

use crate::attr::{
    FieldAttr, COB_TYPE_ALPHANUMERIC, COB_TYPE_GROUP, COB_TYPE_NUMERIC_DISPLAY, COB_TYPE_NUMERIC_PACKED,
};
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
fn cob_move_display_to_display(
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
fn cob_move_display_to_packed(src_full: &[u8], src_attr: &FieldAttr, dst: &mut [u8], dst_attr: &FieldAttr) {
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
fn cob_move_packed_to_display(src: &[u8], src_attr: &FieldAttr, dst_full: &mut [u8], dst_attr: &FieldAttr) {
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

/// `cob_move_alphanum_to_alphanum` (`move.c:444`): the classic string MOVE — left-justified by
/// default (truncate/space-pad on the right), or `JUSTIFIED RIGHT` (truncate/space-pad on the left).
fn cob_move_alphanum_to_alphanum(data1: &[u8], dst: &mut [u8], dst_attr: &FieldAttr) {
    let size1 = data1.len();
    let size2 = dst.len();
    if size1 >= size2 {
        if dst_attr.justified() {
            dst.copy_from_slice(&data1[size1 - size2..]);
        } else {
            dst.copy_from_slice(&data1[..size2]);
        }
    } else if dst_attr.justified() {
        let pad = size2 - size1;
        for b in &mut dst[..pad] {
            *b = b' ';
        }
        dst[pad..].copy_from_slice(data1);
    } else {
        dst[..size1].copy_from_slice(data1);
        for b in &mut dst[size1..] {
            *b = b' ';
        }
    }
}

/// `cob_move_display_to_alphanum` (`move.c:384`): numeric DISPLAY -> alphanumeric. The overpunch sign
/// is cleaned to a plain digit (`COB_GET_SIGN`), implied `P` positions become `'0'`, and the result is
/// left- or right-justified with space padding. (`f1`'s sign is restored in C; invisible to `dst`.)
fn cob_move_display_to_alphanum(src_full: &[u8], src_attr: &FieldAttr, dst: &mut [u8], dst_attr: &FieldAttr) {
    let mut tmp = src_full.to_vec();
    let _ = sign::display_get_sign_strip(&mut tmp, src_attr); // COB_GET_SIGN: de-overpunch the digits
    let off = src_attr.data_offset();
    let size1 = src_attr.data_size(src_full.len());
    let data1 = tmp.get(off..off + size1).unwrap_or(&[]);
    let size1 = data1.len();
    let mut size2 = dst.len();
    let zero_size = if (src_attr.scale as i32) < 0 { (-(src_attr.scale as i32)) as usize } else { 0 };

    if dst_attr.justified() {
        let mut zs = zero_size;
        if zs != 0 {
            zs = zs.min(size2);
            size2 -= zs;
            for b in &mut dst[size2..size2 + zs] {
                *b = b'0';
            }
        }
        if size2 != 0 {
            let diff = size2 as i64 - size1 as i64;
            let (d2, s2) = if diff > 0 {
                for b in &mut dst[..diff as usize] {
                    *b = b' ';
                }
                (diff as usize, size2 - diff as usize)
            } else {
                (0usize, size2)
            };
            dst[d2..d2 + s2].copy_from_slice(&data1[size1 - s2..size1]);
        }
    } else {
        let diff = size2 as i64 - size1 as i64;
        if diff < 0 {
            dst[..size2].copy_from_slice(&data1[..size2]);
        } else {
            dst[..size1].copy_from_slice(data1);
            let mut diff = diff as usize;
            let mut zs = 0usize;
            if zero_size != 0 {
                zs = zero_size.min(diff);
                for b in &mut dst[size1..size1 + zs] {
                    *b = b'0';
                }
                diff -= zs;
            }
            if diff != 0 {
                for b in &mut dst[size1 + zs..size1 + zs + diff] {
                    *b = b' ';
                }
            }
        }
    }
}

/// `cob_move_alphanum_to_display` (`move.c:293`): parse free-form alphanumeric text into a numeric
/// DISPLAY field — skip leading whitespace, optional `+`/`-`, count integer digits, right-align by
/// the receiver scale, copy digits (ignoring whitespace and the thousands separator), and on any
/// invalid character zero-fill the receiver. `dec_pt`/`num_sep` default to `'.'`/`','`.
fn cob_move_alphanum_to_display(data1: &[u8], dst: &mut [u8], dst_attr: &FieldAttr) {
    const DEC_PT: u8 = b'.';
    const NUM_SEP: u8 = b',';
    let off = dst_attr.data_offset();
    let fsize = dst_attr.data_size(dst.len());
    // memset(f2->data, '0', f2->size)
    for b in dst.iter_mut() {
        *b = b'0';
    }
    let e1 = data1.len();
    let mut s1 = 0usize;
    // skip whitespace
    while s1 < e1 && data1[s1].is_ascii_whitespace() {
        s1 += 1;
    }
    // sign
    let mut sign = 0i32;
    if s1 != e1 && (data1[s1] == b'+' || data1[s1] == b'-') {
        sign = if data1[s1] == b'+' { 1 } else { -1 };
        s1 += 1;
    }
    // count integer digits (before decimal point)
    let mut count = 0i64;
    {
        let mut p = s1;
        while p < e1 && data1[p] != DEC_PT {
            if data1[p].is_ascii_digit() {
                count += 1;
            }
            p += 1;
        }
    }
    // destination digit window [off .. off+fsize); s2 walks it
    let mut s2 = off;
    let e2 = off + fsize;
    let size = fsize as i64 - dst_attr.scale as i64;
    if count < size {
        s2 += (size - count) as usize;
    } else {
        let mut c = count;
        while c > size {
            c -= 1;
            while s1 < e1 && !data1[s1].is_ascii_digit() {
                s1 += 1;
            }
            s1 += 1;
        }
    }
    // move
    let mut dpcount = 0i32;
    let mut error = false;
    while s1 < e1 && s2 < e2 {
        let ch = data1[s1];
        if ch.is_ascii_digit() {
            wr(dst, s2 as i64, ch);
            s2 += 1;
        } else if ch == DEC_PT {
            dpcount += 1;
            if dpcount > 1 {
                error = true;
                break;
            }
        } else if !(ch.is_ascii_whitespace() || ch == NUM_SEP) {
            error = true;
            break;
        }
        s1 += 1;
    }
    if error {
        for b in dst.iter_mut() {
            *b = b'0';
        }
        sign::display_put_sign(dst, dst_attr, 0);
        return;
    }
    sign::display_put_sign(dst, dst_attr, sign);
}

/// `cob_move_ibm (dst, src, len)` (move.c:1411): copy `len` bytes left-to-right (the IBM `MVC`
/// instruction). For non-aliasing Rust slices this is a plain copy.
pub fn cob_move_ibm(dst: &mut [u8], src: &[u8], len: usize) {
    dst[..len].copy_from_slice(&src[..len]);
}

/// `cob_init_table (tbl, len, occ)` (move.c:1426): propagate the first `len`-byte element through an
/// `occ`-occurrence table by doubling copies (used by `INITIALIZE`). `tbl` is the full `len*occ` buffer
/// with the seed element in `tbl[..len]`.
pub fn cob_init_table(tbl: &mut [u8], len: usize, occ: usize) {
    if occ < 1 {
        return;
    }
    let mut m = len;
    let mut i = len;
    let mut j = 1usize;
    loop {
        j *= 2;
        let (front, rest) = tbl.split_at_mut(m);
        rest[..i].copy_from_slice(&front[..i]);
        m += i;
        i *= 2;
        if j * 2 >= occ {
            break;
        }
    }
    if j < occ {
        let copy = len * (occ - j);
        let (front, rest) = tbl.split_at_mut(m);
        rest[..copy].copy_from_slice(&front[..copy]);
    }
}

/// `cob_move_all (src, dst)` (move.c:1356): the figurative `MOVE ALL pattern` — fill `dst` by repeating
/// the source bytes. Alphanumeric/edited receivers fill directly; numeric receivers fill a display temp
/// and re-`cob_move`. `src` is the (1+ byte) pattern.
fn cob_move_all(src: &[u8], dst: &mut [u8], dst_attr: &FieldAttr) -> Result<(), DecimalError> {
    use crate::attr::{COB_TYPE_ALPHANUMERIC, COB_TYPE_ALPHANUMERIC_EDITED, COB_TYPE_NUMERIC_BINARY};
    let ft = dst_attr.field_type;
    let is_alnum = matches!(ft, COB_TYPE_ALPHANUMERIC | COB_TYPE_ALPHANUMERIC_EDITED | COB_TYPE_GROUP | crate::attr::COB_TYPE_ALPHANUMERIC_ALL);
    if is_alnum {
        if src.len() == 1 {
            for b in dst.iter_mut() {
                *b = src[0];
            }
        } else {
            for i in 0..dst.len() {
                dst[i] = src[i % src.len()];
            }
        }
        return Ok(());
    }
    let is_numeric = matches!(ft, COB_TYPE_NUMERIC_DISPLAY | COB_TYPE_NUMERIC_PACKED | COB_TYPE_NUMERIC_BINARY | 0x13..=0x1B);
    let max = crate::COB_MAX_DIGITS as usize;
    if !is_numeric {
        // numeric-edited: MOVE ALL is an alphanumeric fill into a display temp, then move
        let digcount = dst.len();
        let mut p = vec![0u8; digcount];
        fill(&mut p, src);
        let tattr = FieldAttr { field_type: COB_TYPE_ALPHANUMERIC, digits: digcount as u16, scale: 0, flags: 0 };
        return cob_move(&p, &tattr, dst, dst_attr);
    }
    let digcount = if src.len() == 1 { max } else { max };
    let mut p = vec![0u8; digcount];
    fill(&mut p, src);
    let tattr = FieldAttr { field_type: COB_TYPE_NUMERIC_DISPLAY, digits: digcount as u16, scale: 0, flags: crate::attr::COB_FLAG_HAVE_SIGN };
    cob_move(&p, &tattr, dst, dst_attr)
}

fn fill(buf: &mut [u8], src: &[u8]) {
    if src.len() == 1 {
        for b in buf.iter_mut() {
            *b = src[0];
        }
    } else {
        for i in 0..buf.len() {
            buf[i] = src[i % src.len()];
        }
    }
}

/// `indirect_move (func, src, dst, size, scale)` (move.c:1332): route `src` through a `size`/`scale`
/// signed-DISPLAY intermediate (produced by `func`) before `cob_move`-ing into `dst` — used when a
/// non-display source must be edited/stored via the display path.
#[allow(dead_code)] // wired by the edited-MOVE batch (display_to_edited / edited_to_display routing)
fn indirect_move<F>(func: F, src: &[u8], sa: &FieldAttr, dst: &mut [u8], da: &FieldAttr, size: usize, scale: i32) -> Result<(), DecimalError>
where
    F: FnOnce(&[u8], &FieldAttr, &mut [u8], &FieldAttr),
{
    let mut buff = vec![0u8; size];
    let tattr = FieldAttr { field_type: COB_TYPE_NUMERIC_DISPLAY, digits: size as u16, scale: scale as i16, flags: crate::attr::COB_FLAG_HAVE_SIGN };
    func(src, sa, &mut buff, &tattr);
    cob_move(&buff, &tattr, dst, da)
}

/// `cob_binary_mget_sint64 (f)` (move.c:196): read a BINARY field as a signed 64-bit integer (endian
/// from `COB_FLAG_BINARY_SWAP`, sign-extended for signed fields). Equals [`crate::binary::binary_decode`].
pub fn cob_binary_mget_sint64(data: &[u8], attr: &FieldAttr) -> i64 {
    crate::binary::binary_decode(data, attr) as i64
}

/// `cob_binary_mget_uint64 (f)` (move.c:231): read a BINARY field as an unsigned 64-bit integer
/// (no sign extension), honoring `COB_FLAG_BINARY_SWAP`.
pub fn cob_binary_mget_uint64(data: &[u8], attr: &FieldAttr) -> u64 {
    let n = data.len().min(8);
    let swap = attr.flags & crate::attr::COB_FLAG_BINARY_SWAP != 0;
    let mut u: u64 = 0;
    if swap {
        for &b in data.iter().take(n) {
            u = (u << 8) | b as u64;
        }
    } else {
        for &b in data.iter().take(n).rev() {
            u = (u << 8) | b as u64;
        }
    }
    u
}

/// `cob_binary_mset_sint64 (f, n)` (move.c:254): write the low `f.size` bytes of `n` into a BINARY
/// field with the field's endianness (a raw store — no PIC-digit truncation).
pub fn cob_binary_mset_sint64(out: &mut [u8], attr: &FieldAttr, n: i64) {
    let size = out.len().min(8);
    let swap = attr.flags & crate::attr::COB_FLAG_BINARY_SWAP != 0;
    if swap {
        out[..size].copy_from_slice(&n.to_be_bytes()[8 - size..]);
    } else {
        out[..size].copy_from_slice(&n.to_le_bytes()[..size]);
    }
}

/// `cob_binary_mset_uint64 (f, n)` (move.c:284): unsigned form of [`cob_binary_mset_sint64`].
pub fn cob_binary_mset_uint64(out: &mut [u8], attr: &FieldAttr, n: u64) {
    cob_binary_mset_sint64(out, attr, n as i64);
}

fn pow10_ll(n: i32) -> i64 {
    10i64.checked_pow(n.max(0) as u32).unwrap_or(i64::MAX)
}

fn bin_trunc_digits(attr: &FieldAttr) -> i32 {
    let scale = attr.scale as i32;
    if scale >= 0 {
        attr.digits as i32
    } else {
        attr.digits as i32 + scale
    }
}

/// `cob_move_binary_to_binary (f1, f2)` (move.c:709): binary -> binary, value-preserving (scale
/// ignored), with optional PIC-digit truncation on the receiver.
fn cob_move_binary_to_binary(s: &[u8], sa: &FieldAttr, d: &mut [u8], da: &FieldAttr) {
    let trunc = da.flags & crate::attr::COB_FLAG_BINARY_TRUNC != 0;
    if sa.have_sign() {
        let mut sval = cob_binary_mget_sint64(s, sa);
        let sign = sval < 0;
        if trunc {
            sval %= pow10_ll(bin_trunc_digits(da));
        }
        if da.have_sign() {
            cob_binary_mset_sint64(d, da, sval);
        } else if sign {
            cob_binary_mset_uint64(d, da, (sval as i128).unsigned_abs() as u64);
        } else {
            cob_binary_mset_uint64(d, da, sval as u64);
        }
    } else {
        let mut uval = cob_binary_mget_uint64(s, sa);
        if trunc {
            uval %= pow10_ll(bin_trunc_digits(da)) as u64;
        }
        if da.have_sign() {
            cob_binary_mset_sint64(d, da, uval as i64);
        } else {
            cob_binary_mset_uint64(d, da, uval);
        }
    }
}

/// `cob_move_display_to_binary (f1, f2)` (move.c:763): parse the zoned digits (scale-aligned to the
/// receiver) into an integer, then store it in the binary receiver. Overflow (>19 digits) routes
/// through the decimal layer.
fn cob_move_display_to_binary(s: &[u8], sa: &FieldAttr, d: &mut [u8], da: &FieldAttr) -> Result<(), DecimalError> {
    let off = sa.data_offset();
    let size1 = sa.data_size(s.len());
    let data1 = s.get(off..off + size1).unwrap_or(&[]);
    let size = (size1 as i64 - sa.scale as i64 + da.scale as i64).max(0) as usize;
    let trunc = da.flags & crate::attr::COB_FLAG_BINARY_TRUNC != 0;
    let mut target_digits: i64 = if trunc {
        let scale = da.scale as i32;
        let digits = da.digits as i32;
        let t = if scale > digits {
            digits
        } else if scale > 0 {
            digits
        } else {
            digits + scale
        };
        (t as i64).min(size as i64)
    } else {
        size as i64
    };
    let mut i = (size as i64 - target_digits).max(0) as usize;
    while i < size1 {
        if sign::d2i(data1[i]) != 0 {
            break;
        }
        target_digits -= 1;
        i += 1;
    }
    if target_digits > 19 {
        // overflow -> decimal layer
        let bytes = crate::cob_decimal::cob_decimal_setget_fld(s, sa, da, d.len(), crate::arith::Round::Truncate).map_err(|_| DecimalError::UnsupportedConversion { src_type: sa.field_type, dst_type: da.field_type })?;
        d.copy_from_slice(&bytes);
        return Ok(());
    }
    let mut tmp = s.to_vec();
    let sgn = sign::display_get_sign_strip(&mut tmp, sa); // COB_GET_SIGN_ADJUST (value re-read below)
    let data1 = tmp.get(off..off + size1).unwrap_or(&[]);
    let mut val: u64 = 0;
    while i < size {
        if i < size1 {
            val = val * 10 + sign::d2i(data1[i]) as u64;
        } else {
            val *= 10;
        }
        i += 1;
    }
    if da.have_sign() {
        let v2 = if sgn < 0 { -(val as i64) } else { val as i64 };
        cob_binary_mset_sint64(d, da, v2);
    } else {
        cob_binary_mset_uint64(d, da, val);
    }
    Ok(())
}

/// `cob_move_binary_to_display (f1, f2)` (move.c:836): decode the binary value to a digit string and
/// store it (scale-aligned) into the display receiver, with sign.
fn cob_move_binary_to_display(s: &[u8], sa: &FieldAttr, d: &mut [u8], da: &FieldAttr) {
    let mut sign = 1i32;
    let val: u64 = if sa.have_sign() {
        let v2 = cob_binary_mget_sint64(s, sa);
        if v2 < 0 {
            sign = -1;
            (v2 as i128).unsigned_abs() as u64
        } else {
            v2 as u64
        }
    } else {
        cob_binary_mget_uint64(s, sa)
    };
    let mut buff = [0u8; 32];
    let mut i = 20usize;
    let mut v = val;
    while v > 0 {
        i -= 1;
        buff[i] = sign::i2d((v % 10) as u8);
        v /= 10;
    }
    store_common_region(d, da, &buff[i..20], (20 - i) as i64, sa.scale as i64, true);
    sign::display_put_sign(d, da, sign);
}

/// `cob_move_fp_to_fp (src, dst)` (move.c:673): convert between FLOAT (f32) / DOUBLE (f64) /
/// L_DOUBLE (x87 80-bit) by widening to the largest then narrowing to the target.
fn cob_move_fp_to_fp(s: &[u8], sa: &FieldAttr, d: &mut [u8], da: &FieldAttr) {
    let dval: f64 = match sa.field_type {
        0x13 => f32::from_le_bytes(s[..4].try_into().unwrap_or([0; 4])) as f64,
        0x14 => f64::from_le_bytes(s[..8].try_into().unwrap_or([0; 8])),
        _ => crate::cob_decimal::extended80_to_f64(s),
    };
    match da.field_type {
        0x13 => d[..4].copy_from_slice(&(dval as f32).to_le_bytes()),
        0x14 => d[..8].copy_from_slice(&dval.to_le_bytes()),
        _ => {
            let e = crate::cob_decimal::f64_to_extended80(dval);
            let n = d.len().min(e.len());
            d[..n].copy_from_slice(&e[..n]);
        }
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
    // Figurative `ALL` source (move.c:1469): repeat the pattern across the receiver.
    if src_attr.field_type == crate::attr::COB_TYPE_ALPHANUMERIC_ALL {
        return cob_move_all(src, dst, dst_attr);
    }
    // Non-elementary move: a GROUP on either side is a raw alphanumeric copy (move.c:1473).
    if src_attr.field_type == COB_TYPE_GROUP || dst_attr.field_type == COB_TYPE_GROUP {
        cob_move_alphanum_to_alphanum(src, dst, dst_attr);
        return Ok(());
    }
    use crate::attr::COB_TYPE_NUMERIC_BINARY;
    let src_bin = src_attr.field_type == COB_TYPE_NUMERIC_BINARY;
    let dst_bin = dst_attr.field_type == COB_TYPE_NUMERIC_BINARY;

    // Binary MOVE leaves (move.c): the named per-pair routines. DISPLAY<->BINARY and BINARY<->BINARY
    // are direct; BINARY<->PACKED (and other numeric pairs) route through the decimal layer, as the
    // C dispatch does via cob_decimal_setget_fld.
    if src_bin && dst_bin {
        cob_move_binary_to_binary(src, src_attr, dst, dst_attr);
        return Ok(());
    }
    if dst_bin {
        if src_attr.field_type == COB_TYPE_NUMERIC_DISPLAY {
            return cob_move_display_to_binary(src, src_attr, dst, dst_attr);
        }
        let bytes = crate::cob_decimal::cob_decimal_setget_fld(src, src_attr, dst_attr, dst.len(), crate::arith::Round::Truncate)
            .map_err(|_| DecimalError::UnsupportedConversion { src_type: src_attr.field_type, dst_type: dst_attr.field_type })?;
        dst.copy_from_slice(&bytes);
        return Ok(());
    }
    if src_bin {
        if dst_attr.field_type == COB_TYPE_NUMERIC_DISPLAY {
            cob_move_binary_to_display(src, src_attr, dst, dst_attr);
            return Ok(());
        }
        let bytes = crate::cob_decimal::cob_decimal_setget_fld(src, src_attr, dst_attr, dst.len(), crate::arith::Round::Truncate)
            .map_err(|_| DecimalError::UnsupportedConversion { src_type: src_attr.field_type, dst_type: dst_attr.field_type })?;
        dst.copy_from_slice(&bytes);
        return Ok(());
    }

    // Floating-point leaves (move.c): float<->float widen/narrow; float<->decimal via the decimal layer.
    let is_fp = |a: &FieldAttr| matches!(a.field_type, 0x13 | 0x14 | 0x15);
    if is_fp(src_attr) && is_fp(dst_attr) {
        cob_move_fp_to_fp(src, src_attr, dst, dst_attr);
        return Ok(());
    }
    if is_fp(dst_attr) {
        // encode: src -> decimal -> the float receiver (no recursion through cob_move)
        let d = crate::cob_decimal::cob_decimal_set_field(src, src_attr);
        let v = crate::cob_decimal::cob_decimal_get_double(&d);
        match dst_attr.field_type {
            0x13 => dst[..4].copy_from_slice(&(v as f32).to_le_bytes()),
            0x14 => dst[..8].copy_from_slice(&v.to_le_bytes()),
            _ => {
                let e = crate::cob_decimal::f64_to_extended80(v);
                let n = dst.len().min(10);
                dst[..n].copy_from_slice(&e[..n]);
            }
        }
        return Ok(());
    }
    if is_fp(src_attr) {
        let bytes = crate::cob_decimal::cob_decimal_setget_fld(src, src_attr, dst_attr, dst.len(), crate::arith::Round::Truncate)
            .map_err(|_| DecimalError::UnsupportedConversion { src_type: src_attr.field_type, dst_type: dst_attr.field_type })?;
        dst.copy_from_slice(&bytes);
        return Ok(());
    }

    match (src_attr.field_type, dst_attr.field_type) {
        (COB_TYPE_NUMERIC_DISPLAY, COB_TYPE_NUMERIC_DISPLAY) => {
            cob_move_display_to_display(src, src_attr, dst, dst_attr);
            Ok(())
        }
        (COB_TYPE_NUMERIC_DISPLAY, COB_TYPE_NUMERIC_PACKED) => {
            cob_move_display_to_packed(src, src_attr, dst, dst_attr);
            Ok(())
        }
        (COB_TYPE_NUMERIC_PACKED, COB_TYPE_NUMERIC_DISPLAY) => {
            cob_move_packed_to_display(src, src_attr, dst, dst_attr);
            Ok(())
        }
        // Alphanumeric leaves (move.c default src/dst dispatch).
        (COB_TYPE_NUMERIC_DISPLAY, COB_TYPE_ALPHANUMERIC) => {
            cob_move_display_to_alphanum(src, src_attr, dst, dst_attr);
            Ok(())
        }
        (COB_TYPE_ALPHANUMERIC, COB_TYPE_NUMERIC_DISPLAY) => {
            cob_move_alphanum_to_display(src, dst, dst_attr);
            Ok(())
        }
        (COB_TYPE_ALPHANUMERIC, COB_TYPE_ALPHANUMERIC) => {
            cob_move_alphanum_to_alphanum(src, dst, dst_attr);
            Ok(())
        }
        _ => Err(DecimalError::UnsupportedConversion {
            src_type: src_attr.field_type,
            dst_type: dst_attr.field_type,
        }),
    }
}




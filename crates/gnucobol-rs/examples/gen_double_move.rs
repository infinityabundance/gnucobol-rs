//! Generator for the float->decimal differential (numeric.c cob_decimal_set_double + the mpf path):
//! MOVE COMP-2 (IEEE double) to DISPLAY fields of varied digits/scales. Operand bytes are the f64's
//! little-endian image so identical bytes feed the libcob oracle (cob_move) and the Rust mirror.
//!
//! Row: `label s_type s_dig s_scale s_flags s_size s_hex d_type d_dig d_scale d_flags d_size`

const DISPLAY: u32 = 0x10;
const DOUBLE: u32 = 0x14;
const HAVE_SIGN: u32 = 1;

fn hex(b: &[u8]) -> String {
    b.iter().map(|x| format!("{x:02x}")).collect()
}

fn main() {
    // A spread that stresses ordinary values, ties, large/small magnitudes, and many-significant-digit
    // doubles (where the 96-digit mpf_get_str rounding could in principle bite).
    let vals: &[f64] = &[
        0.0, 1.0, -1.0, 2.0, 0.5, 0.25, 0.1, 0.2, 0.3, 0.7, 2.5, -2.5, 123.45, -123.45, 999.999,
        3.141592653589793, 2.718281828459045, 1234.5, 0.001, -0.001, 12345.678, 87654.321,
        1e6, 1e7, 1e8, 1.5e9, 1e-3, 1e-6, 9.99999e7, 1.0 / 3.0, 2.0 / 3.0, 10.0 / 3.0, 100.0 / 7.0,
        0.123456789012345, 9876543.21, 0.000123456789, 4294967296.0, 65536.5, 16777217.0,
        99999999.99, -99999999.99, 0.049999999999, 0.05, 0.15, 0.25, 0.35, 0.45, 1.005, 2.675,
    ];
    // (digits, scale, signed) display receivers
    let shapes: &[(u32, i32, bool)] = &[
        (9, 0, true),
        (9, 4, true),
        (18, 2, true),
        (18, 6, true),
        (8, 2, false),
        (15, 3, true),
        (6, 0, true),
        (12, 9, true),
    ];
    let mut id = 0u64;
    // forward: COMP-2 -> DISPLAY (exercises cob_decimal_set_double)
    for &v in vals {
        let bytes = v.to_le_bytes();
        for &(dig, scale, signed) in shapes {
            let flags = if signed { HAVE_SIGN } else { 0 };
            println!(
                "x{id} {DOUBLE} 0 0 0 8 {} {DISPLAY} {dig} {scale} {flags} {dig}",
                hex(&bytes),
            );
            id += 1;
        }
    }
    // NOTE: the reverse DISPLAY->COMP-2 leg is intentionally NOT generated here. Through this harness
    // cob_move(DISPLAY->COMP-2) takes a non-get_double path that drops the fractional part (a PIC/path
    // artifact, not cob_decimal_get_double). The decimal->double direction is verified directly against
    // the FLOAT.1-sealed decimal_to_f64_trunc by the `get_double_via_mpf_matches_sealed_primitive` unit
    // test (bit-exact), so it does not need an oracle leg here.
}

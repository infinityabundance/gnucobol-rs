//! Calls the `extern "C"` entry points directly (via the rlib) and checks the C-ABI shim round-trips a
//! `cob_field` exactly as libcob would. The C-vs-libcob gold test is `verify_vs_libcob.sh`.
use gnucobol_rs_ffi::{cob_get_int, cob_move, cob_set_int, CobField, CobFieldAttr};

const DISPLAY: u16 = 0x10;
const PACKED: u16 = 0x12;

#[test]
fn move_display_to_comp3_then_get_int() {
    let mut sd = *b"1234";
    let sa = CobFieldAttr { type_: DISPLAY, digits: 4, scale: 0, flags: 0, pic: std::ptr::null() };
    let src = CobField { size: 4, data: sd.as_mut_ptr(), attr: &sa };
    let mut dd = [0u8; 3];
    let da = CobFieldAttr { type_: PACKED, digits: 4, scale: 0, flags: 0, pic: std::ptr::null() };
    let dst = CobField { size: 3, data: dd.as_mut_ptr(), attr: &da };
    unsafe { cob_move(&src, &dst as *const _ as *mut _) };
    assert_eq!(dd, [0x01, 0x23, 0x4f]); // packed 1234, unsigned sign nibble 0xF
    assert_eq!(unsafe { cob_get_int(&dst) }, 1234);
}

#[test]
fn set_int_into_display() {
    let mut xd = [0u8; 4];
    let xa = CobFieldAttr { type_: DISPLAY, digits: 4, scale: 0, flags: 0, pic: std::ptr::null() };
    let x = CobField { size: 4, data: xd.as_mut_ptr(), attr: &xa };
    unsafe { cob_set_int(&x as *const _ as *mut _, 5678) };
    assert_eq!(&xd, b"5678");
}

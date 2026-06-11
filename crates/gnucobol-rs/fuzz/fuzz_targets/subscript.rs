#![no_main]
//! Fuzz table subscript access (`GNURUST.SUBSCRIPT.1`).
//! FUZZFOR: GNURUST.SUBSCRIPT.1
//! panic-freedom — bounded element access never reads out of range.
use libfuzzer_sys::fuzz_target;
fuzz_target!(|data: &[u8]| {
    gnucobol_rs::subscript::__fuzz_subscript(data);
});

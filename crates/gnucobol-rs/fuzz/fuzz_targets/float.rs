#![no_main]
//! Fuzz float-field conversions (COMP-1/COMP-2/FLOAT-DECIMAL-16/34, both directions).
//! FUZZFOR: GNURUST.FLOAT.1
//! Panic-freedom: every conversion is total over arbitrary (mag, scale) and arbitrary field bytes.
use libfuzzer_sys::fuzz_target;
fuzz_target!(|data: &[u8]| {
    gnucobol_rs::float::__fuzz_float(data);
});

#![no_main]
//! Fuzz bit-logical ops (B-AND/B-OR/B-XOR/B-NOT + shifts).
//! FUZZFOR: GNURUST.LOGICAL.1
//! Panic-freedom: every op is total over arbitrary operand pairs (incl. shift counts >= 64).
use libfuzzer_sys::fuzz_target;
fuzz_target!(|data: &[u8]| {
    gnucobol_rs::logical::__fuzz_logical(data);
});

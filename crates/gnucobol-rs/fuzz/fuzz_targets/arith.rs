#![no_main]
//! Fuzz decimal arithmetic: arbitrary operands/attrs/op. Asserts only panic-freedom
//! (`GNURUST.PANICPOLICY.0`): hostile inputs yield a typed `ArithError`, never a panic/overflow.

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    gnucobol_rs::__fuzz_arith(data);
});

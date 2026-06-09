#![no_main]
//! Fuzz the `intrinsic` court: arbitrary bytes as input. The assertion is panic-freedom
//! FUZZFOR: GNURUST.INTRINSIC.LENGTH.1, GNURUST.INTRINSIC.NUMVAL.1, GNURUST.INTRINSIC.NUMVAL-C.1, GNURUST.INTRINSIC.INTEGER.1, GNURUST.INTRINSIC.MOD-REM.1, GNURUST.INTRINSIC.CASE.1, GNURUST.INTRINSIC.ORD-CHAR.1, GNURUST.INTRINSIC.DATE.1
//! (`GNURUST.PANICPOLICY.0`) -- any hostile/malformed input yields a typed result or a value, never a panic.

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    gnucobol_rs::__fuzz_intrinsic(data);
});

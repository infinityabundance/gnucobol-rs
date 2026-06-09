#![no_main]
//! Fuzz the `value` court: arbitrary bytes. Panic-freedom (`GNURUST.PANICPOLICY.0`).
//! FUZZFOR: GNURUST.14, GNURUST.17, GNURUST.18

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| { gnucobol_rs::__fuzz_value(data); });

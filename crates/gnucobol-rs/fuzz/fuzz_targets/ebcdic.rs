#![no_main]
//! Fuzz the `ebcdic` court: arbitrary bytes. Panic-freedom (`GNURUST.PANICPOLICY.0`).
//! FUZZFOR: GNURUST.15

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| { gnucobol_rs::__fuzz_ebcdic(data); });

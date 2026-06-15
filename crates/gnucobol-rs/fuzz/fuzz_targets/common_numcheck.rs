#![no_main]
//! Fuzz the not-numeric diagnostic generators: arbitrary type codes + data never panic.
//! FUZZFOR: GNURUST.COMMON.NUMCHECK.1

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    gnucobol_rs::__fuzz_common_numcheck(data);
});

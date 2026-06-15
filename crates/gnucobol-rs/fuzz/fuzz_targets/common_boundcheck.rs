#![no_main]
//! Fuzz the runtime bounds-check message generators: arbitrary indices/sizes never panic and produce
//! well-formed messages.
//! FUZZFOR: GNURUST.COMMON.BOUNDCHECK.1

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    gnucobol_rs::__fuzz_common_boundcheck(data);
});

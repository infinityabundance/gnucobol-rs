#![no_main]
//! Fuzz the VALUE initial-image court: arbitrary record specs. Asserts only panic-freedom
//! (`GNURUST.PANICPOLICY.0`): hostile PICs/literals yield a typed `InitError`, never a panic.

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    gnucobol_rs::__fuzz_init(data);
});

#![no_main]
//! Fuzz the copybook expander: arbitrary main source + copybooks (separated by 0x01). Asserts only
//! FUZZFOR: GNURUST.5, GNURUST.6
//! panic-freedom (`GNURUST.PANICPOLICY.0`): cycles/missing/deep nesting yield a typed `CopyError`.

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    gnucobol_rs::__fuzz_copybook(data);
});

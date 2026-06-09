#![no_main]
//! Fuzz the `inspect` court: arbitrary bytes as input. The assertion is panic-freedom
//! FUZZFOR: GNURUST.INSPECT.1
//! (`GNURUST.PANICPOLICY.0`) -- any hostile/malformed input yields a typed result or a value, never a panic.

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    gnucobol_rs::__fuzz_inspect(data);
});

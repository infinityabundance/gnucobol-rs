#![no_main]
//! Fuzz the CBL_ logic/bit builtins: arbitrary buffers + lengths never panic.
//! FUZZFOR: GNURUST.COMMON.CBL.1

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    gnucobol_rs::__fuzz_common_cbl(data);
});

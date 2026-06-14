#![no_main]
//! Fuzz the native XML/JSON GENERATE serializer: arbitrary trees serialize without panic.
//! FUZZFOR: GNURUST.MLIO.GENERATE.1
//! (`GNURUST.PANICPOLICY.0`) -- any hostile/malformed ml-tree yields valid bytes, never a panic.

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    gnucobol_rs::__fuzz_mlio(data);
});

#![no_main]
//! Fuzz class-condition predicates (`GNURUST.CLASS.1`): arbitrary bytes.
//! FUZZFOR: GNURUST.CLASS.1
//! panic-freedom — the predicates are total over any bytes.
use libfuzzer_sys::fuzz_target;
fuzz_target!(|data: &[u8]| {
    gnucobol_rs::class::__fuzz_class(data);
});

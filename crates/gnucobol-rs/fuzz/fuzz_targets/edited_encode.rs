#![no_main]
//! Fuzz numeric->edited encode (`GNURUST.16c`): arbitrary picture + value.
//! FUZZFOR: GNURUST.16C
//! panic-freedom — any hostile picture/value pair yields bytes or a typed `EditedError`, never a panic.

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    gnucobol_rs::edited::__fuzz_edited_encode(data);
});

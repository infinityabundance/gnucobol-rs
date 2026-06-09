#![no_main]
//! Fuzz the edited-picture decode (`GNURUST.16a`): arbitrary picture + bytes. The only assertion is
//! FUZZFOR: GNURUST.16
//! panic-freedom — any hostile picture/byte pair yields a typed `EditedError`, never a panic.

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    gnucobol_rs::edited::__fuzz_edited(data);
});

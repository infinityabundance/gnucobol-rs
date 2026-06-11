#![no_main]
//! Fuzz reference modification (`GNURUST.REFMOD.1`): arbitrary start/length/bytes.
//! FUZZFOR: GNURUST.REFMOD.1
//! panic-freedom — bounded slicing/overwrite never reads or writes out of bounds.
use libfuzzer_sys::fuzz_target;
fuzz_target!(|data: &[u8]| {
    gnucobol_rs::refmod::__fuzz_refmod(data);
});

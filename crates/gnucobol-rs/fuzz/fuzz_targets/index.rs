#![no_main]
//! Fuzz USAGE INDEX storage + SET arithmetic (`GNURUST.INDEX.1`).
//! FUZZFOR: GNURUST.INDEX.1
//! panic-freedom + store/value round-trip — SET TO/UP BY/DOWN BY never panic over any input.
use libfuzzer_sys::fuzz_target;
fuzz_target!(|data: &[u8]| {
    gnucobol_rs::index_item::__fuzz_index(data);
});

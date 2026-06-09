#![no_main]
//! FUZZFOR: GNURUST.SEARCH.TABLE.1
//! Fuzz SEARCH/SEARCH ALL over arbitrary table bytes. Panic-freedom (`GNURUST.PANICPOLICY.0`).

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| { gnucobol_rs::__fuzz_search(data); });

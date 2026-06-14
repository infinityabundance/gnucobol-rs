#![no_main]
//! Fuzz the native XML PARSE state machine: arbitrary documents drive `cob_xml_parse` to termination.
//! FUZZFOR: GNURUST.MLIO.PARSE.1
//! (`GNURUST.PANICPOLICY.0`) -- the parse loop always terminates and never panics on hostile input.

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    gnucobol_rs::__fuzz_mlio_parse(data);
});

#![no_main]
//! Fuzz the clean-room COBOL front-end: arbitrary source bytes drive `run_program` to either an
//! executed result or a fail-closed `RunError`, never a panic (`GNURUST.FRONTEND.1`,
//! `GNURUST.PANICPOLICY.0`).
//! FUZZFOR: GNURUST.FRONTEND.1, GNURUST.FILEIO.MULTI-RECORD-FD.1

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    gnucobol_rs::__fuzz_frontend(data);
});

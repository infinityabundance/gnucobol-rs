#![no_main]
//! Fuzz the line-sequential WRITE court: arbitrary bytes + config bits. The assertion is panic-freedom
//! FUZZFOR: GNURUST.FILEIO.LINESEQ.1
//! (`GNURUST.PANICPOLICY.0`) -- any hostile/malformed input yields a typed result or value, never a panic.

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    gnucobol_rs::__fuzz_lineseq(data);
});

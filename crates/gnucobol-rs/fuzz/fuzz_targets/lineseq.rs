#![no_main]
//! Fuzz the line-sequential WRITE+READ + record-sequential courts: arbitrary bytes + config. Panic-freedom.
//! FUZZFOR: GNURUST.FILEIO.LINESEQ.1, GNURUST.FILEIO.LINESEQ.2, GNURUST.FILEIO.SEQ.1
//! (`GNURUST.PANICPOLICY.0`) -- any hostile/malformed input yields a typed result or value, never a panic.

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    gnucobol_rs::__fuzz_lineseq(data);
});

#![no_main]
//! Fuzz the PIC parser: arbitrary bytes as a PICTURE string. The only assertion is panic-freedom
//! (`GNURUST.PANICPOLICY.0`): any hostile/malformed picture yields a typed `PicError`, never a panic.

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    gnucobol_rs::__fuzz_pic(data);
});

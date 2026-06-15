#![no_main]
//! Fuzz the native SCREEN SECTION DISPLAY emitter: arbitrary positions/payloads produce a well-formed
//! ncurses prologue..epilogue envelope without panicking.
//! FUZZFOR: GNURUST.SCREENIO.INIT.1
//! FUZZFOR: GNURUST.SCREENIO.DISPLAY.2
//! FUZZFOR: GNURUST.SCREENIO.DISPLAY.3
//! FUZZFOR: GNURUST.SCREENIO.ATTR.1
//! FUZZFOR: GNURUST.SCREENIO.COLOR.1

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    gnucobol_rs::__fuzz_screenio(data);
});

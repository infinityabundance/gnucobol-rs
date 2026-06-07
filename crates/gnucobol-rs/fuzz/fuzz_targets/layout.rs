#![no_main]
//! Fuzz the record layout engine: arbitrary items (levels/PICs/OCCURS/REDEFINES). Asserts only
//! panic-freedom (`GNURUST.PANICPOLICY.0`): hostile nesting/counts yield a typed `LayoutError`.

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    gnucobol_rs::__fuzz_layout(data);
});

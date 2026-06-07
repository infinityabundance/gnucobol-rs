#![no_main]
//! Fuzz the hostile surface: arbitrary bytes + attributes through `cob_move` and the value
//! decoders. The only assertion is panic-freedom (`GNURUST.PANICPOLICY.0`): a corrupt/oversized
//! field must yield a typed result or guarded bytes, never a panic, OOB index, or overflow.

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    cobol_decimal_rs::__fuzz_cob_move(data);
});

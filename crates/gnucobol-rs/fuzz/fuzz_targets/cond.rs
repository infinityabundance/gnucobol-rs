#![no_main]
//! Fuzz LEVEL-88 evaluation: arbitrary parent bytes/attrs + value tables. Asserts only
//! panic-freedom (`GNURUST.PANICPOLICY.0`): hostile inputs yield a typed `ConditionError`/bool.

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    gnucobol_rs::__fuzz_cond(data);
});

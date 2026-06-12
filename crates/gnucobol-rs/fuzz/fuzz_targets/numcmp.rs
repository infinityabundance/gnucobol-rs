#![no_main]
//! Fuzz numeric comparison (cob_numeric_cmp on the Mpz/cob_decimal layer).
//! FUZZFOR: GNURUST.NUMCMP.1
//! Panic-freedom: comparing any two numeric field byte images yields -1/0/1, never a panic.
use libfuzzer_sys::fuzz_target;
fuzz_target!(|data: &[u8]| {
    gnucobol_rs::cob_decimal::__fuzz_numcmp(data);
});

#![no_main]
//! Fuzz integer exponentiation (cob_s32_pow/cob_s64_pow).
//! FUZZFOR: GNURUST.INTPOW.1
//! Panic-freedom: any (base, power) yields a wrapped result or a typed error, never a panic.
use libfuzzer_sys::fuzz_target;
fuzz_target!(|data: &[u8]| {
    gnucobol_rs::int_pow::__fuzz_int_pow(data);
});

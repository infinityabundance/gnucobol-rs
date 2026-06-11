#![no_main]
//! Fuzz OCCURS DEPENDING ON (`GNURUST.ODO.1`).
//! FUZZFOR: GNURUST.ODO.1
//! panic-freedom — used-length + bounded element access never read out of range.
use libfuzzer_sys::fuzz_target;
fuzz_target!(|data: &[u8]| {
    gnucobol_rs::odo::__fuzz_odo(data);
});

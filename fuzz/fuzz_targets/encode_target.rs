// Author: Julian Bolivar
// Version: 0.0.1
// Date: 2026-07-01
//
// Fuzz target: `ReedSolomon::encode` must never panic or overflow on arbitrary
// (fuzzer-size-capped) input; an over-long output length must surface as
// `RsError::InvalidInput`, not a panic.
#![no_main]

use libfuzzer_sys::fuzz_target;
use reedsolomon::ReedSolomon;

fuzz_target!(|input: &[u8]| {
    let rs = ReedSolomon::default();
    let _ = rs.encode(input);
});

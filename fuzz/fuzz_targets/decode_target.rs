// Author: Julian Bolivar
// Version: 0.1.0
// Date: 2026-07-01
//
// Fuzz target: `ReedSolomon::decode` must never panic, loop forever, or read
// out of bounds on arbitrary input, for any `original_len` (including large and
// inconsistent values that exercise the InvalidInput / overflow-guard paths).
#![no_main]

use libfuzzer_sys::fuzz_target;
use reedsolomon::ReedSolomon;

fuzz_target!(|input: &[u8]| {
    if input.len() < 4 {
        return;
    }
    // Derive a wide `original_len` from the first 4 bytes so the fuzzer reaches
    // small, huge, and inconsistent values — not just a 0..=255 range.
    let original_len =
        u32::from_le_bytes([input[0], input[1], input[2], input[3]]) as usize;
    let rs = ReedSolomon::default();
    let _ = rs.decode(&input[4..], original_len);
});

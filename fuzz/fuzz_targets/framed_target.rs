// Author: Julian Bolivar
// Version: 0.2.0
// Date: 2026-07-01
//
// Fuzz target: `ReedSolomon::decode_framed` must never panic, loop forever, or
// read out of bounds when fed arbitrary bytes as a frame (malformed magic /
// version / CRC / params / truncated header must all fail gracefully).
#![no_main]

use libfuzzer_sys::fuzz_target;
use reedsolomon::ReedSolomon;

fuzz_target!(|input: &[u8]| {
    let rs = ReedSolomon::default();
    let _ = rs.decode_framed(input);
});

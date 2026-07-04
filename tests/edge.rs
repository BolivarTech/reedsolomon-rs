// Author: Julian Bolivar
// Version: 0.2.0
// Date: 2026-07-01

//! Construction and I/O edge-case tests for the Reed-Solomon codec.
//!
//! These pin the fixed-vector boundary behaviour of the *already-implemented*
//! public API: `new` preconditions, boundary payload sizes, structural
//! `decode` rejections (`InvalidInput`), the detect-only `t = 0` code, and the
//! rule that last-block padding corruption counts toward the error budget.

use reedsolomon::{ReedSolomon, RsError};

/// Byte pattern XORed into a codeword to inject a recoverable error.
const ERROR_MASK_RECOVERABLE: u8 = 0x5A;
/// Byte pattern XORed into a codeword to inject an over-capacity error.
const ERROR_MASK_OVERFLOW: u8 = 0xA5;
/// Correction capacity of the default RS(255, 223) code (`t = parity_len / 2`).
const DEFAULT_T: usize = 16;

/// `new` accepts only `1 <= parity_len`, `1 <= data_len`, `sum <= 255`.
#[test]
fn new_precondition_errors() {
    assert!(ReedSolomon::new(0, 223).is_err());
    assert!(ReedSolomon::new(32, 0).is_err());
    assert!(ReedSolomon::new(0, 0).is_err());
    assert!(ReedSolomon::new(32, 224).is_err()); // sum 256 > 255
    assert!(ReedSolomon::new(32, 223).is_ok()); // sum 255 boundary
    assert!(ReedSolomon::new(1, 1).is_ok()); // n=2, t=0
    assert!(ReedSolomon::new(254, 1).is_ok()); // max parity
}

/// Payload sizes around the chunk boundary all round-trip exactly.
#[test]
fn boundary_sizes_round_trip() {
    let rs = ReedSolomon::default();
    for len in [0usize, 1, 222, 223, 224, 446, 669] {
        let data = vec![0x42u8; len];
        let enc = rs.encode(&data).unwrap();
        assert_eq!(rs.decode(&enc, len).unwrap(), data);
    }
}

/// Structural `decode` violations are reported as `InvalidInput`, never a
/// panic or silent wrong result, including the `usize::MAX` overflow case.
#[test]
fn decode_structural_errors_are_invalid_input() {
    let rs = ReedSolomon::default();
    let enc = rs.encode(&[1, 2, 3]).unwrap(); // one 255-byte block
    assert!(matches!(
        rs.decode(&enc[..254], 3),
        Err(RsError::InvalidInput(_))
    )); // not multiple of n
    assert!(matches!(rs.decode(&enc, 0), Err(RsError::InvalidInput(_)))); // len 0 but data
    assert!(matches!(rs.decode(&[], 5), Err(RsError::InvalidInput(_)))); // empty but len>0
    assert_eq!(rs.decode(&[], 0).unwrap(), Vec::<u8>::new()); // empty ok
    assert!(matches!(
        rs.decode(&enc, usize::MAX),
        Err(RsError::InvalidInput(_))
    )); // no overflow
}

/// A `t = 0` code (`parity_len = 1`) decodes a clean block but declares any
/// corruption `Uncorrectable` (detection only, zero correction).
#[test]
fn t0_code_detects_but_cannot_correct() {
    let rs = ReedSolomon::new(1, 5).unwrap(); // t=0
    let data = [1u8, 2, 3, 4, 5];
    let enc = rs.encode(&data).unwrap();
    assert_eq!(rs.decode(&enc, 5).unwrap(), data);
    let mut bad = enc.clone();
    bad[0] ^= 1;
    assert!(matches!(rs.decode(&bad, 5), Err(RsError::Uncorrectable(_))));
}

/// Corruption inside the stripped last-block padding still counts toward the
/// per-block error budget: `<= t` recovers, `> t` is `Uncorrectable`.
#[test]
fn corruption_in_last_block_padding_counts_toward_budget() {
    let rs = ReedSolomon::default();
    let data = vec![0x33u8; 100]; // < 223 => one block, padding spans [100, 223)
    let enc = rs.encode(&data).unwrap();
    // <= t corruptions inside the (stripped) padding region still recover.
    let mut c = enc.clone();
    for byte in c.iter_mut().skip(100).take(DEFAULT_T) {
        *byte ^= ERROR_MASK_RECOVERABLE;
    }
    assert_eq!(rs.decode(&c, data.len()).unwrap(), data);
    // > t corruptions in padding => Uncorrectable (padding counts in the budget).
    let mut c2 = enc.clone();
    for byte in c2.iter_mut().skip(100).take(DEFAULT_T + 1) {
        *byte ^= ERROR_MASK_OVERFLOW;
    }
    assert!(matches!(
        rs.decode(&c2, data.len()),
        Err(RsError::Uncorrectable(_))
    ));
}

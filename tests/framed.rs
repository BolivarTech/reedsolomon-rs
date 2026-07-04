// Author: Julian Bolivar
// Version: 0.2.0
// Date: 2026-06-29

//! Integration tests for the framed-format API: KAT pinning `encode_framed`
//! against the `reedsolo`-generated reference fixture, and a multi-block
//! round-trip that verifies `decode_framed` recovers corruption in the payload.

#![forbid(unsafe_code)]

use reedsolomon::ReedSolomon;

#[path = "kat_vectors_framed.rs"]
mod kat_vectors_framed;
use kat_vectors_framed::{RS255_DATA, RS255_FRAMED};

/// Confirms that `encode_framed` produces the exact byte sequence computed by
/// the offline `scripts/gen_kat.py` reference (reedsolo, fcr=112, prim=0x187).
/// A mismatch means the header layout or CRC computation drifted from the spec.
#[test]
fn framed_kat_matches_reference() {
    let rs = ReedSolomon::default();
    assert_eq!(rs.encode_framed(RS255_DATA).unwrap(), RS255_FRAMED);
}

/// Encodes a 500-byte payload (three blocks), corrupts one byte per codeword
/// (well within t=16), and asserts `decode_framed` recovers the original.
#[test]
fn framed_round_trip_recovers_errors() {
    let rs = ReedSolomon::default();
    let msg = vec![0x5Au8; 500]; // multi-block
    let mut framed = rs.encode_framed(&msg).unwrap();
    // corrupt <= t per block in the payload region (after the 17-byte header)
    for b in 0..(framed.len() / 255) {
        framed[17 + b * 255 + 1] ^= 0x33;
    }
    assert_eq!(rs.decode_framed(&framed).unwrap(), msg);
}

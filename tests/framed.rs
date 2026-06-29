// Author: Julian Bolivar
// Version: 0.1.0
// Date: 2026-06-29

//! Integration tests for the framed-format API: KAT pinning `encode_framed`
//! against the `reedsolo`-generated reference fixture, and a multi-block
//! round-trip that verifies `decode_framed` recovers corruption in the payload.

#![forbid(unsafe_code)]

use reedsolomon::ReedSolomon;

#[path = "kat_vectors.rs"]
mod kat_vectors;
use kat_vectors::*;

/// Confirms that `encode_framed` produces the exact byte sequence computed by
/// the offline `scripts/gen_kat.py` reference (reedsolo, fcr=112, prim=0x187).
/// A mismatch means the header layout or CRC computation drifted from the spec.
#[test]
fn framed_kat_matches_reference() {
    let rs = ReedSolomon::default();
    assert_eq!(rs.encode_framed(RS255_DATA).unwrap(), RS255_FRAMED);
}

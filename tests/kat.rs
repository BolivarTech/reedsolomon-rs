// Author: Julian Bolivar
// Version: 0.1.0
// Date: 2026-06-29

//! External Known-Answer Tests (KATs) pinning our RS convention against an
//! independent `reedsolo` reference configured identically (fcr=112,
//! prim=0x187, generator=2, c_exp=8). The fixture vectors in `kat_vectors.rs`
//! are generated offline by `scripts/gen_kat.py` and committed as the build
//! source of truth -- `cargo` never runs Python. These tests prove our native
//! `encode`/`decode` match that reference byte-for-byte across the default
//! RS(255,223) code, a shortened RS(15,11,4) code, and an odd-parity
//! RS(13,10,3) code (t=1).

#![forbid(unsafe_code)]

use reedsolomon::ReedSolomon;

#[path = "kat_vectors.rs"]
mod kat_vectors;
use kat_vectors::{
    RS13_CORRUPTED, RS13_DATA, RS13_DECODED, RS13_ENCODED, RS13_UNCORRECTABLE, RS15_CORRUPTED,
    RS15_DATA, RS15_DECODED, RS15_ENCODED, RS15_UNCORRECTABLE, RS255_CORRUPTED, RS255_DATA,
    RS255_DECODED, RS255_ENCODED, RS255_UNCORRECTABLE,
};

#[test]
fn rs255_encode_matches_reference() {
    let rs = ReedSolomon::default();
    assert_eq!(rs.encode(RS255_DATA).unwrap(), RS255_ENCODED);
}

#[test]
fn rs255_decode_recovers_reference_corruption() {
    let rs = ReedSolomon::default();
    assert_eq!(
        rs.decode(RS255_CORRUPTED, RS255_DECODED.len()).unwrap(),
        RS255_DECODED
    );
}

#[test]
fn rs255_decode_declares_failure_on_over_capacity() {
    let rs = ReedSolomon::default();
    assert!(rs.decode(RS255_UNCORRECTABLE, RS255_DATA.len()).is_err());
}

#[test]
fn rs15_shortened_encode_decode_match_reference() {
    let rs = ReedSolomon::new(4, 11).unwrap();
    assert_eq!(rs.encode(RS15_DATA).unwrap(), RS15_ENCODED);
    assert_eq!(
        rs.decode(RS15_CORRUPTED, RS15_DECODED.len()).unwrap(),
        RS15_DECODED
    );
}

#[test]
fn rs15_shortened_decode_declares_failure_on_over_capacity() {
    let rs = ReedSolomon::new(4, 11).unwrap();
    assert!(rs.decode(RS15_UNCORRECTABLE, RS15_DATA.len()).is_err());
}

#[test]
fn rs13_odd_parity_encode_decode_match_reference() {
    let rs = ReedSolomon::new(3, 10).unwrap(); // t = 1 (odd parity)
    assert_eq!(rs.encode(RS13_DATA).unwrap(), RS13_ENCODED);
    assert_eq!(
        rs.decode(RS13_CORRUPTED, RS13_DECODED.len()).unwrap(),
        RS13_DECODED
    );
}

#[test]
fn rs13_odd_parity_decode_declares_failure_on_over_capacity() {
    let rs = ReedSolomon::new(3, 10).unwrap(); // t = 1 (odd parity)
    assert!(rs.decode(RS13_UNCORRECTABLE, RS13_DATA.len()).is_err());
}

// Author: Julian Bolivar
// Version: 0.1.0
// Date: 2026-06-29

//! Reed-Solomon decoder pipeline: syndromes → inversionless Berlekamp-Massey
//! → Chien search → Forney → correct → post-correction syndrome verification.

// Suppress dead_code lint until Task 11 wires the decode pipeline into lib.rs.
#![allow(dead_code)]

use crate::gf256;
use crate::poly;
use crate::FCR;

/// Syndromes `S[s] = R(α^{FCR+s})` for `s in 0..parity_len`.
pub(crate) fn syndromes(block: &[u8], parity_len: usize) -> Vec<u8> {
    (0..parity_len)
        .map(|s| poly::eval(block, gf256::pow(gf256::ALPHA, FCR + s)))
        .collect()
}

/// True iff every syndrome is zero (error-free block, fast path).
pub(crate) fn all_zero(s: &[u8]) -> bool {
    s.iter().all(|&b| b == 0)
}

/// Stub (Red phase): real inversionless Berlekamp-Massey lands in Green.
pub(crate) fn berlekamp_massey(_synd: &[u8], _t: usize) -> Vec<u8> {
    vec![1]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clean_codeword_has_zero_syndromes() {
        let data = [9u8, 8, 7, 6, 5, 4, 3, 2, 1, 0, 11];
        let enc = crate::encode::encode_blocks(&data, 11, 4).unwrap();
        let s = syndromes(&enc, 4);
        assert!(all_zero(&s), "valid codeword => zero syndromes");
    }

    #[test]
    fn single_error_gives_nonzero_syndromes() {
        let data = [9u8, 8, 7, 6, 5, 4, 3, 2, 1, 0, 11];
        let mut enc = crate::encode::encode_blocks(&data, 11, 4).unwrap();
        enc[3] ^= 0x5A;
        let s = syndromes(&enc, 4);
        assert!(!all_zero(&s));
    }

    #[test]
    fn bm_locator_degree_matches_single_error() {
        // one error => deg(Lambda) == 1
        let data = [1u8, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11];
        let mut enc = crate::encode::encode_blocks(&data, 11, 4).unwrap();
        enc[2] ^= 0x33;
        let s = syndromes(&enc, 4);
        let lambda = berlekamp_massey(&s, 2);
        let degree = lambda.len() - 1 - lambda.iter().take_while(|&&c| c == 0).count();
        assert_eq!(degree, 1, "exactly one error");
    }
}

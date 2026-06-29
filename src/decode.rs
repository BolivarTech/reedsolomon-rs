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
}

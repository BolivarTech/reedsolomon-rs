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

/// Inversionless Berlekamp-Massey. `synd[s] = S_s` (the `2t` syndromes).
/// Returns Λ (big-endian). Λ may be scaled by a nonzero constant — this does
/// **not** affect Chien roots nor the Forney ratio `Ω/Λ'`, so no normalization
/// is needed.
///
/// Genuinely inversionless recurrence (no GF inverse in the loop, only
/// `add`/`mul`): `Λ*(x) = γ·Λ(x) − δ·x·B(x)`; on a length change
/// `B←Λ, L←r+1−L, γ←δ`, otherwise `B←x·B`. Internally low-endian (index =
/// degree), converted to big-endian at the end. KAT-pinned (Task 12).
pub(crate) fn berlekamp_massey(synd: &[u8], _t: usize) -> Vec<u8> {
    // Low-endian (index = degree). Inversionless: replace `d/b` by carrying the
    // last discrepancy `gamma` and scaling Λ by it. Shift is the EXPLICIT `x^m`.
    let n = synd.len();
    let mut lambda = vec![0u8; n + 1];
    lambda[0] = 1;
    let mut b = vec![0u8; n + 1];
    b[0] = 1;
    let mut l = 0usize;
    let mut m = 1usize; // current shift exponent x^m
    let mut gamma = 1u8; // last nonzero discrepancy
    for r in 0..n {
        // discrepancy δ = Σ_{i=0..l} Λ_i · S_{r-i}
        let mut delta = 0u8;
        for i in 0..=l {
            if i <= r {
                delta = gf256::add(delta, gf256::mul(lambda[i], synd[r - i]));
            }
        }
        if delta == 0 {
            m += 1;
        } else {
            // Λ <- γ·Λ − δ·x^m·B   (no inverse; subtraction == XOR)
            let prev = lambda.clone(); // un-scaled old Λ (becomes B on length change)
            for c in lambda.iter_mut() {
                *c = gf256::mul(gamma, *c);
            }
            for k in 0..b.len() {
                if k + m < lambda.len() {
                    lambda[k + m] = gf256::add(lambda[k + m], gf256::mul(delta, b[k]));
                }
            }
            if 2 * l <= r {
                l = r + 1 - l;
                b = prev;
                gamma = delta;
                m = 1;
            } else {
                m += 1;
            }
        }
    }
    // trim high-degree zeros (low-endian), then convert to big-endian
    let mut deg = lambda.len() - 1;
    while deg > 0 && lambda[deg] == 0 {
        deg -= 1;
    }
    let mut be = lambda[..=deg].to_vec();
    be.reverse();
    be
}

/// Chien search stub (Red phase: no real logic yet).
pub(crate) fn chien_search(lambda: &[u8], n: usize) -> Vec<usize> {
    let _ = (lambda, n);
    Vec::new()
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

    #[test]
    fn chien_finds_the_injected_position() {
        let data = [1u8, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11];
        let mut enc = crate::encode::encode_blocks(&data, 11, 4).unwrap();
        let pos = 6usize;
        enc[pos] ^= 0x77;
        let s = syndromes(&enc, 4);
        let lambda = berlekamp_massey(&s, 2);
        let roots = chien_search(&lambda, enc.len());
        assert!(
            roots.contains(&pos),
            "Chien locates the error byte position"
        );
    }
}

// Author: Julian Bolivar
// Version: 0.1.0
// Date: 2026-06-29

//! Reed-Solomon decoder pipeline: syndromes → inversionless Berlekamp-Massey
//! → Chien search → Forney → correct → post-correction syndrome verification.

// Suppress dead_code lint until Task 11 wires the decode pipeline into lib.rs.
#![allow(dead_code)]

use crate::gf256;
use crate::poly;
use crate::RsError;
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

/// Chien search: return the byte positions `j in 0..n` whose locator value
/// `X = α^{n-1-j}` is an error location (`Λ(X^{-1}) == 0`). Only positions
/// inside `[0, n)` are returned; "phantom" roots of shortened codes are dropped
/// here and surface as a degree/root-count mismatch in `decode_block`.
pub(crate) fn chien_search(lambda: &[u8], n: usize) -> Vec<usize> {
    let mut positions = Vec::new();
    for j in 0..n {
        // X = alpha^(n-1-j); X^{-1} = alpha^(-(n-1-j)) = alpha^(255-((n-1-j)%255))
        let exp_inv = (255 - ((n - 1 - j) % 255)) % 255;
        let x_inv = gf256::pow(gf256::ALPHA, exp_inv);
        if poly::eval(lambda, x_inv) == 0 {
            positions.push(j);
        }
    }
    positions
}

/// Error-evaluator and Forney magnitudes. Returns one magnitude per position
/// (in the given order), or `None` if any magnitude is zero (degenerate
/// solution → caller declares `Uncorrectable`).
///
/// For each error location `j` with locator value `X = α^{n-1-j}`, the Forney
/// algorithm computes the magnitude
/// `e = X^{1-FCR} · Ω(X^{-1}) / Λ'(X^{-1})`, where:
/// - `Ω(x) = [S(x)·Λ(x)] mod x^{2t}` is the error-evaluator polynomial;
/// - `Λ'(x)` is the formal derivative of Λ (only odd-degree terms survive in
///   characteristic 2);
/// - the `X^{1-FCR}` factor (with **FCR = 112**) is **mandatory** — omitting it
///   (the implicit `FCR = 1` assumption) yields wrong magnitudes.
///
/// `lambda` is big-endian (Task 7) and `positions` come from Chien search
/// (Task 8). To keep a single endianness convention, `Ω` and `Λ'` are built
/// low-endian (index = degree), then reversed to big-endian so every evaluation
/// goes through [`poly::eval`].
pub(crate) fn forney(
    synd: &[u8],
    lambda: &[u8], // big-endian
    positions: &[usize],
    n: usize,
) -> Option<Vec<u8>> {
    let two_t = synd.len();
    // Convolve low-endian (index = degree), then convert to big-endian so ALL
    // evaluations go through poly::eval (single endianness convention).
    let lambda_le: Vec<u8> = {
        let mut v = lambda.to_vec();
        v.reverse();
        v
    };
    // Ω(x) = (S(x)·Λ(x)) mod x^{2t}, low-endian: omega_le[i] = Σ_{j<=i} S_{i-j}·Λ_j
    let mut omega_le = vec![0u8; two_t];
    for i in 0..two_t {
        let mut acc = 0u8;
        for j in 0..=i {
            if j < lambda_le.len() {
                acc = gf256::add(acc, gf256::mul(synd[i - j], lambda_le[j]));
            }
        }
        omega_le[i] = acc;
    }
    // Λ'(x): formal derivative — odd-degree terms survive in char 2 (low-endian).
    let mut lp_le = vec![0u8; lambda_le.len().saturating_sub(1)];
    for d in 1..lambda_le.len() {
        if d % 2 == 1 {
            lp_le[d - 1] = lambda_le[d];
        }
    }
    // to big-endian for poly::eval
    let omega_be: Vec<u8> = {
        let mut v = omega_le;
        v.reverse();
        v
    };
    let lp_be: Vec<u8> = {
        let mut v = lp_le;
        v.reverse();
        v
    };
    let mut mags = Vec::with_capacity(positions.len());
    for &j in positions {
        let exp = (n - 1 - j) % 255; // X = α^exp
        let x = gf256::pow(gf256::ALPHA, exp);
        let x_inv = gf256::inv(x);
        let num = poly::eval(&omega_be, x_inv);
        let den = poly::eval(&lp_be, x_inv);
        if den == 0 {
            return None;
        }
        // Forney magnitude with FCR factor: e = X^{1-FCR} · Ω(X^{-1}) / Λ'(X^{-1})
        let factor = gf256::mul(x, gf256::pow(x_inv, FCR)); // X^{1} · X^{-FCR}
        let mag = gf256::mul(factor, gf256::div(num, den));
        if mag == 0 {
            return None;
        }
        mags.push(mag);
    }
    Some(mags)
}

/// Decode and error-correct one `n`-byte block; return its `data_len` data
/// bytes. `Uncorrectable` on any inconsistency (the never-mis-correct invariant).
pub(crate) fn decode_block(_block: &[u8], _parity_len: usize) -> Result<Vec<u8>, RsError> {
    todo!("decode_block: implemented in Green phase")
}

/// Block loop with structural validation and final truncation to `original_len`.
pub(crate) fn decode_blocks(
    _encoded: &[u8],
    _original_len: usize,
    _data_len: usize,
    _parity_len: usize,
) -> Result<Vec<u8>, RsError> {
    todo!("decode_blocks: implemented in Green phase")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decode_block_recovers_up_to_t_errors() {
        let data = [1u8, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11];
        let enc = crate::encode::encode_blocks(&data, 11, 4).unwrap();
        let mut blk = enc.clone();
        blk[1] ^= 0x10;
        blk[7] ^= 0x20; // t=2 errors
        let recovered = decode_block(&blk, 4).unwrap();
        assert_eq!(recovered, &data[..]);
    }

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

    #[test]
    fn forney_corrects_a_single_known_error() {
        let data = [5u8, 4, 3, 2, 1, 9, 8, 7, 6, 0, 1];
        let mut enc = crate::encode::encode_blocks(&data, 11, 4).unwrap();
        let clean = enc.clone();
        let pos = 9usize;
        enc[pos] ^= 0xC4;
        let s = syndromes(&enc, 4);
        let lambda = berlekamp_massey(&s, 2);
        let positions = chien_search(&lambda, enc.len());
        let mags = forney(&s, &lambda, &positions, enc.len()).unwrap();
        for (k, &p) in positions.iter().enumerate() {
            enc[p] = crate::gf256::add(enc[p], mags[k]);
        }
        assert_eq!(
            enc, clean,
            "applying Forney magnitudes restores the codeword"
        );
    }
}

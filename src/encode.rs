// Author: Julian Bolivar
// Version: 0.1.0
// Date: 2026-06-29

//! Systematic Reed-Solomon encoder over GF(2^8).

// Suppress dead_code lint until the systematic encoder (later task) calls
// build_generator from non-test production code.
#![allow(dead_code)]

use crate::gf256;
use crate::poly;
use crate::FCR;

/// Build the code generator `g(x) = Π_{i=FCR}^{FCR+parity_len-1} (x - α^i)`.
///
/// Returns a monic, big-endian polynomial of length `parity_len + 1` whose
/// roots are `α^FCR, α^(FCR+1), …, α^(FCR+parity_len-1)` over GF(2^8).
///
/// # Parameters
/// - `parity_len`: number of parity symbols `2t`; determines the degree of
///   `g(x)` and the number of correctable errors `t = parity_len / 2`.
///
/// # Examples
/// ```ignore
/// // For parity_len = 4 the polynomial has degree 4 (5 coefficients).
/// let g = build_generator(4);
/// assert_eq!(g.len(), 5);
/// assert_eq!(g[0], 1); // monic
/// ```
pub(crate) fn build_generator(parity_len: usize) -> Vec<u8> {
    let mut g = vec![1u8];
    for i in 0..parity_len {
        let root = gf256::pow(gf256::ALPHA, FCR + i);
        // Multiply g by (x - root); in GF(2^8), subtraction == addition == XOR.
        g = poly::mul(&g, &[1, root]);
    }
    g
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generator_has_expected_shape_and_roots() {
        let g = build_generator(4);
        assert_eq!(g.len(), 5, "degree parity_len");
        assert_eq!(g[0], 1, "monic");
        for i in 0..4usize {
            let root = crate::gf256::pow(crate::gf256::ALPHA, crate::FCR + i);
            assert_eq!(crate::poly::eval(&g, root), 0, "g(alpha^(FCR+i)) == 0");
        }
    }

    #[test]
    fn encode_rejects_length_overflow() {
        // data_len=1, parity_len=254 => n=255; a len near usize::MAX overflows B*n.
        // Use a fake huge len via a zero-length slice is impossible; assert the
        // checked-arithmetic helper directly (see encoded_len).
        assert!(encoded_len(usize::MAX, 1, 254).is_none());
        assert_eq!(encoded_len(0, 11, 4), Some(0));
        assert_eq!(encoded_len(25, 11, 4), Some(45));
    }
}

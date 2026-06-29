// Author: Julian Bolivar
// Version: 0.1.0
// Date: 2026-06-29

//! Polynomials over GF(2^8), stored **big-endian**: index 0 is the
//! highest-degree coefficient.
// Suppress dead_code lint until encode/decode modules (later tasks)
// reference these functions from non-test production code.
#![allow(dead_code)]

use crate::gf256;

/// Evaluate `p(x)` by Horner's method.
///
/// Returns the field element obtained by evaluating the polynomial `p` at `x`
/// using the recurrence `acc = acc * x + c` over GF(2^8).
///
/// # Parameters
/// - `p`: coefficient slice, big-endian (index 0 = leading coefficient).
/// - `x`: the field element at which to evaluate.
///
/// # Examples
/// ```ignore
/// // constant polynomial p = [7] evaluates to 7 at any point.
/// assert_eq!(eval(&[7], 99), 7);
/// ```
pub(crate) fn eval(p: &[u8], x: u8) -> u8 {
    let mut acc = 0u8;
    for &c in p {
        acc = gf256::add(gf256::mul(acc, x), c);
    }
    acc
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn eval_constant_poly_returns_constant() {
        // A single-coefficient polynomial is a constant: p(x) = 7 for all x.
        assert_eq!(eval(&[7], 99), 7);
    }
}

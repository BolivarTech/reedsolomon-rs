// Author: Julian Bolivar
// Version: 0.1.0
// Date: 2026-06-29

//! Polynomials over GF(2^8), stored **big-endian**: index 0 is the
//! highest-degree coefficient.

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

/// Multiply two polynomials over GF(2^8).
///
/// Returns a polynomial of degree `deg(a) + deg(b)`, or an empty `Vec` if
/// either input is empty.
///
/// # Parameters
/// - `a`, `b`: polynomials in big-endian order.
pub(crate) fn mul(a: &[u8], b: &[u8]) -> Vec<u8> {
    if a.is_empty() || b.is_empty() {
        return Vec::new();
    }
    let mut out = vec![0u8; a.len() + b.len() - 1];
    for (i, &ca) in a.iter().enumerate() {
        for (j, &cb) in b.iter().enumerate() {
            out[i + j] = gf256::add(out[i + j], gf256::mul(ca, cb));
        }
    }
    out
}

/// Compute `dividend mod divisor` over GF(2^8).
///
/// `divisor` must be monic (leading coefficient `1`). The result has length
/// `divisor.len() - 1` (i.e. degree one less than the divisor).
///
/// # Parameters
/// - `dividend`: polynomial to be divided (big-endian).
/// - `divisor`: monic divisor polynomial (big-endian).
///
/// # Panics
/// Debug-asserts that `divisor[0] == 1`.
pub(crate) fn remainder(dividend: &[u8], divisor: &[u8]) -> Vec<u8> {
    debug_assert!(divisor.first() == Some(&1), "divisor must be monic");
    let mut work = dividend.to_vec();
    let dl = divisor.len();
    for i in 0..work.len().saturating_sub(dl - 1) {
        let coef = work[i];
        if coef != 0 {
            for j in 1..dl {
                work[i + j] = gf256::add(work[i + j], gf256::mul(divisor[j], coef));
            }
        }
    }
    work[work.len() - (dl - 1)..].to_vec()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn eval_constant_poly_returns_constant() {
        // A single-coefficient polynomial is a constant: p(x) = 7 for all x.
        assert_eq!(eval(&[7], 99), 7);

        // p(x) = x^2 + 3x + 5 (big-endian [1,3,5]); eval at x=2 must equal 7
        let p = [1u8, 3, 5];
        let expected = crate::gf256::add(
            crate::gf256::add(
                crate::gf256::mul(1, crate::gf256::mul(2, 2)),
                crate::gf256::mul(3, 2),
            ),
            5,
        );
        assert_eq!(eval(&p, 2), expected); // expected == 7
    }

    #[test]
    fn remainder_degree_is_below_divisor() {
        let dividend = [1u8, 0, 0, 0, 0]; // x^4
        let divisor = [1u8, 1, 1]; // x^2 + x + 1 (monic)
        let r = remainder(&dividend, &divisor);
        assert_eq!(r.len(), divisor.len() - 1);
        // x^4 mod (x^2+x+1) over GF(2^8) = x, big-endian [1, 0]
        assert_eq!(r, vec![1u8, 0u8]);
    }

    #[test]
    fn mul_then_eval_is_pointwise_product() {
        let a = [1u8, 2]; // x + 2
        let b = [1u8, 3]; // x + 3
        let prod = mul(&a, &b);
        for x in 0u16..256 {
            let x = x as u8;
            assert_eq!(eval(&prod, x), crate::gf256::mul(eval(&a, x), eval(&b, x)));
        }
    }
}

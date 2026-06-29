// Author: Julian Bolivar
// Version: 0.1.0
// Date: 2026-06-29

//! GF(2^8) arithmetic for the Reed-Solomon codec.
//!
//! Conventional-basis field built with the CCSDS field polynomial `0x187`
//! (`x^8 + x^7 + x^2 + x + 1`) and primitive element `α = 0x02`. All multiply,
//! divide and inverse operations use compile-time `const` log/antilog tables.
//!
//! All items are `pub(crate)`; they appear unused to the lib target while only
//! referenced from tests, hence the module-level suppression below.
#![allow(dead_code)]

/// CCSDS field-generator polynomial `x^8 + x^7 + x^2 + x + 1` (9-bit, `0x187`).
pub(crate) const FIELD_POLY: u16 = 0x187;
/// Primitive element generating the multiplicative group.
pub(crate) const ALPHA: u8 = 2;
/// Number of field elements.
pub(crate) const FIELD_SIZE: usize = 256;

/// Antilog table: `EXP[i] = α^i`. Length 512 so `mul` never needs a modulo
/// (two logs in `0..=254` sum to `≤ 508`). `EXP[255] = α^0 = 1`.
pub(crate) const EXP: [u8; 512] = build_tables().0;
/// Log table: `LOG[α^i] = i`; `LOG[0]` is unused (0 has no log).
pub(crate) const LOG: [u8; 256] = build_tables().1;

const fn build_tables() -> ([u8; 512], [u8; 256]) {
    let mut exp = [0u8; 512];
    let mut log = [0u8; 256];
    let mut x: u16 = 1;
    let mut i = 0usize;
    while i < 255 {
        exp[i] = x as u8;
        log[x as usize] = i as u8;
        x <<= 1;
        if x & 0x100 != 0 {
            x ^= FIELD_POLY;
        }
        i += 1;
    }
    let mut j = 255usize;
    while j < 512 {
        exp[j] = exp[j - 255];
        j += 1;
    }
    (exp, log)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `α^0` must equal 1 (identity element of the multiplicative group).
    #[test]
    fn exp_zero_is_one() {
        assert_eq!(EXP[0], 1, "alpha^0 == 1");
    }

    /// `α^1` must equal the primitive element value `ALPHA`.
    #[test]
    fn exp_one_is_alpha() {
        assert_eq!(EXP[1], ALPHA, "alpha^1 == alpha");
    }

    /// Multiplicative order of `α` over GF(2^8) is 255: `α^255 = α^0 = 1`.
    #[test]
    fn multiplicative_order_is_255() {
        assert_eq!(EXP[255], 1, "multiplicative order is 255");
    }

    /// `log(1) == 0` since `α^0 = 1`.
    #[test]
    fn log_of_one_is_zero() {
        assert_eq!(LOG[1], 0, "log(1) == 0");
    }

    /// Round-trip: `EXP[LOG[x]] == x` for every non-zero field element.
    #[test]
    fn exp_log_roundtrip_for_all_nonzero_elements() {
        for x in 1u16..256 {
            assert_eq!(
                EXP[LOG[x as usize] as usize], x as u8,
                "exp(log(x)) == x for x={x}"
            );
        }
    }

    /// Multiplying any element by zero must yield zero (zero is the absorbing
    /// element of the multiplicative group extended to all of GF(2^8)).
    #[test]
    fn mul_zero_absorbs() {
        assert_eq!(mul(0, 123), 0);
        assert_eq!(mul(123, 0), 0);
        assert_eq!(mul(0, 0), 0);
    }

    /// The multiplicative identity element is 1: `mul(1, x) == x` for all x.
    #[test]
    fn mul_identity() {
        for x in 0u16..256 {
            assert_eq!(mul(1, x as u8), x as u8, "1 * {x} == {x}");
            assert_eq!(mul(x as u8, 1), x as u8, "{x} * 1 == {x}");
        }
    }

    /// GF(2^8) multiplication must be commutative for every pair of elements.
    #[test]
    fn mul_commutative() {
        for a in 0u16..256 {
            for b in 0u16..256 {
                assert_eq!(
                    mul(a as u8, b as u8),
                    mul(b as u8, a as u8),
                    "mul({a},{b}) != mul({b},{a})"
                );
            }
        }
    }

    /// `mul` is distributive over `add`: `a*(b+c) == a*b + a*c` for all
    /// elements in the field.
    #[test]
    fn mul_distributive_over_add() {
        for a in 0u16..256 {
            for b in 0u16..256 {
                for c in [0u8, 1, 2, 17, 200, 255] {
                    let lhs = mul(a as u8, add(b as u8, c));
                    let rhs = add(mul(a as u8, b as u8), mul(a as u8, c));
                    assert_eq!(lhs, rhs, "distributive failed: {a}*({b}+{c})");
                }
            }
        }
    }

    /// For every non-zero element `x`, `mul(x, inv(x)) == 1`.
    #[test]
    fn inv_times_element_is_one() {
        for a in 1u16..256 {
            assert_eq!(mul(a as u8, inv(a as u8)), 1, "x * inv(x) == 1 failed for x={a}");
        }
    }

    /// `div(a, b)` equals `mul(a, inv(b))` for all non-zero `b`; zero
    /// dividend always yields zero.
    #[test]
    fn div_consistent_with_mul_inv() {
        assert_eq!(div(0, 7), 0);
        for a in 0u16..256 {
            for b in 1u16..256 {
                assert_eq!(
                    div(a as u8, b as u8),
                    mul(a as u8, inv(b as u8)),
                    "div({a},{b}) != mul({a}, inv({b}))"
                );
            }
        }
    }

    /// `pow` edge cases: x^0 == 1, 0^e == 0 for e>0, α^255 == 1 (order),
    /// and `pow(b, e)` matches iterative `mul` for all small exponents.
    #[test]
    fn pow_edge_cases_and_matches_repeated_mul() {
        assert_eq!(pow(0, 0), 1, "0^0 == 1 by convention");
        assert_eq!(pow(7, 0), 1, "x^0 == 1");
        assert_eq!(pow(0, 5), 0, "0^e == 0 for e>0");
        assert_eq!(pow(ALPHA, 255), 1, "α^255 == 1 (multiplicative order)");
        // Verify pow matches iterative mul for base=3 up to exponent 299.
        let mut acc = 1u8;
        for e in 0..300usize {
            assert_eq!(pow(3, e), acc, "pow(3,{e}) != iterative mul");
            acc = mul(acc, 3);
        }
    }
}

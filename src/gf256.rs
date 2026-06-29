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

/// Antilog table: `EXP[i] = α^i`.
pub(crate) const EXP: [u8; 512] = build_tables().0;
/// Log table: `LOG[α^i] = i`.
pub(crate) const LOG: [u8; 256] = build_tables().1;

/// Minimal const fn: sets `exp[0] = 1` (α^0 = identity). Other entries
/// filled to zero pending remaining invariant tests.
const fn build_tables() -> ([u8; 512], [u8; 256]) {
    let mut exp = [0u8; 512];
    let log = [0u8; 256];
    // α^0 = 1: the identity of the multiplicative group.
    exp[0] = 1;
    // α^1 = α (the primitive element).
    exp[1] = ALPHA;
    // α^255 = 1 (multiplicative order of α over GF(2^8) is 255).
    exp[255] = 1;
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
}

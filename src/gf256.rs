// Author: Julian Bolivar
// Version: 0.1.0
// Date: 2026-06-29

//! GF(2^8) arithmetic for the Reed-Solomon codec.
//!
//! Conventional-basis field built with the CCSDS field polynomial `0x187`
//! (`x^8 + x^7 + x^2 + x + 1`) and primitive element `α = 0x02`. All multiply,
//! divide and inverse operations use compile-time `const` log/antilog tables.

/// CCSDS field-generator polynomial stub — replaced in Green.
pub(crate) const FIELD_POLY: u16 = 0;
/// Primitive element stub — replaced in Green.
pub(crate) const ALPHA: u8 = 0;
/// Number of field elements.
pub(crate) const FIELD_SIZE: usize = 256;

/// Antilog table stub — all zeros until `build_tables` is wired.
pub(crate) const EXP: [u8; 512] = [0u8; 512];
/// Log table stub — all zeros until `build_tables` is wired.
pub(crate) const LOG: [u8; 256] = [0u8; 256];

#[cfg(test)]
mod tests {
    use super::*;

    /// `α^0` must equal 1 (identity element of the multiplicative group).
    #[test]
    fn exp_zero_is_one() {
        assert_eq!(EXP[0], 1, "alpha^0 == 1");
    }
}

// Author: Julian Bolivar
// Version: 0.0.1
// Date: 2026-06-28

//! # reedsolomon
//!
//! Pure-Rust, **no-`unsafe`** Reed-Solomon forward error correction (FEC) over
//! GF(2^8), defaulting to the **RS(255, 223)** code used in deep-space
//! communication (CCSDS standard): 223 data bytes + 32 parity bytes per
//! 255-byte codeword, correcting up to **16 corrupted bytes per block**.
//!
//! This crate is the FEC layer of the [`cryptovault`] vault module, but is
//! self-contained and reusable on its own. It protects an already-encrypted
//! payload against bit-rot / storage corruption — it is **not** a security
//! primitive (it operates on ciphertext, carries no secrets, and has no
//! secret-dependent timing).
//!
//! ## Status: WORK IN PROGRESS (name-reservation scaffold)
//!
//! The public API below is the contract; the GF(2^8) implementation is being
//! written natively to remove any third-party FEC dependency.
//!
//! [`cryptovault`]: https://crates.io/crates/cryptovault

#![forbid(unsafe_code)]

pub(crate) mod decode;
pub(crate) mod encode;
pub(crate) mod gf256;
pub(crate) mod poly;

/// First consecutive root of the code generator polynomial (CCSDS convention).
pub(crate) const FCR: usize = 112;

use std::fmt;

/// Default number of data bytes per Reed-Solomon block — `k` in RS(*n*, *k*).
pub const DEFAULT_DATA_LEN: usize = 223;

/// Default number of parity bytes per block. Corrects up to `parity/2` byte
/// errors per codeword (`32/2 = 16` for the default code).
pub const DEFAULT_PARITY_LEN: usize = 32;

/// Maximum codeword length in GF(2^8): `data_len + parity_len <= 255`.
pub const MAX_BLOCK_SIZE: usize = 255;

/// Error returned by [`ReedSolomon`] operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RsError {
    /// A block contains more corrupted bytes than the code can correct, OR an
    /// uncorrectable/inconsistent encoded block was supplied. The decoder must
    /// **declare failure rather than mis-correct** (return wrong-but-plausible
    /// data) — a mis-correction is the one outcome this crate must never
    /// produce silently.
    Uncorrectable(String),
    /// A construction or input precondition was violated (e.g. zero lengths,
    /// `parity_len + data_len > 255`).
    InvalidInput(String),
}

impl fmt::Display for RsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Uncorrectable(m) => write!(f, "Reed-Solomon uncorrectable: {m}"),
            Self::InvalidInput(m) => write!(f, "Reed-Solomon invalid input: {m}"),
        }
    }
}

impl std::error::Error for RsError {}

/// Systematic Reed-Solomon RS(*n*, *k*) codec over GF(2^8).
///
/// `encode` appends `parity_len` parity bytes per `data_len`-byte chunk;
/// `decode` error-corrects each block independently and strips chunk padding.
#[derive(Debug, Clone, Copy)]
pub struct ReedSolomon {
    // Fields used by encode/decode (implemented in later tasks).
    #[allow(dead_code)]
    parity_len: usize,
    #[allow(dead_code)]
    data_len: usize,
}

impl Default for ReedSolomon {
    fn default() -> Self {
        Self {
            parity_len: DEFAULT_PARITY_LEN,
            data_len: DEFAULT_DATA_LEN,
        }
    }
}

impl ReedSolomon {
    /// Builds a codec with custom block sizes.
    ///
    /// # Errors
    /// [`RsError::InvalidInput`] if either length is zero or
    /// `parity_len + data_len > 255`.
    pub fn new(parity_len: usize, data_len: usize) -> Result<Self, RsError> {
        if parity_len == 0 || data_len == 0 {
            return Err(RsError::InvalidInput(
                "parity_len and data_len must be > 0".into(),
            ));
        }
        if parity_len + data_len > MAX_BLOCK_SIZE {
            return Err(RsError::InvalidInput(format!(
                "parity_len ({parity_len}) + data_len ({data_len}) exceeds GF(2^8) limit 255"
            )));
        }
        Ok(Self {
            parity_len,
            data_len,
        })
    }

    /// Encodes `data` into RS codewords (data chunks + parity).
    ///
    /// TODO: native GF(2^8) systematic encoder.
    pub fn encode(&self, _data: &[u8]) -> Vec<u8> {
        todo!("native GF(2^8) RS encoder")
    }

    /// Decodes + error-corrects `encoded`, returning the original `original_len`
    /// bytes. Must return [`RsError::Uncorrectable`] (never mis-corrected data)
    /// when a block exceeds the correction capacity.
    ///
    /// TODO: native GF(2^8) decoder (syndromes → inversionless Berlekamp-Massey →
    /// Chien search → Forney → post-correction syndrome verification).
    pub fn decode(&self, _encoded: &[u8], _original_len: usize) -> Result<Vec<u8>, RsError> {
        todo!("native GF(2^8) RS decoder")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_rejects_oversized_block() {
        assert!(matches!(
            ReedSolomon::new(32, 224),
            Err(RsError::InvalidInput(_))
        ));
    }

    #[test]
    fn new_rejects_zero_lengths() {
        assert!(ReedSolomon::new(0, 223).is_err());
        assert!(ReedSolomon::new(32, 0).is_err());
    }

    #[test]
    fn default_is_rs_255_223() {
        let rs = ReedSolomon::default();
        assert_eq!(rs.data_len, 223);
        assert_eq!(rs.parity_len, 32);
    }

    // TODO: round-trip KATs, corrupt-<=16-per-block recovery,
    // >16-errors-declares-failure (no mis-correction), and fuzzing.
}

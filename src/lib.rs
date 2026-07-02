// Author: Julian Bolivar
// Version: 0.1.0
// Date: 2026-06-29

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
//! ## Status: native codec wired
//!
//! The public [`ReedSolomon::encode`] / [`ReedSolomon::decode`] now delegate to
//! the native GF(2^8) block codec (no third-party FEC dependency): systematic
//! encoding and a syndromes → inversionless Berlekamp-Massey → Chien → Forney
//! decoder with mandatory post-correction syndrome verification.
//!
//! ## Framed format
//!
//! [`ReedSolomon::encode_framed`] / [`ReedSolomon::decode_framed`] add a
//! CRC-checked, self-describing 17-byte header in front of the raw codewords so
//! a decoder can reject a code-parameter mismatch and recover the true
//! `original_len` without the caller tracking either out of band. The raw
//! [`ReedSolomon::encode`] / [`ReedSolomon::decode`] path is unchanged and
//! carries zero overhead.
//!
//! ## Usage
//!
//! Raw path — zero overhead; the caller tracks `(parity_len, data_len)` and the
//! original length:
//!
//! ```
//! use reedsolomon::ReedSolomon;
//!
//! let rs = ReedSolomon::default();
//! let data: &[u8] = b"hello reed-solomon";
//! let encoded = rs.encode(data)?;
//! let decoded = rs.decode(&encoded, data.len())?;
//! assert_eq!(decoded.as_slice(), data);
//! # Ok::<(), reedsolomon::RsError>(())
//! ```
//!
//! Framed path — self-describing; the header carries the parameters and length,
//! so no external tracking is needed:
//!
//! ```
//! use reedsolomon::ReedSolomon;
//!
//! let rs = ReedSolomon::default();
//! let data: &[u8] = b"hello reed-solomon";
//! let framed = rs.encode_framed(data)?;
//! let decoded = rs.decode_framed(&framed)?;
//! assert_eq!(decoded.as_slice(), data);
//! # Ok::<(), reedsolomon::RsError>(())
//! ```
//!
//! Allocation is linear in input size and grown via `try_reserve`; an
//! unsatisfiable allocation returns [`RsError::InvalidInput`] instead of
//! aborting the process (memory-exhaustion DoS hardening).
//!
//! [`cryptovault`]: https://crates.io/crates/cryptovault

#![forbid(unsafe_code)]

pub(crate) mod crc;
pub(crate) mod decode;
pub(crate) mod encode;
mod frame;
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
    /// Number of parity bytes appended per `data_len`-byte chunk.
    parity_len: usize,
    /// Number of data bytes per chunk (`k` in RS(*n*, *k*)).
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
    /// # Errors
    /// [`RsError::InvalidInput`] if the encoded length overflows `usize` or the
    /// output allocation fails.
    pub fn encode(&self, data: &[u8]) -> Result<Vec<u8>, RsError> {
        crate::encode::encode_blocks(data, self.data_len, self.parity_len)
    }

    /// Decodes and error-corrects `encoded`, returning the first `original_len`
    /// original bytes (final-chunk zero padding stripped).
    ///
    /// Each `n = data_len + parity_len` block is decoded independently:
    /// syndromes → inversionless Berlekamp-Massey → Chien search → Forney →
    /// correction → **mandatory post-correction syndrome verification**. A block
    /// is accepted only if the result is a valid codeword within the correction
    /// capacity `t = parity_len / 2`; otherwise the decoder returns
    /// [`RsError::Uncorrectable`] and **never** wrong-but-plausible data.
    ///
    /// # Caller responsibilities (raw path)
    ///
    /// The raw stream is **not** self-describing. Two values must match the
    /// encode side; the framed methods ([`Self::encode_framed`] /
    /// [`Self::decode_framed`]) embed them in a header and remove both burdens.
    ///
    /// * **Matching `(parity_len, data_len)`.** Most mismatches fail loud (a
    ///   different `n` gives a wrong block split → [`RsError::InvalidInput`]; a
    ///   larger decode parity set evaluates roots the codeword does not annul →
    ///   [`RsError::Uncorrectable`]). **One family is silent:** the same `n`, the
    ///   same FCR, and `parity_decode <= parity_encode`. Then the decode syndrome
    ///   roots are a *subset* of the encode roots, so every syndrome is genuinely
    ///   zero, the fast path reports "no error", and `data_len` bytes are
    ///   returned with some parity bytes silently surfacing as data.
    ///   Post-correction verification **cannot** catch this — the received word
    ///   *is* a valid codeword of the smaller code.
    ///
    ///   *Worked example:* encode with `RS(255, 223)` (parity 32) and decode with
    ///   `RS(255, 239)` (parity 16) → 16 of the 32 parity bytes are silently
    ///   returned as data. Use identical parameters on both sides, or the framed
    ///   API, to eliminate this footgun.
    ///
    /// * **Correct `original_len`.** Zero padding is indistinguishable from data,
    ///   so the true length cannot be recovered from the stream. For a given
    ///   encoded stream (block count `B`), any `original_len` in
    ///   `((B-1)·data_len, B·data_len]` passes validation; an in-range but
    ///   incorrect value yields a wrong truncation, **not** an error.
    ///
    /// # Errors
    /// [`RsError::Uncorrectable`] when a block exceeds the correction capacity or
    /// fails post-correction verification; [`RsError::InvalidInput`] when
    /// `encoded.len()` is not a whole number of `n`-byte blocks, `original_len`
    /// is inconsistent with the block geometry, or length arithmetic would
    /// overflow.
    pub fn decode(&self, encoded: &[u8], original_len: usize) -> Result<Vec<u8>, RsError> {
        crate::decode::decode_blocks(encoded, original_len, self.data_len, self.parity_len)
    }

    /// Number of parity bytes appended per `data_len`-byte chunk.
    pub(crate) fn parity_len(&self) -> usize {
        self.parity_len
    }

    /// Number of data bytes per chunk (`k` in RS(*n*, *k*)).
    pub(crate) fn data_len(&self) -> usize {
        self.data_len
    }

    /// Encode with a self-describing, CRC-checked header (see crate docs). The
    /// header carries the code parameters and original length so `decode_framed`
    /// rejects a parameter mismatch instead of silently mis-decoding, closing the
    /// caller footguns of a parameter mismatch and an externally tracked
    /// `original_len`.
    ///
    /// # Errors
    /// [`RsError::InvalidInput`] if the framed output allocation fails; otherwise
    /// propagates any error from [`Self::encode`].
    pub fn encode_framed(&self, data: &[u8]) -> Result<Vec<u8>, RsError> {
        crate::frame::encode_framed(self, data)
    }

    /// Decode a framed stream produced by [`Self::encode_framed`]. The embedded
    /// CRC-checked header removes the parameter-match and `original_len` footguns
    /// of the raw [`Self::decode`] path: a code-parameter mismatch or a corrupted
    /// header is rejected as [`RsError::InvalidInput`], never silently mis-decoded.
    ///
    /// # Limitations (v0.1.0)
    /// The 17-byte header is CRC-**checked** but not FEC-**corrected**, so a
    /// single bit-flip anywhere in the header makes the whole frame fail with
    /// [`RsError::InvalidInput`] — the payload itself remains FEC-protected. An
    /// embedded `original_len` that does not fit the platform `usize` (only
    /// possible on 32-bit targets) is rejected, never silently truncated. A
    /// FEC-protected header is a possible future hardening.
    ///
    /// # Errors
    /// [`RsError::InvalidInput`] on a short / bad-magic / bad-version / CRC-fail
    /// header or a code-parameter mismatch; [`RsError::Uncorrectable`] when a
    /// block exceeds the correction capacity.
    pub fn decode_framed(&self, framed: &[u8]) -> Result<Vec<u8>, RsError> {
        crate::frame::decode_framed(self, framed)
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

    #[test]
    fn default_round_trips_short_payload() {
        let rs = ReedSolomon::default();
        let msg = b"the quick brown fox";
        let enc = rs.encode(msg).unwrap();
        let dec = rs.decode(&enc, msg.len()).unwrap();
        assert_eq!(dec, msg);
    }

    #[test]
    fn default_recovers_16_errors_per_block() {
        let rs = ReedSolomon::default();
        let msg = vec![0xABu8; 223];
        let mut enc = rs.encode(&msg).unwrap();
        for i in 0..16 {
            enc[i * 3] ^= 0x5A;
        } // 16 errors in the single block
        assert_eq!(rs.decode(&enc, msg.len()).unwrap(), msg);
    }

    #[test]
    fn public_framed_methods_round_trip() {
        let rs = ReedSolomon::default();
        let msg = b"public framed api";
        let framed = rs.encode_framed(msg).unwrap();
        assert_eq!(rs.decode_framed(&framed).unwrap(), msg);
    }

    #[test]
    fn default_fails_loud_on_17_errors() {
        let rs = ReedSolomon::default();
        let msg = vec![0x11u8; 223];
        let mut enc = rs.encode(&msg).unwrap();
        for byte in enc.iter_mut().take(17) {
            *byte ^= 0xFF;
        }
        assert!(matches!(
            rs.decode(&enc, msg.len()),
            Err(RsError::Uncorrectable(_))
        ));
    }
}

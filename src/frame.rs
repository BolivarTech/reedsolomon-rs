// Author: Julian Bolivar
// Version: 0.1.0
// Date: 2026-06-29

//! Self-describing framed format: a CRC-checked header carrying
//! `(version, parity_len, data_len, original_len)` wraps the raw codewords so
//! `decode_framed` rejects parameter mismatch and reads the true length.

use crate::crc::crc32;
use crate::{ReedSolomon, RsError};

/// Frame magic bytes (`"RS"`).
pub(crate) const FRAME_MAGIC: [u8; 2] = [0x52, 0x53];
/// Frame format version.
pub(crate) const FRAME_VERSION: u8 = 1;
/// Fixed header length: magic(2) + version(1) + parity(1) + data(1) + len(8) + crc(4).
pub(crate) const FRAME_HEADER_LEN: usize = 17;

/// Encode `data` with a self-describing header.
pub(crate) fn encode_framed(rs: &ReedSolomon, data: &[u8]) -> Result<Vec<u8>, RsError> {
    let body = rs.encode(data)?;
    let mut header = [0u8; FRAME_HEADER_LEN];
    header[0] = FRAME_MAGIC[0];
    header[1] = FRAME_MAGIC[1];
    header[2] = FRAME_VERSION;
    header[3] = rs.parity_len() as u8; // parity_len, data_len are always <= 254
    header[4] = rs.data_len() as u8;
    header[5..13].copy_from_slice(&(data.len() as u64).to_be_bytes());
    let crc = crc32(&header[..13]);
    header[13..17].copy_from_slice(&crc.to_be_bytes());

    let mut out = Vec::new();
    out.try_reserve(FRAME_HEADER_LEN + body.len())
        .map_err(|_| RsError::InvalidInput("framed allocation too large".into()))?;
    out.extend_from_slice(&header);
    out.extend_from_slice(&body);
    Ok(out)
}

/// Decode a framed stream; reject header/parameter inconsistencies as `InvalidInput`.
pub(crate) fn decode_framed(rs: &ReedSolomon, framed: &[u8]) -> Result<Vec<u8>, RsError> {
    if framed.len() < FRAME_HEADER_LEN {
        return Err(RsError::InvalidInput("framed shorter than header".into()));
    }
    let h = &framed[..FRAME_HEADER_LEN];
    if h[0..2] != FRAME_MAGIC {
        return Err(RsError::InvalidInput("bad frame magic".into()));
    }
    let stored = u32::from_be_bytes([h[13], h[14], h[15], h[16]]);
    if crc32(&h[..13]) != stored {
        return Err(RsError::InvalidInput("frame header CRC mismatch".into()));
    }
    if h[3] as usize != rs.parity_len() || h[4] as usize != rs.data_len() {
        return Err(RsError::InvalidInput(
            "frame codec parameter mismatch".into(),
        ));
    }
    let original_len_u64 = u64::from_be_bytes(h[5..13].try_into().expect("8 bytes"));
    // Reject (never silently truncate) a length that does not fit usize — only
    // possible on 32-bit targets where usize < u64.
    let original_len = usize::try_from(original_len_u64)
        .map_err(|_| RsError::InvalidInput("framed original_len exceeds usize".into()))?;
    rs.decode(&framed[FRAME_HEADER_LEN..], original_len)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ReedSolomon;

    #[test]
    fn framed_round_trips() {
        let rs = ReedSolomon::default();
        let msg = b"framed payload bytes";
        let framed = encode_framed(&rs, msg).unwrap();
        assert_eq!(&framed[0..2], &FRAME_MAGIC);
        assert_eq!(decode_framed(&rs, &framed).unwrap(), msg);
    }

    #[test]
    fn framed_rejects_parameter_mismatch() {
        let enc_rs = ReedSolomon::default(); // (32, 223)
        let framed = encode_framed(&enc_rs, b"hello").unwrap();
        let dec_rs = ReedSolomon::new(16, 239).unwrap(); // subset-roots silent case
        assert!(matches!(
            decode_framed(&dec_rs, &framed),
            Err(crate::RsError::InvalidInput(_))
        ));
    }

    #[test]
    fn framed_rejects_short_input() {
        let rs = ReedSolomon::default();
        assert!(matches!(
            decode_framed(&rs, &[0u8; 4]),
            Err(crate::RsError::InvalidInput(_))
        ));
    }

    #[test]
    fn framed_rejects_corrupted_header() {
        let rs = ReedSolomon::default();
        let msg = b"hello";
        let mut framed = encode_framed(&rs, msg).unwrap();
        // Flip the low byte of `original_len` (5 -> 4): a still-valid length that,
        // absent the header CRC, would silently mis-decode to "hell". The CRC must
        // catch this and reject the frame as `InvalidInput`.
        framed[12] ^= 0x01;
        assert!(matches!(
            decode_framed(&rs, &framed),
            Err(crate::RsError::InvalidInput(_))
        ));
    }

    #[test]
    fn framed_rejects_bad_magic() {
        let rs = ReedSolomon::default();
        let mut framed = encode_framed(&rs, b"hello").unwrap();
        framed[0] = 0x00; // corrupt magic, then re-seal the CRC so only the
        let crc = crate::crc::crc32(&framed[..13]); // magic check can reject it.
        framed[13..17].copy_from_slice(&crc.to_be_bytes());
        assert!(matches!(
            decode_framed(&rs, &framed),
            Err(crate::RsError::InvalidInput(_))
        ));
    }

    #[test]
    fn framed_rejects_unsupported_version() {
        let rs = ReedSolomon::default();
        let mut framed = encode_framed(&rs, b"hello").unwrap();
        framed[2] = FRAME_VERSION + 1; // future version, then re-seal the CRC so
        let crc = crate::crc::crc32(&framed[..13]); // only the version check rejects.
        framed[13..17].copy_from_slice(&crc.to_be_bytes());
        assert!(matches!(
            decode_framed(&rs, &framed),
            Err(crate::RsError::InvalidInput(_))
        ));
    }
}

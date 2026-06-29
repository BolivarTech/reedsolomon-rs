// Author: Julian Bolivar
// Version: 0.1.0
// Date: 2026-06-29

//! Self-describing framed format: a CRC-checked header carrying
//! `(version, parity_len, data_len, original_len)` wraps the raw codewords so
//! `decode_framed` rejects parameter mismatch and reads the true length.

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
}

// Author: Julian Bolivar
// Version: 0.1.0
// Date: 2026-06-29

//! Systematic Reed-Solomon encoder over GF(2^8).

use crate::gf256;
use crate::poly;
use crate::RsError;
use crate::FCR;

/// Encoded length `ceil(len/data_len) * (data_len + parity_len)` with checked
/// arithmetic; `None` on `usize` overflow. `len == 0` → `Some(0)`.
///
/// # Parameters
/// - `len`: total number of input bytes.
/// - `data_len`: number of data bytes per block.
/// - `parity_len`: number of parity bytes per block.
pub(crate) fn encoded_len(len: usize, data_len: usize, parity_len: usize) -> Option<usize> {
    if len == 0 {
        return Some(0);
    }
    let blocks = len.checked_add(data_len.checked_sub(1)?)? / data_len; // ceil division
    let n = data_len.checked_add(parity_len)?;
    blocks.checked_mul(n)
}

/// Systematic encode of `data` into `data_len`+`parity_len`-byte codewords.
///
/// Each `data_len`-byte chunk is zero-padded then paired with `parity_len` parity
/// bytes computed as `remainder(message(x)·x^parity_len, g(x))`. The output is
/// the concatenation of all codewords (data bytes verbatim, then parity bytes).
/// Empty input produces empty output.
///
/// # Errors
/// [`RsError::InvalidInput`] if the encoded length overflows `usize` or if
/// the output `Vec` allocation fails.
pub(crate) fn encode_blocks(
    data: &[u8],
    data_len: usize,
    parity_len: usize,
) -> Result<Vec<u8>, RsError> {
    let total = encoded_len(data.len(), data_len, parity_len).ok_or_else(|| {
        RsError::InvalidInput(format!(
            "encoded length overflows usize (len={}, data_len={data_len}, parity_len={parity_len})",
            data.len()
        ))
    })?;
    let mut out = Vec::new();
    out.try_reserve(total)
        .map_err(|_| RsError::InvalidInput("output allocation too large".into()))?;
    if data.is_empty() {
        return Ok(out);
    }
    let g = build_generator(parity_len);
    for chunk in data.chunks(data_len) {
        // zero-pad chunk to data_len (high-degree data, then parity)
        let mut block = vec![0u8; data_len + parity_len];
        block[..chunk.len()].copy_from_slice(chunk);
        // remainder of message(x)*x^parity_len mod g(x): the data sits in the
        // high `data_len` positions, the low `parity_len` are 0 before division.
        let parity = poly::remainder(&block, &g);
        // poly::remainder returns exactly divisor.len()-1 == parity_len bytes (fixed-range slice, no normalisation).
        debug_assert_eq!(
            parity.len(),
            parity_len,
            "poly::remainder must return parity_len bytes"
        );
        block[data_len..].copy_from_slice(&parity);
        out.extend_from_slice(&block);
    }
    Ok(out)
}

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
    fn encode_is_systematic_and_right_length() {
        let data = [1u8, 2, 3, 4, 5]; // data_len=11 => one zero-padded block
        let out = encode_blocks(&data, 11, 4).unwrap();
        assert_eq!(out.len(), 15, "B*n = 1*(11+4)");
        assert_eq!(&out[..5], &data, "data preserved verbatim");
        assert_eq!(&out[5..11], &[0u8; 6], "tail zero-padding before parity");
    }

    #[test]
    fn encode_empty_is_empty() {
        assert_eq!(encode_blocks(&[], 11, 4).unwrap(), Vec::<u8>::new());
    }

    #[test]
    fn encode_multiblock_length() {
        let data = vec![7u8; 25]; // data_len=11 => ceil(25/11)=3 blocks
        let out = encode_blocks(&data, 11, 4).unwrap();
        assert_eq!(out.len(), 3 * 15);
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

    #[test]
    fn encoded_len_zero_data_len_returns_none() {
        // data_len=0 must return None rather than underflow-panicking on `data_len - 1`.
        assert!(encoded_len(10, 0, 4).is_none());
    }
}

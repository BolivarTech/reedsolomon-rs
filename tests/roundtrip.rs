// Author: Julian Bolivar
// Version: 0.2.0
// Date: 2026-07-01

//! Property-based tests for the Reed-Solomon codec.
//!
//! These exercise the *already-implemented* public contract with wide random
//! exploration (`proptest`), complementing the fixed KAT and unit vectors:
//!
//! * clean round-trip for arbitrary data (default and custom parameters),
//! * exact recovery of up to `t = parity_len / 2` errors per block,
//! * the **never-mis-correct** invariant: a decode that returns `Ok` must yield
//!   a valid codeword within Hamming distance `t` of the received block,
//!   otherwise it must return [`RsError::Uncorrectable`].

use proptest::prelude::*;
use reedsolomon::{ReedSolomon, RsError, MAX_BLOCK_SIZE};

/// Byte pattern XORed into a codeword to inject a recoverable error.
const ERROR_MASK_RECOVERABLE: u8 = 0x5A;
/// Byte pattern XORed into a codeword to inject an over-capacity error.
const ERROR_MASK_OVERFLOW: u8 = 0xA5;
/// LCG multiplier (Knuth MMIX) for deterministic per-case error positions.
const LCG_MULT: u64 = 6364136223846793005;
/// Default codeword length RS(255, 223).
const DEFAULT_N: usize = 255;
/// Correction capacity of the default code (`t = parity_len / 2`).
const DEFAULT_T: usize = 16;
/// Error count guaranteed to exceed the default capacity (`> t`).
const OVER_CAPACITY_ERRORS: usize = 40;

proptest! {
    #![proptest_config(ProptestConfig::with_cases(512))] // wider exploration

    /// Any byte slice survives a clean encode/decode round-trip under the
    /// default RS(255, 223) code.
    #[test]
    fn roundtrip_default(data in proptest::collection::vec(any::<u8>(), 0..2000)) {
        let rs = ReedSolomon::default();
        let enc = rs.encode(&data).unwrap();
        prop_assert_eq!(rs.decode(&enc, data.len()).unwrap(), data);
    }

    /// Round-trip holds for arbitrary valid custom `(parity_len, data_len)`.
    #[test]
    fn roundtrip_custom_params(
        parity in 1usize..40, extra in 0usize..40,
        data in proptest::collection::vec(any::<u8>(), 0..600),
    ) {
        let data_len = extra + 1;
        prop_assume!(parity + data_len <= MAX_BLOCK_SIZE);
        let rs = ReedSolomon::new(parity, data_len).unwrap();
        let enc = rs.encode(&data).unwrap();
        prop_assert_eq!(rs.decode(&enc, data.len()).unwrap(), data);
    }

    /// Up to `t = 16` corrupted bytes in a single (padded) block are recovered
    /// exactly.
    #[test]
    fn recovers_up_to_t_errors(
        data in proptest::collection::vec(any::<u8>(), 1..223),
        seed in any::<u64>(),
    ) {
        let rs = ReedSolomon::default();
        let mut enc = rs.encode(&data).unwrap();
        let mut s = seed;
        for _ in 0..DEFAULT_T {
            s = s.wrapping_mul(LCG_MULT).wrapping_add(1);
            let pos = (s >> 33) as usize % DEFAULT_N;
            enc[pos] ^= ERROR_MASK_RECOVERABLE;
        }
        prop_assert_eq!(rs.decode(&enc, data.len()).unwrap(), data);
    }

    /// Over-capacity corruption never yields wrong-but-plausible data: decode
    /// either declares [`RsError::Uncorrectable`] or returns a value that
    /// re-encodes to within distance `t` of the received block.
    #[test]
    fn no_garbage_invariant(
        data in proptest::collection::vec(any::<u8>(), 1..223),
        s0 in any::<u64>(),
    ) {
        let rs = ReedSolomon::default();
        let mut enc = rs.encode(&data).unwrap();
        let mut s = s0;
        for _ in 0..OVER_CAPACITY_ERRORS {
            s = s.wrapping_mul(LCG_MULT).wrapping_add(1);
            enc[(s >> 33) as usize % DEFAULT_N] ^= ERROR_MASK_OVERFLOW;
        }
        match rs.decode(&enc, data.len()) {
            Err(RsError::Uncorrectable(_)) => {}
            Err(RsError::InvalidInput(_)) => prop_assert!(false, "structural input was valid"),
            Ok(d) => {
                let re = rs.encode(&d).unwrap();
                let dist = re.iter().zip(enc.iter()).filter(|(a, b)| a != b).count();
                prop_assert!(dist <= DEFAULT_T, "Ok result must be within t of the received block");
            }
        }
    }
}

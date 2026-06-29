// Author: Julian Bolivar
// Version: 0.1.0
// Date: 2026-06-29

//! CRC-32/IEEE for the framed-format header integrity check (no external dep).

const fn build_table() -> [u32; 256] {
    let mut table = [0u32; 256];
    let mut i = 0usize;
    while i < 256 {
        let mut c = i as u32;
        let mut k = 0;
        while k < 8 {
            c = if c & 1 != 0 {
                0xEDB8_8320 ^ (c >> 1)
            } else {
                c >> 1
            };
            k += 1;
        }
        table[i] = c;
        i += 1;
    }
    table
}
const TABLE: [u32; 256] = build_table();

/// CRC-32/IEEE (reflected, init `0xFFFFFFFF`, final XOR `0xFFFFFFFF`).
///
/// Used by `src/frame.rs` (Task 11B) to integrity-check the framed-format
/// header.
// used by src/frame.rs (Task 11B)
#[allow(dead_code)]
pub(crate) fn crc32(data: &[u8]) -> u32 {
    let mut crc = 0xFFFF_FFFFu32;
    for &b in data {
        crc = TABLE[((crc ^ b as u32) & 0xFF) as usize] ^ (crc >> 8);
    }
    crc ^ 0xFFFF_FFFF
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn crc32_ieee_check_value() {
        assert_eq!(crc32(b"123456789"), 0xCBF4_3926);
    }

    #[test]
    fn crc32_empty_returns_zero() {
        assert_eq!(crc32(b""), 0);
    }
}

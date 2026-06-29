// Author: Julian Bolivar
// Version: 0.1.0
// Date: 2026-06-29

//! CRC-32/IEEE for the framed-format header integrity check (no external dep).

// used by src/frame.rs (Task 11B)
#[allow(dead_code)]
pub(crate) fn crc32(_data: &[u8]) -> u32 {
    0
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn crc32_ieee_check_value() {
        assert_eq!(crc32(b"123456789"), 0xCBF4_3926);
    }
}

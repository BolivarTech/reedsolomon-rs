// Author: Julian Bolivar
// Version: 0.1.0
// Date: 2026-06-29

//! Reed-Solomon decoder pipeline: syndromes → inversionless Berlekamp-Massey
//! → Chien search → Forney → correct → post-correction syndrome verification.

#[cfg(test)]
mod tests {
    #[test]
    fn clean_codeword_has_zero_syndromes() {
        let data = [9u8, 8, 7, 6, 5, 4, 3, 2, 1, 0, 11];
        let enc = crate::encode::encode_blocks(&data, 11, 4).unwrap();
        let s = crate::decode::syndromes(&enc, 4);
        assert!(crate::decode::all_zero(&s), "valid codeword => zero syndromes");
    }
}

// Author: Julian Bolivar
// Version: 0.1.0
// Date: 2026-06-29

//! Systematic Reed-Solomon encoder over GF(2^8).

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
}

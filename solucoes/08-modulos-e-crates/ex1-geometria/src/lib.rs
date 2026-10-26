//! Exercício 1: a árvore de arquivos de `geometria`.

pub mod geometria;

#[cfg(test)]
mod tests {
    use super::geometria::{circulo, quadrado};

    #[test]
    fn areas() {
        assert!((circulo::area(1.0) - std::f64::consts::PI).abs() < 1e-12);
        assert_eq!(quadrado::area(3.0), 9.0);
    }
}

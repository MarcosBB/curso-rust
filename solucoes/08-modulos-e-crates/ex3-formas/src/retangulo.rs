//! Retângulos.

use crate::area::Area;

/// Um retângulo.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Retangulo {
    /// A largura.
    pub largura: f64,
    /// A altura.
    pub altura: f64,
}

impl Area for Retangulo {
    fn area(&self) -> f64 {
        self.largura * self.altura
    }
}

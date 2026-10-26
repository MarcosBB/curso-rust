//! Círculos.

use crate::area::Area;

/// Um círculo.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Circulo {
    /// O raio.
    pub raio: f64,
}

impl Area for Circulo {
    fn area(&self) -> f64 {
        std::f64::consts::PI * self.raio * self.raio
    }
}

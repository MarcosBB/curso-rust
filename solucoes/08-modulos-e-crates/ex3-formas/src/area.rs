//! A trait `Area` e funções sobre ela.

/// Uma forma com área.
pub trait Area {
    /// A área da forma.
    fn area(&self) -> f64;
}

/// Soma das áreas de várias formas, de tipos diferentes.
pub fn area_total(formas: &[&dyn Area]) -> f64 {
    formas.iter().map(|f| f.area()).sum()
}

//! Uma carteira que soma valores. Depende de `moeda` 1.0.

/// Reexporta a versão de `moeda` que esta crate usa, para quem chama
/// conseguir criar exatamente o tipo que as funções esperam.
pub use moeda;

use moeda::Real;

/// Soma dois valores.
pub fn somar(a: Real, b: Real) -> Real {
    Real(a.0 + b.0)
}

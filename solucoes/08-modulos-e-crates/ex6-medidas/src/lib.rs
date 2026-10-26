//! Medidas: comprimentos, partilhas e percentuais (exercício 6 do Módulo 08).
//!
//! ```
//! use medidas::{Percentual, dividir};
//!
//! let partes = dividir(100, 3);
//! assert_eq!(partes, vec![34, 33, 33]);
//!
//! let desconto = Percentual::new(10).expect("10 é válido");
//! assert_eq!(desconto.aplicar(250.0), 25.0);
//! ```

use std::num::ParseFloatError;

/// Lê um comprimento em metros, com ou sem o sufixo `m`.
///
/// # Errors
///
/// Devolve o [`ParseFloatError`] se o texto, sem o `m`, não for um número.
///
/// # Examples
///
/// ```
/// assert_eq!(medidas::parse_metros("12.5m")?, 12.5);
/// assert_eq!(medidas::parse_metros(" 3 ")?, 3.0);
/// assert!(medidas::parse_metros("doze").is_err());
/// # Ok::<(), std::num::ParseFloatError>(())
/// ```
pub fn parse_metros(s: &str) -> Result<f64, ParseFloatError> {
    let s = s.trim();
    s.strip_suffix('m').unwrap_or(s).trim().parse()
}

/// Divide `total` em `partes` inteiras, o mais iguais possível. As primeiras
/// partes recebem a sobra, então a soma é sempre `total`.
///
/// # Panics
///
/// Entra em *panic* se `partes` for zero.
///
/// ```should_panic
/// medidas::dividir(10, 0);
/// ```
///
/// # Examples
///
/// ```
/// assert_eq!(medidas::dividir(10, 3), vec![4, 3, 3]);
/// assert_eq!(medidas::dividir(2, 4), vec![1, 1, 0, 0]);
/// ```
pub fn dividir(total: u32, partes: u32) -> Vec<u32> {
    assert!(partes > 0, "não dá para dividir em zero partes");
    let base = total / partes;
    let sobra = total % partes;
    (0..partes).map(|i| base + u32::from(i < sobra)).collect()
}

/// Um percentual de 0 a 100.
///
/// O campo é privado: todo `Percentual` passou por [`Percentual::new`], que
/// valida. Por isso isto não compila:
///
/// ```compile_fail
/// let p = medidas::Percentual(150);
/// ```
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Percentual(u8);

impl Percentual {
    /// Cria um percentual, ou `None` se `valor` passar de 100.
    ///
    /// ```
    /// assert!(medidas::Percentual::new(100).is_some());
    /// assert!(medidas::Percentual::new(101).is_none());
    /// ```
    pub fn new(valor: u8) -> Option<Percentual> {
        (valor <= 100).then_some(Percentual(valor))
    }

    /// O valor, de 0 a 100.
    pub fn valor(self) -> u8 {
        self.0
    }

    /// Quanto este percentual representa de `x`.
    ///
    /// ```
    /// let p = medidas::Percentual::new(50).unwrap();
    /// assert_eq!(p.aplicar(8.0), 4.0);
    /// ```
    pub fn aplicar(self, x: f64) -> f64 {
        x * f64::from(self.0) / 100.0
    }
}

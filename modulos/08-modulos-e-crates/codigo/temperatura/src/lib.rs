//! Conversões de temperatura: o exemplo de documentação da lição 09.
//!
//! As funções trabalham com graus Celsius em `f64`. Para valores que não
//! podem ficar abaixo do zero absoluto, use [`Kelvin`].
//!
//! ```
//! use temperatura::{ler, para_fahrenheit};
//!
//! let c = ler("36.5")?;
//! assert_eq!(para_fahrenheit(c), 97.7);
//! # Ok::<(), std::num::ParseFloatError>(())
//! ```

use std::num::ParseFloatError;

/// Converte graus Celsius para Fahrenheit.
///
/// # Examples
///
/// ```
/// assert_eq!(temperatura::para_fahrenheit(100.0), 212.0);
/// assert_eq!(temperatura::para_fahrenheit(-40.0), -40.0);
/// ```
pub fn para_fahrenheit(celsius: f64) -> f64 {
    celsius * 9.0 / 5.0 + 32.0
}

/// Lê uma temperatura em Celsius, aceitando espaços e um `°C` opcional no fim.
///
/// Veja também [`para_fahrenheit`].
///
/// # Errors
///
/// Devolve o [`ParseFloatError`] do `parse` se o que sobrar não for um número.
///
/// # Examples
///
/// ```
/// use temperatura::ler;
///
/// assert_eq!(ler(" 21.5 °C")?, 21.5);
/// assert!(ler("quente").is_err());
/// # Ok::<(), std::num::ParseFloatError>(())
/// ```
pub fn ler(texto: &str) -> Result<f64, ParseFloatError> {
    let texto = texto.trim();
    let texto = texto.strip_suffix("°C").unwrap_or(texto);
    texto.trim().parse()
}

/// Média de uma lista de leituras.
///
/// # Panics
///
/// Entra em *panic* se `leituras` estiver vazia: média de nada é um bug de
/// quem chamou, não uma condição esperada.
///
/// ```should_panic
/// temperatura::media(&[]);
/// ```
///
/// # Examples
///
/// ```
/// assert_eq!(temperatura::media(&[20.0, 22.0, 27.0]), 23.0);
/// ```
pub fn media(leituras: &[f64]) -> f64 {
    assert!(!leituras.is_empty(), "média de uma lista vazia");
    leituras.iter().sum::<f64>() / leituras.len() as f64
}

/// Lê um arquivo com uma temperatura por linha.
///
/// # Errors
///
/// Devolve o erro de E/S se o arquivo não puder ser lido. Linhas que não são
/// números são ignoradas.
///
/// # Examples
///
/// O exemplo compila, mas não roda (`no_run`), porque o arquivo não existe:
///
/// ```no_run
/// let leituras = temperatura::ler_arquivo("sensor.txt")?;
/// println!("{} leituras", leituras.len());
/// # Ok::<(), std::io::Error>(())
/// ```
pub fn ler_arquivo(caminho: &str) -> std::io::Result<Vec<f64>> {
    let texto = std::fs::read_to_string(caminho)?;
    Ok(texto.lines().filter_map(|l| ler(l).ok()).collect())
}

/// Uma temperatura em kelvin, nunca abaixo do zero absoluto.
///
/// O campo é privado, então a única forma de criar um `Kelvin` é por
/// [`Kelvin::new`], que valida. Por isso isto não compila:
///
/// ```compile_fail
/// let k = temperatura::Kelvin(-10.0);
/// ```
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Kelvin(f64);

impl Kelvin {
    /// Cria um `Kelvin`, ou `None` se o valor for negativo (ou `NaN`).
    ///
    /// ```
    /// use temperatura::Kelvin;
    ///
    /// assert!(Kelvin::new(273.15).is_some());
    /// assert!(Kelvin::new(-1.0).is_none());
    /// ```
    pub fn new(valor: f64) -> Option<Kelvin> {
        (valor >= 0.0).then_some(Kelvin(valor))
    }

    /// O valor em graus Celsius.
    ///
    /// ```
    /// let k = temperatura::Kelvin::new(0.0).unwrap();
    /// assert_eq!(k.para_celsius(), -273.15);
    /// ```
    pub fn para_celsius(self) -> f64 {
        self.0 - 273.15
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn le_com_e_sem_sufixo() {
        assert_eq!(ler("10"), Ok(10.0));
        assert_eq!(ler("10°C"), Ok(10.0));
    }
}

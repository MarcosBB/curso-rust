//! Solução do exercício 1 do Módulo 09.

/// O menor valor possível em graus Celsius.
pub const ZERO_ABSOLUTO: f64 = -273.15;

#[derive(Debug)]
pub struct Termometro {
    celsius: f64,
}

#[derive(Debug)]
pub enum ErroTemperatura {
    /// O texto não é um número.
    NaoNumero(String),
    /// O número está abaixo do zero absoluto.
    AbaixoDoZeroAbsoluto(f64),
}

impl Termometro {
    /// Cria um termômetro.
    ///
    /// # Panics
    ///
    /// Se `celsius` estiver abaixo do zero absoluto.
    pub fn new(celsius: f64) -> Termometro {
        assert!(
            celsius >= ZERO_ABSOLUTO,
            "abaixo do zero absoluto: {celsius}"
        );
        Termometro { celsius }
    }

    pub fn fahrenheit(&self) -> f64 {
        self.celsius * 9.0 / 5.0 + 32.0
    }
}

/// Lê uma temperatura em Celsius de um texto.
///
/// # Errors
///
/// [`ErroTemperatura::NaoNumero`] se o texto não for número, e
/// [`ErroTemperatura::AbaixoDoZeroAbsoluto`] se for menor que -273,15.
pub fn ler(texto: &str) -> Result<Termometro, ErroTemperatura> {
    let texto = texto.trim();
    let celsius: f64 = texto
        .parse()
        .map_err(|_| ErroTemperatura::NaoNumero(texto.to_string()))?;
    if celsius < ZERO_ABSOLUTO {
        return Err(ErroTemperatura::AbaixoDoZeroAbsoluto(celsius));
    }
    Ok(Termometro::new(celsius))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::assert_matches;

    #[test]
    #[should_panic(expected = "abaixo do zero absoluto")]
    fn abaixo_do_zero_absoluto() {
        Termometro::new(-300.0);
    }

    #[test]
    fn texto_invalido() {
        assert_matches!(ler("abc"), Err(ErroTemperatura::NaoNumero(t)) if t == "abc");
    }

    #[test]
    fn muito_frio() {
        assert_matches!(
            ler("-300"),
            Err(ErroTemperatura::AbaixoDoZeroAbsoluto(c)) if c == -300.0
        );
    }

    #[test]
    fn agua_fervendo() -> Result<(), ErroTemperatura> {
        let t = ler("100")?;
        assert!((t.fahrenheit() - 212.0).abs() < 1e-9);
        Ok(())
    }
}

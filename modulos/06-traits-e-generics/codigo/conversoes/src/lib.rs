//! Exemplos da lição 05: `From`, `Into`, `TryFrom`, `FromStr` e `AsRef`.

use std::fmt;
use std::num::ParseIntError;
use std::path::Path;
use std::str::FromStr;

/// Um percentual de 0 a 100. Só dá para criar por `TryFrom` ou `parse`,
/// que validam a faixa.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Percentual(u8);

/// O que pode dar errado ao criar um `Percentual`.
#[derive(Debug, PartialEq, Eq)]
pub enum ErroPercentual {
    ForaDaFaixa(i64),
    SemSimbolo,
    NaoNumerico(ParseIntError),
}

impl fmt::Display for ErroPercentual {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ErroPercentual::ForaDaFaixa(v) => write!(f, "{v} está fora de 0..=100"),
            ErroPercentual::SemSimbolo => write!(f, "faltou o símbolo %"),
            ErroPercentual::NaoNumerico(_) => write!(f, "não é um número"),
        }
    }
}

impl std::error::Error for ErroPercentual {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ErroPercentual::NaoNumerico(e) => Some(e),
            _ => None,
        }
    }
}

/// `?` em `parse::<i64>()` converte o `ParseIntError` por aqui.
impl From<ParseIntError> for ErroPercentual {
    fn from(e: ParseIntError) -> Self {
        ErroPercentual::NaoNumerico(e)
    }
}

/// Conversão que pode falhar: de qualquer inteiro de 64 bits.
impl TryFrom<i64> for Percentual {
    type Error = ErroPercentual;

    fn try_from(valor: i64) -> Result<Self, Self::Error> {
        match valor {
            0..=100 => Ok(Percentual(valor as u8)), // a faixa garante que cabe em u8
            _ => Err(ErroPercentual::ForaDaFaixa(valor)),
        }
    }
}

/// Conversão de texto: é o que faz `"75%".parse::<Percentual>()` funcionar.
impl FromStr for Percentual {
    type Err = ErroPercentual;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let numero = s
            .trim()
            .strip_suffix('%')
            .ok_or(ErroPercentual::SemSimbolo)?;
        let valor: i64 = numero.parse()?; // ParseIntError -> ErroPercentual
        Percentual::try_from(valor)
    }
}

/// Conversão que nunca falha nem perde informação: `Percentual` -> fração.
impl From<Percentual> for f64 {
    fn from(p: Percentual) -> f64 {
        f64::from(p.0) / 100.0
    }
}

impl fmt::Display for Percentual {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}%", self.0)
    }
}

/// Peça `Into` no parâmetro: aceita `&str`, `String`, `Box<str>`...
pub fn saudar(nome: impl Into<String>) -> String {
    let mut s: String = nome.into();
    s.insert_str(0, "Olá, ");
    s
}

/// `AsRef<Path>`: aceita `&str`, `String`, `&Path`, `PathBuf`...
pub fn extensao(caminho: impl AsRef<Path>) -> Option<String> {
    let ext = caminho.as_ref().extension()?;
    Some(ext.to_string_lossy().into_owned())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn try_from_valida_a_faixa() {
        assert_eq!(Percentual::try_from(75), Ok(Percentual(75)));
        assert_eq!(
            Percentual::try_from(150),
            Err(ErroPercentual::ForaDaFaixa(150))
        );
        // `try_into` vem de graça, e o tipo de destino vem da anotação.
        let p: Result<Percentual, _> = (-1_i64).try_into();
        assert_eq!(p, Err(ErroPercentual::ForaDaFaixa(-1)));
    }

    #[test]
    fn parse_usa_from_str() {
        assert_eq!("42%".parse::<Percentual>(), Ok(Percentual(42)));
        assert_eq!("42".parse::<Percentual>(), Err(ErroPercentual::SemSimbolo));
        assert!(matches!(
            "abc%".parse::<Percentual>(),
            Err(ErroPercentual::NaoNumerico(_))
        ));
    }

    #[test]
    fn from_e_into() {
        let p = Percentual::try_from(25).unwrap();
        let fracao: f64 = p.into(); // `Into` veio do `impl From<Percentual> for f64`
        assert_eq!(fracao, 0.25);
        assert_eq!(f64::from(p), 0.25);
    }

    #[test]
    fn into_string_e_as_ref_path() {
        assert_eq!(saudar("Ana"), "Olá, Ana");
        assert_eq!(saudar(String::from("Bia")), "Olá, Bia");
        assert_eq!(extensao("notas.txt"), Some(String::from("txt")));
        assert_eq!(extensao(PathBuf::from("a/b.rs")), Some(String::from("rs")));
        assert_eq!(extensao(String::from("sem_extensao")), None);
    }
}

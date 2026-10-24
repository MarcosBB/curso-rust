//! Exercício 4: conversões de `Porta`.

use std::error::Error;
use std::fmt;
use std::num::ParseIntError;
use std::str::FromStr;

/// Uma porta TCP de usuário: de 1024 a 65535.
/// O campo é privado: só se cria por `try_from` ou `parse`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Porta(u16);

#[derive(Debug, PartialEq, Eq)]
pub enum ErroPorta {
    Zero,
    Reservada(u16),
    NaoNumerica(ParseIntError),
}

impl fmt::Display for ErroPorta {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ErroPorta::Zero => write!(f, "a porta 0 não é válida"),
            ErroPorta::Reservada(p) => write!(f, "a porta {p} é reservada (use 1024 ou mais)"),
            ErroPorta::NaoNumerica(_) => write!(f, "a porta precisa ser um número de 0 a 65535"),
        }
    }
}

impl Error for ErroPorta {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            ErroPorta::NaoNumerica(e) => Some(e),
            ErroPorta::Zero | ErroPorta::Reservada(_) => None,
        }
    }
}

impl From<ParseIntError> for ErroPorta {
    fn from(e: ParseIntError) -> Self {
        ErroPorta::NaoNumerica(e)
    }
}

impl TryFrom<u16> for Porta {
    type Error = ErroPorta;

    fn try_from(valor: u16) -> Result<Self, Self::Error> {
        match valor {
            0 => Err(ErroPorta::Zero),
            1..=1023 => Err(ErroPorta::Reservada(valor)),
            _ => Ok(Porta(valor)),
        }
    }
}

impl FromStr for Porta {
    type Err = ErroPorta;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let numero: u16 = s.trim().parse()?; // ParseIntError -> ErroPorta
        Porta::try_from(numero)
    }
}

impl From<Porta> for u16 {
    fn from(p: Porta) -> u16 {
        p.0
    }
}

impl fmt::Display for Porta {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, ":{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn try_from_valido_e_invalido() {
        assert_eq!(u16::from(Porta::try_from(8080).unwrap()), 8080);
        assert_eq!(Porta::try_from(0), Err(ErroPorta::Zero));
        assert_eq!(Porta::try_from(80), Err(ErroPorta::Reservada(80)));
        let p: Result<Porta, _> = 1024_u16.try_into();
        assert!(p.is_ok());
    }

    #[test]
    fn parse() {
        assert_eq!("8080".parse::<Porta>(), Porta::try_from(8080));
        assert_eq!(" 3000\n".parse::<Porta>(), Porta::try_from(3000));
        assert_eq!("22".parse::<Porta>(), Err(ErroPorta::Reservada(22)));
        assert!(matches!(
            "70000".parse::<Porta>(),
            Err(ErroPorta::NaoNumerica(_))
        ));
        assert!(matches!(
            "http".parse::<Porta>(),
            Err(ErroPorta::NaoNumerica(_))
        ));
    }

    #[test]
    fn into_u16() {
        let p: Porta = "5432".parse().unwrap();
        let n: u16 = p.into();
        assert_eq!(n, 5432);
    }

    #[test]
    fn mensagens_e_causa() {
        assert_eq!(Porta::try_from(9000).unwrap().to_string(), ":9000");
        let e = "abc".parse::<Porta>().unwrap_err();
        assert_eq!(e.to_string(), "a porta precisa ser um número de 0 a 65535");
        assert!(e.source().is_some());
        assert!(ErroPorta::Zero.source().is_none());
    }

    #[test]
    fn interrogacao_com_box_dyn_error() {
        fn ler(texto: &str) -> Result<u16, Box<dyn Error>> {
            let p: Porta = texto.parse()?; // ErroPorta -> Box<dyn Error>
            Ok(p.into())
        }
        assert_eq!(ler("8443").unwrap(), 8443);
        assert_eq!(
            ler("443").unwrap_err().to_string(),
            "a porta 443 é reservada (use 1024 ou mais)"
        );
    }
}

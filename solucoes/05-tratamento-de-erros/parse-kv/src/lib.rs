use std::error::Error;
use std::fmt;
use std::num::ParseIntError;

#[derive(Debug)]
pub enum ParseError {
    /// A linha não tem `=`.
    MissingEquals,
    /// A chave (antes do `=`) está vazia.
    EmptyKey,
    /// O valor (depois do `=`) não é um `i64`.
    BadNumber(ParseIntError),
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::MissingEquals => write!(f, "faltou o `=`"),
            ParseError::EmptyKey => write!(f, "chave vazia"),
            ParseError::BadNumber(_) => write!(f, "valor inválido"),
        }
    }
}

impl Error for ParseError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            ParseError::BadNumber(e) => Some(e),
            ParseError::MissingEquals | ParseError::EmptyKey => None,
        }
    }
}

impl From<ParseIntError> for ParseError {
    fn from(e: ParseIntError) -> Self {
        ParseError::BadNumber(e)
    }
}

/// Interpreta uma linha `chave = valor`, em que o valor é um inteiro.
pub fn parse_kv(line: &str) -> Result<(String, i64), ParseError> {
    let (key, value) = line.split_once('=').ok_or(ParseError::MissingEquals)?;
    let key = key.trim();
    if key.is_empty() {
        return Err(ParseError::EmptyKey);
    }
    let value: i64 = value.trim().parse()?;
    Ok((key.to_string(), value))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error;

    #[test]
    fn linha_valida() {
        assert_eq!(parse_kv("idade = 42").unwrap(), ("idade".to_string(), 42));
        assert_eq!(parse_kv("  x=-7  ").unwrap(), ("x".to_string(), -7));
    }

    #[test]
    fn sem_igual() {
        let e = parse_kv("idade 42").unwrap_err();
        assert!(matches!(e, ParseError::MissingEquals));
        assert_eq!(e.to_string(), "faltou o `=`");
        assert!(e.source().is_none());
    }

    #[test]
    fn chave_vazia() {
        let e = parse_kv("  = 42").unwrap_err();
        assert!(matches!(e, ParseError::EmptyKey));
        assert_eq!(e.to_string(), "chave vazia");
    }

    #[test]
    fn numero_invalido_guarda_a_causa() {
        let e = parse_kv("idade = muitos").unwrap_err();
        assert!(matches!(e, ParseError::BadNumber(_)));
        assert_eq!(e.to_string(), "valor inválido");
        let cause = e.source().expect("BadNumber deveria ter causa");
        assert_eq!(cause.to_string(), "invalid digit found in string");
    }

    #[test]
    fn from_converte_parse_int_error() {
        let pe = "x".parse::<i64>().unwrap_err();
        let e: ParseError = pe.into();
        assert!(matches!(e, ParseError::BadNumber(_)));
    }
}

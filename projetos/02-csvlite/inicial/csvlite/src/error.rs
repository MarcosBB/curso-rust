use std::fmt;

use crate::Record;

/// Um erro de leitura ou de conversão, com a posição em que aconteceu.
///
/// Linhas e colunas começam em 1. A coluna conta caracteres, não bytes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Error {
    /// O que deu errado.
    pub kind: ErrorKind,
    /// Linha do problema.
    pub line: usize,
    /// Coluna do problema.
    pub column: usize,
}

/// Os tipos de erro possíveis.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ErrorKind {
    /// A entrada acabou com aspas abertas. A posição é a da aspa de abertura.
    UnterminatedQuote,
    /// Depois de fechar as aspas veio algo que não é separador nem fim de linha.
    CharAfterQuote(char),
    /// No modo estrito, um registro tem um número de campos diferente do primeiro.
    FieldCount {
        /// Número de campos do primeiro registro.
        expected: usize,
        /// Número de campos deste registro.
        found: usize,
    },
    /// Um campo não pôde ser convertido (usado por [`FromRecord`](crate::FromRecord)).
    InvalidField {
        /// Índice do campo, começando em 0 (como em [`Record::get`]).
        index: usize,
        /// Descrição do problema.
        message: String,
    },
}

impl Error {
    /// Cria um erro [`ErrorKind::InvalidField`] na linha em que `record` começa.
    ///
    /// Feito para implementações de [`FromRecord`](crate::FromRecord).
    /// A coluna é sempre 1 (o registro não guarda a coluna de cada campo).
    pub fn invalid_field(record: &Record, index: usize, message: impl fmt::Display) -> Self {
        // Etapa 6.
        todo!()
    }
}

impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Etapa 3. Mensagens esperadas (veja os testes):
        //   UnterminatedQuote     -> "aspas não fechadas"
        //   CharAfterQuote('y')   -> "caractere 'y' depois de fechar aspas"
        //   FieldCount { 1, 2 }   -> "esperava 1 campo, encontrei 2"
        //   FieldCount { 2, 3 }   -> "esperava 2 campos, encontrei 3"
        //   InvalidField { 1, m } -> "campo 2: <m>"  (o índice 0 é o "campo 1")
        todo!()
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Etapa 3: "linha L, coluna C: <descrição do kind>"
        todo!()
    }
}

impl std::error::Error for Error {}

#[cfg(test)]
mod step3_tests {
    use super::*;

    #[test]
    fn display_with_position() {
        let err = Error {
            kind: ErrorKind::CharAfterQuote('y'),
            line: 2,
            column: 4,
        };
        assert_eq!(
            err.to_string(),
            "linha 2, coluna 4: caractere 'y' depois de fechar aspas"
        );
    }

    #[test]
    fn display_unterminated_quote() {
        let err = Error {
            kind: ErrorKind::UnterminatedQuote,
            line: 2,
            column: 1,
        };
        assert_eq!(err.to_string(), "linha 2, coluna 1: aspas não fechadas");
    }

    #[test]
    fn display_pluralizes_fields() {
        let one = ErrorKind::FieldCount {
            expected: 1,
            found: 2,
        };
        let two = ErrorKind::FieldCount {
            expected: 2,
            found: 3,
        };
        assert_eq!(one.to_string(), "esperava 1 campo, encontrei 2");
        assert_eq!(two.to_string(), "esperava 2 campos, encontrei 3");
    }

    #[test]
    fn is_a_std_error() {
        fn takes_error(_: &dyn std::error::Error) {}
        let err = Error {
            kind: ErrorKind::UnterminatedQuote,
            line: 1,
            column: 1,
        };
        takes_error(&err);
    }
}

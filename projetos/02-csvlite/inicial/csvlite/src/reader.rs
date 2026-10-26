use std::iter::Peekable;
use std::str::Chars;

use crate::{Error, ErrorKind, Record};

/// Opções de leitura.
///
/// Use a sintaxe de atualização para mudar só o que interessa:
/// `Options { delimiter: ';', ..Options::default() }`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Options {
    /// Separador de campos. Padrão: `,`.
    pub delimiter: char,
    /// Se `true`, aceita registros com números de campos diferentes.
    /// Padrão: `false` (modo estrito).
    pub flexible: bool,
}

impl Default for Options {
    fn default() -> Self {
        Options {
            delimiter: ',',
            flexible: false,
        }
    }
}

/// Lê registros de um texto CSV, um de cada vez.
///
/// É um iterador de `Result<Record, Error>`. Depois do primeiro erro, ele
/// não devolve mais nada.
///
/// Os campos abaixo são uma sugestão: mude à vontade.
#[derive(Debug, Clone)]
pub struct Reader<'a> {
    chars: Peekable<Chars<'a>>,
    opts: Options,
    /// Posição do próximo caractere a ser lido (começa em 1).
    line: usize,
    column: usize,
    /// Número de campos do primeiro registro (modo estrito).
    expected: Option<usize>,
    /// Vira `true` depois de um erro: o iterador para.
    done: bool,
}

impl<'a> Reader<'a> {
    /// Cria um leitor sobre `input`.
    pub fn new(input: &'a str, opts: Options) -> Self {
        // Etapa 1. Linhas e colunas começam em 1.
        todo!()
    }

    // Sugestão de métodos auxiliares (privados), um por etapa:
    // - `bump(&mut self) -> Option<char>`: consome um caractere e atualiza
    //   `line`/`column` (etapa 1);
    // - um método para campo sem aspas (etapa 1) e outro para campo entre
    //   aspas (etapa 2), ambos dizendo como o campo terminou (separador ou
    //   fim do registro);
    // - `read_record(&mut self) -> Result<Record, Error>` (etapas 1 e 2);
    // - a verificação do número de campos do modo estrito (etapa 3).
}

impl Iterator for Reader<'_> {
    type Item = Result<Record, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        // Etapa 1: entrada vazia (ou já consumida) devolve `None`.
        // Etapa 3: depois de um `Err`, devolva sempre `None` (campo `done`).
        todo!()
    }
}

#[cfg(test)]
fn read_all(input: &str, opts: Options) -> Result<Vec<Vec<String>>, Error> {
    Reader::new(input, opts)
        .map(|r| r.map(|record| record.fields))
        .collect()
}

#[cfg(test)]
fn read(input: &str) -> Result<Vec<Vec<String>>, Error> {
    read_all(input, Options::default())
}

#[cfg(test)]
mod step1_tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case::simple("a,b\nc,d\n", vec![vec!["a", "b"], vec!["c", "d"]])]
    #[case::no_final_newline("a,b\nc,d", vec![vec!["a", "b"], vec!["c", "d"]])]
    #[case::crlf("a,b\r\nc,d\r\n", vec![vec!["a", "b"], vec!["c", "d"]])]
    #[case::empty_fields(",a,\n,,\n", vec![vec!["", "a", ""], vec!["", "", ""]])]
    #[case::empty_input("", vec![])]
    #[case::single_field("x\ny\n", vec![vec!["x"], vec!["y"]])]
    #[case::unicode("ação,ç\nñ,ü\n", vec![vec!["ação", "ç"], vec!["ñ", "ü"]])]
    fn plain_records(#[case] input: &str, #[case] expected: Vec<Vec<&str>>) {
        assert_eq!(read(input).unwrap(), expected);
    }

    #[test]
    fn other_delimiter() {
        let opts = Options {
            delimiter: ';',
            ..Options::default()
        };
        assert_eq!(read_all("a;b,c\n", opts).unwrap(), vec![vec!["a", "b,c"]]);
    }

    #[test]
    fn records_know_their_line() {
        let lines: Vec<usize> = Reader::new("a\nb\nc\n", Options::default())
            .map(|r| r.unwrap().line)
            .collect();
        assert_eq!(lines, [1, 2, 3]);
    }
}

#[cfg(test)]
mod step2_tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case::delimiter_inside("\"a,b\",c\n", vec![vec!["a,b", "c"]])]
    #[case::doubled_quote("\"diga \"\"oi\"\"\",x\n", vec![vec!["diga \"oi\"", "x"]])]
    #[case::newline_inside("\"linha 1\nlinha 2\",x\n", vec![vec!["linha 1\nlinha 2", "x"]])]
    #[case::empty_quoted("\"\",a\n", vec![vec!["", "a"]])]
    #[case::quoted_at_end_of_input("a,\"b\"", vec![vec!["a", "b"]])]
    #[case::quoted_then_crlf("\"a\"\r\nb\r\n", vec![vec!["a"], vec!["b"]])]
    #[case::quote_in_the_middle("a\"b,c\n", vec![vec!["a\"b", "c"]])]
    fn quoted_fields(#[case] input: &str, #[case] expected: Vec<Vec<&str>>) {
        assert_eq!(read(input).unwrap(), expected);
    }

    #[test]
    fn record_with_newline_counts_lines() {
        let lines: Vec<usize> = Reader::new("\"a\nb\"\nc\n", Options::default())
            .map(|r| r.unwrap().line)
            .collect();
        assert_eq!(lines, [1, 3]);
    }

    #[rstest]
    #[case::open_quote("a,\"b", ErrorKind::UnterminatedQuote, 1, 3)]
    #[case::open_quote_on_line_2("a,b\n\"x,y\n", ErrorKind::UnterminatedQuote, 2, 1)]
    #[case::junk_after_quotes("\"a\"b,c", ErrorKind::CharAfterQuote('b'), 1, 4)]
    #[case::junk_on_line_2("a,b\n\"x\"y,2\n", ErrorKind::CharAfterQuote('y'), 2, 4)]
    fn errors_with_position(
        #[case] input: &str,
        #[case] kind: ErrorKind,
        #[case] line: usize,
        #[case] column: usize,
    ) {
        let opts = Options {
            flexible: true,
            ..Options::default()
        };
        assert_eq!(read_all(input, opts), Err(Error { kind, line, column }));
    }
}

#[cfg(test)]
mod step3_tests {
    use super::*;
    use rstest::rstest;
    use std::assert_matches;

    #[rstest]
    #[case::too_many("a,b\nc,d,e\n", ErrorKind::FieldCount { expected: 2, found: 3 }, 2, 1)]
    #[case::too_few("a,b\nc,d\ne\n", ErrorKind::FieldCount { expected: 2, found: 1 }, 3, 1)]
    fn strict_mode(
        #[case] input: &str,
        #[case] kind: ErrorKind,
        #[case] line: usize,
        #[case] column: usize,
    ) {
        assert_eq!(read(input), Err(Error { kind, line, column }));
    }

    #[test]
    fn flexible_accepts_any_length() {
        let opts = Options {
            flexible: true,
            ..Options::default()
        };
        assert_eq!(
            read_all("a,b\nc\n", opts).unwrap(),
            vec![vec!["a", "b"], vec!["c"]]
        );
    }

    #[test]
    fn stops_after_first_error() {
        let mut it = Reader::new("a,b\nc,d,e\nf,g\n", Options::default());
        assert_matches!(it.next(), Some(Ok(_)));
        assert_matches!(
            it.next(),
            Some(Err(Error {
                kind: ErrorKind::FieldCount { .. },
                ..
            }))
        );
        assert_matches!(it.next(), None);
    }

    #[test]
    fn stops_after_quote_error() {
        let mut it = Reader::new("\"a\"x\nb\n", Options::default());
        assert_matches!(it.next(), Some(Err(_)));
        assert_matches!(it.next(), None);
    }
}

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
/// ```
/// use csvlite::{Options, Reader};
///
/// let mut reader = Reader::new("a,b\n\"c,d\",e\n", Options::default());
/// let primeiro = reader.next().unwrap()?;
/// assert_eq!(primeiro.get(0), Some("a"));
/// let segundo = reader.next().unwrap()?;
/// assert_eq!(segundo.get(0), Some("c,d"));
/// assert!(reader.next().is_none());
/// # Ok::<(), csvlite::Error>(())
/// ```
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

/// Como um campo terminou.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum End {
    /// Veio o separador: há outro campo no mesmo registro.
    Field,
    /// Veio uma quebra de linha, ou a entrada acabou: o registro terminou.
    Record,
}

impl<'a> Reader<'a> {
    /// Cria um leitor sobre `input`.
    ///
    /// O leitor empresta o texto (por isso o lifetime `'a`) e não copia nada
    /// até montar cada campo.
    pub fn new(input: &'a str, opts: Options) -> Self {
        Reader {
            chars: input.chars().peekable(),
            opts,
            line: 1,
            column: 1,
            expected: None,
            done: false,
        }
    }

    /// Consome um caractere, mantendo `line` e `column` atualizados.
    fn bump(&mut self) -> Option<char> {
        let c = self.chars.next()?;
        if c == '\n' {
            self.line += 1;
            self.column = 1;
        } else {
            self.column += 1;
        }
        Some(c)
    }

    /// Se `c` (já consumido) termina um campo, diz como.
    ///
    /// Trata `\r\n` como uma quebra só: consome o `\n` que vem depois do `\r`.
    fn end_of_field(&mut self, c: char) -> Option<End> {
        match c {
            '\n' => Some(End::Record),
            c if c == self.opts.delimiter => Some(End::Field),
            '\r' if self.chars.peek() == Some(&'\n') => {
                self.bump();
                Some(End::Record)
            }
            _ => None,
        }
    }

    /// Campo sem aspas: vai até o separador, o fim da linha ou o fim da entrada.
    ///
    /// Uma `"` no meio do campo é texto comum (a RFC proíbe, o Python aceita).
    fn read_unquoted(&mut self) -> (String, End) {
        let mut field = String::new();
        loop {
            let Some(c) = self.bump() else {
                return (field, End::Record);
            };
            match self.end_of_field(c) {
                Some(end) => return (field, end),
                None => field.push(c),
            }
        }
    }

    /// Campo entre aspas. Dentro delas, separador e quebra de linha são
    /// texto, e `""` vira `"`.
    fn read_quoted(&mut self) -> Result<(String, End), Error> {
        // Guarda onde as aspas abriram: é o lugar útil para a mensagem.
        let (line, column) = (self.line, self.column);
        self.bump(); // a aspa de abertura
        let mut field = String::new();
        loop {
            match self.bump() {
                None => {
                    return Err(Error {
                        kind: ErrorKind::UnterminatedQuote,
                        line,
                        column,
                    });
                }
                Some('"') if self.chars.peek() == Some(&'"') => {
                    self.bump();
                    field.push('"');
                }
                Some('"') => break,
                Some(c) => field.push(c),
            }
        }
        // Depois de fechar as aspas, só pode vir separador ou fim de linha.
        let (line, column) = (self.line, self.column);
        let Some(c) = self.bump() else {
            return Ok((field, End::Record));
        };
        match self.end_of_field(c) {
            Some(end) => Ok((field, end)),
            None => Err(Error {
                kind: ErrorKind::CharAfterQuote(c),
                line,
                column,
            }),
        }
    }

    /// Lê um registro inteiro. Só é chamado quando ainda há entrada.
    fn read_record(&mut self) -> Result<Record, Error> {
        let line = self.line;
        let mut fields = Vec::new();
        loop {
            let (field, end) = if self.chars.peek() == Some(&'"') {
                self.read_quoted()?
            } else {
                self.read_unquoted()
            };
            fields.push(field);
            if end == End::Record {
                return Ok(Record { fields, line });
            }
        }
    }

    /// No modo estrito, confere o número de campos com o do primeiro registro.
    fn check_field_count(&mut self, record: &Record) -> Result<(), Error> {
        if self.opts.flexible {
            return Ok(());
        }
        let expected = *self.expected.get_or_insert(record.len());
        if record.len() == expected {
            Ok(())
        } else {
            Err(Error {
                kind: ErrorKind::FieldCount {
                    expected,
                    found: record.len(),
                },
                line: record.line,
                column: 1,
            })
        }
    }
}

impl Iterator for Reader<'_> {
    type Item = Result<Record, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.done {
            return None;
        }
        // Entrada acabou (ou estava vazia): nenhum registro a mais.
        self.chars.peek()?;
        let result = self
            .read_record()
            .and_then(|record| self.check_field_count(&record).map(|()| record));
        if result.is_err() {
            self.done = true;
        }
        Some(result)
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

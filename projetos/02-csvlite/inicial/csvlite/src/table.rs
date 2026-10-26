use crate::{Error, FromRecord, Options, Reader, Record};

/// Um CSV inteiro na memória: o cabeçalho (primeiro registro) e as linhas.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Table {
    headers: Record,
    rows: Vec<Record>,
}

impl Table {
    /// Lê o texto inteiro. O primeiro registro vira o cabeçalho.
    ///
    /// Uma entrada vazia dá uma tabela sem cabeçalho e sem linhas.
    ///
    pub fn parse(input: &str, opts: Options) -> Result<Self, Error> {
        // Etapa 4. Dica: `transpose` e `collect::<Result<Vec<_>, _>>()`.
        todo!()
    }

    /// O cabeçalho.
    pub fn headers(&self) -> &Record {
        todo!()
    }

    /// Os registros depois do cabeçalho.
    pub fn rows(&self) -> &[Record] {
        todo!()
    }

    /// Os valores da coluna chamada `name`, ou `None` se ela não existe.
    ///
    /// Uma linha curta demais (no modo flexível) contribui com `""`.
    pub fn column(&self, name: &str) -> Option<impl Iterator<Item = &str>> {
        // Veja o comentário de `Record::iter` sobre este `as`.
        todo!() as Option<std::iter::Empty<&str>>
    }

    /// Converte cada linha num `T`.
    pub fn deserialize<T: FromRecord>(&self) -> Result<Vec<T>, Error> {
        // Etapa 6.
        todo!()
    }
}

#[cfg(test)]
mod step4_tests {
    use super::*;

    const ALUNOS: &str = "nome,nota\nAna,9\nBruno,7\n";

    #[test]
    fn headers_and_rows() {
        let t = Table::parse(ALUNOS, Options::default()).unwrap();
        assert_eq!(t.headers().iter().collect::<Vec<_>>(), ["nome", "nota"]);
        assert_eq!(t.rows().len(), 2);
        assert_eq!(t.rows()[1].get(0), Some("Bruno"));
        assert_eq!(t.rows()[1].line(), 3);
    }

    #[test]
    fn column_by_name() {
        let t = Table::parse(ALUNOS, Options::default()).unwrap();
        let notas: Vec<&str> = t.column("nota").unwrap().collect();
        assert_eq!(notas, ["9", "7"]);
    }

    #[test]
    fn missing_column() {
        let t = Table::parse(ALUNOS, Options::default()).unwrap();
        assert!(t.column("idade").is_none());
    }

    #[test]
    fn short_rows_give_empty_values() {
        let opts = Options {
            flexible: true,
            ..Options::default()
        };
        let t = Table::parse("a,b\n1\n2,3\n", opts).unwrap();
        let b: Vec<&str> = t.column("b").unwrap().collect();
        assert_eq!(b, ["", "3"]);
    }

    #[test]
    fn empty_table() {
        let t = Table::parse("", Options::default()).unwrap();
        assert!(t.headers().is_empty());
        assert_eq!(t.rows(), []);
    }

    #[test]
    fn header_only() {
        let t = Table::parse("a,b\n", Options::default()).unwrap();
        assert_eq!(t.headers().len(), 2);
        assert_eq!(t.rows(), []);
    }

    #[test]
    fn parse_error_is_returned() {
        let err = Table::parse("a,b\n1,2,3\n", Options::default()).unwrap_err();
        assert_eq!(err.line, 2);
    }
}

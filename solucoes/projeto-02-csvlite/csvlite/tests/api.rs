//! Testes de integração: usam só a API pública, como uma crate de fora.

use csvlite::{Error, ErrorKind, FromRecord, Options, Reader, Record, Table, write_record};

/// Escreve os registros com `write_record` e devolve o texto.
fn write_all(records: &[Vec<String>]) -> String {
    let mut out = Vec::new();
    for record in records {
        write_record(&mut out, record, ',').unwrap();
    }
    String::from_utf8(out).unwrap()
}

mod step5_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        // 64 casos (o padrão é 256) deixam o teste rápido e ainda acham os bugs.
        #![proptest_config(ProptestConfig::with_cases(64))]

        /// Escrever e ler de volta devolve exatamente o que foi escrito.
        #[test]
        fn round_trip(records in prop::collection::vec(prop::collection::vec("\\PC*", 1..4), 1..20)) {
            let text = write_all(&records);
            // `flexible`: os registros gerados podem ter tamanhos diferentes.
            let opts = Options { flexible: true, ..Options::default() };
            let read: Vec<Vec<String>> = Reader::new(&text, opts)
                .map(|r| r.unwrap().iter().map(String::from).collect())
                .collect();
            prop_assert_eq!(read, records);
        }
    }

    // Casos que o proptest encontrou ao testar versões erradas do escritor,
    // guardados como testes com nome (veja a etapa 5 do enunciado).

    #[test]
    fn lone_quote_round_trips() {
        let records = vec![vec![String::from("\"")]];
        let text = write_all(&records);
        let read: Vec<String> = Reader::new(&text, Options::default())
            .next()
            .unwrap()
            .unwrap()
            .iter()
            .map(String::from)
            .collect();
        assert_eq!(read, records[0]);
    }

    #[test]
    fn records_of_different_lengths_need_flexible() {
        let records = vec![vec![String::new()], vec![String::new(), String::new()]];
        let text = write_all(&records);
        let strict: Result<Vec<Record>, Error> = Reader::new(&text, Options::default()).collect();
        assert_eq!(
            strict.unwrap_err().kind,
            ErrorKind::FieldCount {
                expected: 1,
                found: 2
            }
        );
    }
}

mod step6_tests {
    use super::*;

    #[derive(Debug, PartialEq)]
    struct Person {
        name: String,
        age: u32,
    }

    impl FromRecord for Person {
        fn from_record(record: &Record) -> Result<Self, Error> {
            let field = |i: usize| {
                record
                    .get(i)
                    .ok_or_else(|| Error::invalid_field(record, i, "campo ausente"))
            };
            let name = field(0)?.to_string();
            let age = field(1)?
                .parse()
                .map_err(|e| Error::invalid_field(record, 1, e))?;
            Ok(Person { name, age })
        }
    }

    #[test]
    fn deserialize_people() {
        let t = Table::parse("nome,idade\nAna,30\nBruno,25\n", Options::default()).unwrap();
        let people: Vec<Person> = t.deserialize().unwrap();
        assert_eq!(
            people,
            [
                Person {
                    name: "Ana".into(),
                    age: 30
                },
                Person {
                    name: "Bruno".into(),
                    age: 25
                },
            ]
        );
    }

    #[test]
    fn deserialize_error_points_to_the_line() {
        let t = Table::parse("nome,idade\nAna,30\nBruno,vinte\n", Options::default()).unwrap();
        let err = t.deserialize::<Person>().unwrap_err();
        assert_eq!(
            err.kind,
            ErrorKind::InvalidField {
                index: 1,
                message: "invalid digit found in string".into()
            }
        );
        assert_eq!(
            err.to_string(),
            "linha 3, coluna 1: campo 2: invalid digit found in string"
        );
    }

    #[test]
    fn deserialize_missing_field() {
        let opts = Options {
            flexible: true,
            ..Options::default()
        };
        let t = Table::parse("nome,idade\nAna\n", opts).unwrap();
        let err = t.deserialize::<Person>().unwrap_err();
        assert_eq!(err.to_string(), "linha 2, coluna 1: campo 2: campo ausente");
    }
}

use std::iter::Map;
use std::slice;

/// Um registro (uma "linha lógica") do CSV: uma sequência de campos.
///
/// Um registro pode ocupar várias linhas do arquivo, quando um campo entre
/// aspas tem quebra de linha.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Record {
    // `pub(crate)`: o `Reader` monta o registro direto e os testes internos
    // leem os campos, mas quem usa a crate só enxerga os métodos.
    pub(crate) fields: Vec<String>,
    pub(crate) line: usize,
}

impl Record {
    /// Cria um registro a partir de qualquer coleção de textos.
    ///
    /// Aceita `&str`, `String` ou qualquer outra coisa que vire `String`.
    /// Um registro criado assim não veio de uma entrada, e sua
    /// [`line`](Record::line) é 0.
    ///
    /// ```
    /// use csvlite::Record;
    ///
    /// let rec = Record::from_fields(["Ana", "Rio"]);
    /// assert_eq!(rec.get(1), Some("Rio"));
    ///
    /// let rec = Record::from_fields(vec![String::from("x")]);
    /// assert_eq!(rec.len(), 1);
    /// ```
    pub fn from_fields<I, S>(fields: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        Record {
            fields: fields.into_iter().map(Into::into).collect(),
            line: 0,
        }
    }

    /// O campo de índice `i` (começando em 0), se existir.
    pub fn get(&self, i: usize) -> Option<&str> {
        self.fields.get(i).map(String::as_str)
    }

    /// Número de campos.
    pub fn len(&self) -> usize {
        self.fields.len()
    }

    /// `true` se o registro não tem nenhum campo.
    pub fn is_empty(&self) -> bool {
        self.fields.is_empty()
    }

    /// A linha da entrada em que o registro começa (0 se ele não veio de uma entrada).
    pub fn line(&self) -> usize {
        self.line
    }

    /// Percorre os campos como `&str`.
    pub fn iter(&self) -> impl Iterator<Item = &str> {
        self.fields.iter().map(String::as_str)
    }
}

/// Permite `for campo in &registro { ... }`.
impl<'a> IntoIterator for &'a Record {
    type Item = &'a str;
    // O tipo do iterador precisa ter nome. Uma closure não tem nome, então
    // usamos um ponteiro de função (`fn`), que tem.
    type IntoIter = Map<slice::Iter<'a, String>, fn(&String) -> &str>;

    fn into_iter(self) -> Self::IntoIter {
        self.fields.iter().map(String::as_str)
    }
}

#[cfg(test)]
mod step4_tests {
    use super::*;

    #[test]
    fn get_and_len() {
        let rec = Record::from_fields(["a", "b", "c"]);
        assert_eq!(rec.len(), 3);
        assert!(!rec.is_empty());
        assert_eq!(rec.get(0), Some("a"));
        assert_eq!(rec.get(2), Some("c"));
        assert_eq!(rec.get(3), None);
    }

    #[test]
    fn from_fields_accepts_strings() {
        let rec = Record::from_fields(vec![String::from("x"), String::from("y")]);
        assert_eq!(rec.get(1), Some("y"));
        assert_eq!(rec.line(), 0);
    }

    #[test]
    fn empty_record() {
        let rec = Record::default();
        assert!(rec.is_empty());
        assert_eq!(rec.get(0), None);
    }

    #[test]
    fn iter_yields_str() {
        let rec = Record::from_fields(["a", "", "c"]);
        let fields: Vec<&str> = rec.iter().collect();
        assert_eq!(fields, ["a", "", "c"]);
    }

    #[test]
    fn for_loop_over_reference() {
        let rec = Record::from_fields(["1", "2", "3"]);
        let mut total = 0;
        for field in &rec {
            total += field.parse::<i32>().unwrap();
        }
        assert_eq!(total, 6);
        // `rec` continua disponível: o laço só emprestou.
        assert_eq!(rec.len(), 3);
    }
}

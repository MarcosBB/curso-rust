use crate::{Error, Record};

/// Tipos que sabem se construir a partir de um [`Record`].
///
/// É usada por [`Table::deserialize`](crate::Table::deserialize). Uma
/// implementação típica lê cada campo pelo índice, converte e devolve
/// [`Error::invalid_field`] quando algo não bate. Veja o teste de
/// `Person` em `tests/api.rs`.
///
/// É, em miniatura, o que o `#[derive(Deserialize)]` da crate `serde` gera
/// automaticamente.
pub trait FromRecord: Sized {
    /// Constrói o valor a partir do registro.
    fn from_record(record: &Record) -> Result<Self, Error>;
}

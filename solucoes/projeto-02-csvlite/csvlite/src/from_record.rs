use crate::{Error, Record};

/// Tipos que sabem se construir a partir de um [`Record`].
///
/// É usada por [`Table::deserialize`](crate::Table::deserialize). Uma
/// implementação típica lê cada campo pelo índice, converte e devolve
/// [`Error::invalid_field`] quando algo não bate:
///
/// ```
/// use csvlite::{Error, FromRecord, Record};
///
/// struct Ponto {
///     x: i32,
///     y: i32,
/// }
///
/// impl FromRecord for Ponto {
///     fn from_record(record: &Record) -> Result<Self, Error> {
///         let campo = |i: usize| -> Result<i32, Error> {
///             let texto = record.get(i).unwrap_or_default();
///             texto.parse().map_err(|e| Error::invalid_field(record, i, e))
///         };
///         Ok(Ponto { x: campo(0)?, y: campo(1)? })
///     }
/// }
///
/// let p = Ponto::from_record(&Record::from_fields(["3", "-4"]))?;
/// assert_eq!((p.x, p.y), (3, -4));
/// # Ok::<(), Error>(())
/// ```
///
/// É, em miniatura, o que o `#[derive(Deserialize)]` da crate `serde` gera
/// automaticamente.
pub trait FromRecord: Sized {
    /// Constrói o valor a partir do registro.
    ///
    /// # Errors
    ///
    /// Devolve um erro (normalmente [`ErrorKind::InvalidField`](crate::ErrorKind::InvalidField))
    /// quando algum campo falta ou não pode ser convertido.
    fn from_record(record: &Record) -> Result<Self, Error>;
}

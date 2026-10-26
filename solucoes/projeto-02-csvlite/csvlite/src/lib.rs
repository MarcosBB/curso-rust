//! `csvlite`: leitor e escritor de CSV com aspas, só com a biblioteca padrão.
//!
//! Aceita um subconjunto da [RFC 4180](https://www.rfc-editor.org/rfc/rfc4180):
//! campos entre aspas podem conter o separador, quebras de linha e aspas
//! duplicadas (`""`). O separador é configurável, então o `;` do Excel em
//! português funciona:
//!
//! ```
//! use csvlite::{Options, Table};
//!
//! let texto = "nome;nota\nAna;\"9,5\"\nBruno;7\n";
//! let opts = Options { delimiter: ';', ..Options::default() };
//! let tabela = Table::parse(texto, opts)?;
//!
//! let notas: Vec<&str> = tabela.column("nota").unwrap().collect();
//! assert_eq!(notas, ["9,5", "7"]);
//! # Ok::<(), csvlite::Error>(())
//! ```
//!
//! Este é um projeto de estudo. Em código de verdade, use a crate
//! [`csv`](https://docs.rs/csv), que é rápida, completa e testada há anos.
//!
//! # Organização
//!
//! Os módulos internos são privados: a API pública é só o que esta raiz
//! reexporta. Por isso, o caminho interno não existe para quem usa a crate:
//!
//! ```compile_fail
//! use csvlite::reader::Reader; // erro: o módulo `reader` é privado
//! ```

mod error;
mod from_record;
mod reader;
mod record;
mod table;
mod writer;

pub use error::{Error, ErrorKind};
pub use from_record::FromRecord;
pub use reader::{Options, Reader};
pub use record::Record;
pub use table::Table;
pub use writer::write_record;

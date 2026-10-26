//! `csvlite`: leitor e escritor de CSV com aspas, só com a biblioteca padrão.
//!
//! Aceita um subconjunto da [RFC 4180](https://www.rfc-editor.org/rfc/rfc4180):
//! campos entre aspas podem conter o separador, quebras de linha e aspas
//! duplicadas (`""`). O separador é configurável.
//!
//! Este é um projeto de estudo. Em código de verdade, use a crate
//! [`csv`](https://docs.rs/csv), que é rápida, completa e testada há anos.
//!
//! <!-- Etapa 8: acrescente aqui um exemplo de uso (doctest) com `;`. -->
//!
//! # Organização
//!
//! Os módulos internos são privados: a API pública é só o que esta raiz
//! reexporta. Por isso, o caminho interno não existe para quem usa a crate:
//!
//! ```compile_fail
//! use csvlite::reader::Reader; // erro: o módulo `reader` é privado
//! ```

// Só no projeto inicial: enquanto os corpos forem `todo!()`, parâmetros,
// campos e imports aparecem como "não usados". Apague esta linha quando
// terminar a etapa 6.
#![allow(unused)]
// Idem: com o corpo em `todo!()`, o clippy acha que os parâmetros por valor
// não são consumidos. Apague junto com a linha acima.
#![allow(clippy::needless_pass_by_value)]
// Só no projeto inicial: o clippy pedântico cobra uma seção `# Errors` na
// documentação de toda função pública que devolve `Result`. Apague esta
// linha na etapa 8 e escreva essas seções.
#![allow(clippy::missing_errors_doc)]

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

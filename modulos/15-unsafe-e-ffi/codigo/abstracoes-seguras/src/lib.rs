//! Abstrações seguras sobre `unsafe` (lição 03 do Módulo 15).
//!
//! Cada módulo esconde um bloco `unsafe` atrás de uma interface segura.
//! Rode os testes também com `cargo +nightly miri test`.

pub mod buffer;
pub mod celula;
pub mod dividir;
pub mod meu_box;

//! Exemplo do Módulo 08: a árvore de módulos de uma biblioteca pequena.
//!
//! Os módulos `rede` e `cliente` são públicos, e o tipo mais usado,
//! [`Conexao`], também é reexportado aqui na raiz (lição 03).

pub mod cliente;
pub mod rede;

pub use rede::tcp::Conexao;

/// Registra uma mensagem de diagnóstico. Visível em toda a crate, mas fora da API pública.
pub(crate) fn registrar(msg: &str) -> String {
    format!("[chat] {msg}")
}

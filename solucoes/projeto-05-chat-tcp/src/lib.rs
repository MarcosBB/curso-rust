//! `chat`: um servidor de chat em TCP, com `tokio`.
//!
//! Protocolo de linhas (UTF-8, terminadas em `\n`): o cliente manda texto ou
//! comandos (`/nick NOME`, `/quem`, `/sair`), e o servidor repassa a todos.
//! Dá para usar com `nc 127.0.0.1 7878`.

pub mod conexao;
pub mod protocolo;
pub mod sala;
pub mod servidor;

pub use protocolo::{Comando, ErroApelido};
pub use sala::{Evento, Sala};

/// Endereço padrão do servidor e do cliente.
pub const ENDERECO_PADRAO: &str = "127.0.0.1:7878";

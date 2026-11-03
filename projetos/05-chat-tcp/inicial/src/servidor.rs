//! O laço que aceita conexões e cria uma task para cada uma.

use std::sync::Arc;

use tokio::net::TcpListener;
use tokio_util::sync::CancellationToken;
use tokio_util::task::TaskTracker;

use crate::conexao;
use crate::sala::Sala;

/// Quantos eventos o canal da sala guarda para quem está atrasado.
pub const CAPACIDADE: usize = 64;

/// Aceita conexões em `listener` até `desligar` ser cancelado. Depois avisa
/// todas as conexões e só volta quando a última task terminar.
pub async fn rodar(listener: TcpListener, desligar: CancellationToken) {
    // Etapa 3: crie a `Sala` num `Arc` e, num `loop`, `listener.accept().await`
    // e `tokio::spawn(..)` de uma task que chama `conexao::atender`.
    // Etapa 5: `select!` entre o `accept` e `desligar.cancelled()`, um
    // `TaskTracker` no lugar do `tokio::spawn`, e `close` + `wait` no fim.
    todo!()
}

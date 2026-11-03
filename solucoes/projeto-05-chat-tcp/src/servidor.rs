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
    let sala = Arc::new(Sala::new(CAPACIDADE));
    let tarefas = TaskTracker::new();

    loop {
        tokio::select! {
            () = desligar.cancelled() => break,
            aceito = listener.accept() => {
                let (stream, endereco) = match aceito {
                    Ok(par) => par,
                    Err(erro) => {
                        // Um erro aqui é de uma conexão só (ou falta de
                        // descritores): registre e continue aceitando.
                        eprintln!("chat: accept: {erro}");
                        continue;
                    }
                };
                eprintln!("chat: {endereco} conectou");
                let sala = Arc::clone(&sala);
                let desligar = desligar.clone();
                tarefas.spawn(async move {
                    match conexao::atender(stream, sala, desligar).await {
                        Ok(()) => eprintln!("chat: {endereco} desconectou"),
                        Err(erro) => eprintln!("chat: {endereco}: {erro}"),
                    }
                });
            }
        }
    }

    // Não aceita mais ninguém: o listener é fechado aqui.
    drop(listener);
    tarefas.close();
    tarefas.wait().await;
}

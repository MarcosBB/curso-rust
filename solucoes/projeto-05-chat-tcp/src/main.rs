//! O servidor: `chat [ENDERECO]`. Ctrl+C encerra com calma.

use std::env;
use std::process::ExitCode;

use tokio::net::TcpListener;
use tokio::signal;
use tokio_util::sync::CancellationToken;

use chat::{ENDERECO_PADRAO, servidor};

#[tokio::main]
async fn main() -> ExitCode {
    let endereco = env::args()
        .nth(1)
        .unwrap_or_else(|| ENDERECO_PADRAO.to_string());

    let listener = match TcpListener::bind(&endereco).await {
        Ok(l) => l,
        Err(erro) => {
            eprintln!("chat: {endereco}: {erro}");
            return ExitCode::FAILURE;
        }
    };
    // `local_addr` mostra a porta de verdade, mesmo quando você pede a porta 0.
    match listener.local_addr() {
        Ok(local) => eprintln!("chat: escutando em {local} (Ctrl+C para sair)"),
        Err(_) => eprintln!("chat: escutando em {endereco} (Ctrl+C para sair)"),
    }

    let desligar = CancellationToken::new();
    let token = desligar.clone();
    tokio::spawn(async move {
        // Se não der para instalar o tratador de Ctrl+C, o servidor segue
        // rodando; ele só não vai ter o desligamento gracioso.
        if signal::ctrl_c().await.is_ok() {
            eprintln!("chat: encerrando...");
            token.cancel();
        }
    });

    servidor::rodar(listener, desligar).await;
    eprintln!("chat: até mais");
    ExitCode::SUCCESS
}

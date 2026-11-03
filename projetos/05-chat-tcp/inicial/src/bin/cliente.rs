//! Um cliente mínimo: `cliente [ENDERECO]`. O que você digita vai para o
//! servidor; o que o servidor manda aparece na tela.

// Só no projeto inicial (veja `lib.rs`). Apague na etapa 6.
#![allow(unused)]

use std::env;
use std::process::ExitCode;

use tokio::io::{self, AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;

use chat::ENDERECO_PADRAO;

#[tokio::main]
async fn main() -> ExitCode {
    let endereco = env::args()
        .nth(1)
        .unwrap_or_else(|| ENDERECO_PADRAO.to_string());
    match conversar(&endereco).await {
        Ok(()) => ExitCode::SUCCESS,
        Err(erro) => {
            eprintln!("cliente: {endereco}: {erro}");
            ExitCode::FAILURE
        }
    }
}

async fn conversar(endereco: &str) -> io::Result<()> {
    // Etapa 6: `TcpStream::connect`, `into_split`, `BufReader::lines` nas
    // duas entradas (socket e `io::stdin()`) e um `select!` entre elas.
    todo!()
}

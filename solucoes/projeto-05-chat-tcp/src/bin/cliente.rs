//! Um cliente mínimo: `cliente [ENDERECO]`. O que você digita vai para o
//! servidor; o que o servidor manda aparece na tela.

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
    let stream = TcpStream::connect(endereco).await?;
    // Duas metades independentes: uma só lê, a outra só escreve.
    let (leitura, mut escrita) = stream.into_split();
    let mut do_servidor = BufReader::new(leitura).lines();
    let mut do_teclado = BufReader::new(io::stdin()).lines();
    let mut teclado_aberto = true;

    loop {
        // `next_line` é cancel safe (ao contrário de `read_line`): pode ir
        // num `select!` sem perder pedaço de linha.
        tokio::select! {
            linha = do_servidor.next_line() => match linha? {
                Some(linha) => println!("{linha}"),
                None => {
                    println!("(conexão encerrada pelo servidor)");
                    return Ok(());
                }
            },
            // A condição depois do `if` desliga este ramo depois do Ctrl+D;
            // sem ela, `next_line` devolveria `None` de novo a cada volta.
            linha = do_teclado.next_line(), if teclado_aberto => match linha? {
                Some(linha) => {
                    escrita.write_all(linha.as_bytes()).await?;
                    escrita.write_all(b"\n").await?;
                }
                // Ctrl+D: avisa o servidor e espera ele fechar.
                None => {
                    teclado_aberto = false;
                    escrita.write_all(b"/sair\n").await?;
                }
            },
        }
    }
}

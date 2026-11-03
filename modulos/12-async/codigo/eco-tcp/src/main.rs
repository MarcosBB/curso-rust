//! Lição 04: um servidor de eco TCP com tokio. Cada conexão vira uma task.
//!
//! `cargo run` sobe o servidor em 127.0.0.1:7878. Em outro terminal:
//! `nc 127.0.0.1 7878` e digite linhas; o servidor devolve cada uma.

use std::io;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

/// Devolve ao cliente tudo o que ele mandar, até ele fechar a conexão.
async fn atender(mut socket: TcpStream) -> io::Result<()> {
    let mut buf = [0u8; 1024];
    loop {
        let n = socket.read(&mut buf).await?;
        if n == 0 {
            return Ok(()); // o cliente fechou
        }
        socket.write_all(&buf[..n]).await?;
    }
}

/// Aceita conexões para sempre, uma task por cliente.
async fn servir(listener: TcpListener) -> io::Result<()> {
    loop {
        let (socket, cliente) = listener.accept().await?;
        println!("conexão de {cliente}");
        tokio::spawn(async move {
            if let Err(e) = atender(socket).await {
                eprintln!("erro com {cliente}: {e}");
            }
            println!("{cliente} saiu");
        });
    }
}

#[tokio::main]
async fn main() -> io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:7878").await?;
    println!("ouvindo em {}", listener.local_addr()?);
    servir(listener).await
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Sobe o servidor numa porta livre (porta 0) e devolve o endereço.
    async fn subir() -> std::net::SocketAddr {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let endereco = listener.local_addr().unwrap();
        tokio::spawn(servir(listener));
        endereco
    }

    async fn eco(endereco: std::net::SocketAddr, msg: &str) -> String {
        let mut c = TcpStream::connect(endereco).await.unwrap();
        c.write_all(msg.as_bytes()).await.unwrap();
        c.shutdown().await.unwrap(); // avisa que não vamos mandar mais nada
        let mut resposta = String::new();
        c.read_to_string(&mut resposta).await.unwrap();
        resposta
    }

    #[tokio::test]
    async fn devolve_o_que_recebe() {
        let endereco = subir().await;
        assert_eq!(eco(endereco, "olá, eco\n").await, "olá, eco\n");
    }

    #[tokio::test]
    async fn atende_varios_clientes_ao_mesmo_tempo() {
        let endereco = subir().await;
        let mut set = tokio::task::JoinSet::new();
        for i in 0..50 {
            set.spawn(async move { (i, eco(endereco, &format!("cliente {i}")).await) });
        }
        while let Some(r) = set.join_next().await {
            let (i, resposta) = r.unwrap();
            assert_eq!(resposta, format!("cliente {i}"));
        }
    }
}

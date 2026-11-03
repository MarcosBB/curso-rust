//! Testes de integração: sobem o servidor numa porta livre e conversam com
//! ele por TCP, linha a linha, como o `nc` faria.
//!
//! Toda leitura tem um tempo limite: se o servidor não responder, o teste
//! falha em 2 segundos em vez de ficar travado para sempre.

use std::net::SocketAddr;
use std::time::Duration;

use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader, Lines};
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::net::{TcpListener, TcpStream};
use tokio::task::JoinHandle;
use tokio::time::timeout;
use tokio_util::sync::CancellationToken;

use chat::servidor;

const ESPERA: Duration = Duration::from_secs(2);

struct Servidor {
    endereco: SocketAddr,
    desligar: CancellationToken,
    tarefa: JoinHandle<()>,
}

/// Sobe um servidor em `127.0.0.1:0`: o sistema escolhe uma porta livre, e
/// cada teste tem o seu servidor (os testes rodam em paralelo).
async fn iniciar() -> Servidor {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let endereco = listener.local_addr().unwrap();
    let desligar = CancellationToken::new();
    let tarefa = tokio::spawn(servidor::rodar(listener, desligar.clone()));
    Servidor {
        endereco,
        desligar,
        tarefa,
    }
}

struct Cliente {
    linhas: Lines<BufReader<OwnedReadHalf>>,
    escrita: OwnedWriteHalf,
}

impl Cliente {
    /// Conecta e confere a mensagem de boas-vindas.
    async fn conectar(endereco: SocketAddr) -> Cliente {
        let stream = timeout(ESPERA, TcpStream::connect(endereco))
            .await
            .expect("tempo esgotado ao conectar")
            .unwrap();
        let (leitura, escrita) = stream.into_split();
        let mut cliente = Cliente {
            linhas: BufReader::new(leitura).lines(),
            escrita,
        };
        let boas_vindas = cliente.ler().await;
        assert!(boas_vindas.starts_with("*** bem-vindo, "), "{boas_vindas}");
        cliente
    }

    /// Conecta e troca o apelido.
    async fn entrar(endereco: SocketAddr, apelido: &str) -> Cliente {
        let mut cliente = Cliente::conectar(endereco).await;
        cliente.enviar(&format!("/nick {apelido}")).await;
        assert_eq!(cliente.ler().await, format!("*** agora você é {apelido}"));
        cliente
    }

    async fn enviar(&mut self, linha: &str) {
        self.escrita.write_all(linha.as_bytes()).await.unwrap();
        self.escrita.write_all(b"\n").await.unwrap();
    }

    /// A próxima linha que o servidor mandou.
    async fn ler(&mut self) -> String {
        timeout(ESPERA, self.linhas.next_line())
            .await
            .expect("tempo esgotado esperando uma linha")
            .unwrap()
            .expect("o servidor fechou a conexão")
    }

    /// Confere que o servidor fechou a conexão (sem mandar mais nada).
    async fn deve_fechar(&mut self) {
        let linha = timeout(ESPERA, self.linhas.next_line())
            .await
            .expect("tempo esgotado esperando o fim da conexão")
            .unwrap();
        assert_eq!(linha, None, "esperava o fim da conexão");
    }
}

mod step3 {
    use super::*;

    #[tokio::test]
    async fn boas_vindas_com_apelido_provisorio() {
        let s = iniciar().await;
        let mut a = TcpStream::connect(s.endereco).await.unwrap();
        let mut linha = String::new();
        let (leitura, _) = a.split();
        timeout(ESPERA, BufReader::new(leitura).read_line(&mut linha))
            .await
            .unwrap()
            .unwrap();
        assert_eq!(
            linha,
            "*** bem-vindo, convidado-1! comandos: /nick NOME, /quem, /sair\n"
        );
    }

    #[tokio::test]
    async fn nick_e_quem() {
        let s = iniciar().await;
        let mut a = Cliente::conectar(s.endereco).await;
        a.enviar("/quem").await;
        assert_eq!(a.ler().await, "*** online: convidado-1");
        a.enviar("/nick ana").await;
        assert_eq!(a.ler().await, "*** agora você é ana");
        a.enviar("/quem").await;
        assert_eq!(a.ler().await, "*** online: ana");
    }

    #[tokio::test]
    async fn erros_voltam_so_para_quem_errou() {
        let s = iniciar().await;
        let mut a = Cliente::conectar(s.endereco).await;
        a.enviar("/nick").await;
        assert_eq!(a.ler().await, "*** o apelido não pode ser vazio");
        a.enviar("/nick ana maria").await;
        assert_eq!(a.ler().await, "*** caractere não permitido no apelido: ' '");
        a.enviar("/dança").await;
        assert_eq!(a.ler().await, "*** comando desconhecido: /dança");
        // Linhas vazias não geram resposta: a próxima linha é a do /quem.
        a.enviar("").await;
        a.enviar("   ").await;
        a.enviar("/quem").await;
        assert_eq!(a.ler().await, "*** online: convidado-1");
    }

    #[tokio::test]
    async fn apelido_em_uso() {
        let s = iniciar().await;
        let _a = Cliente::entrar(s.endereco, "ana").await;
        let mut b = Cliente::conectar(s.endereco).await;
        b.enviar("/nick ana").await;
        assert_eq!(b.ler().await, "*** o apelido ana já está em uso");
        b.enviar("/quem").await;
        assert_eq!(b.ler().await, "*** online: ana, convidado-2");
    }

    #[tokio::test]
    async fn sair_fecha_a_conexao() {
        let s = iniciar().await;
        let mut a = Cliente::conectar(s.endereco).await;
        a.enviar("/sair").await;
        assert_eq!(a.ler().await, "*** até mais!");
        a.deve_fechar().await;
        // O apelido ficou livre.
        let mut b = Cliente::conectar(s.endereco).await;
        b.enviar("/quem").await;
        assert_eq!(b.ler().await, "*** online: convidado-2");
    }

    #[tokio::test]
    async fn aceita_crlf_do_telnet() {
        let s = iniciar().await;
        let mut a = Cliente::conectar(s.endereco).await;
        a.escrita.write_all(b"/nick ana\r\n").await.unwrap();
        assert_eq!(a.ler().await, "*** agora você é ana");
    }

    #[tokio::test]
    async fn linha_longa_demais_encerra_a_conexao() {
        let s = iniciar().await;
        let mut a = Cliente::conectar(s.endereco).await;
        a.enviar(&"x".repeat(5000)).await;
        assert_eq!(a.ler().await, "*** linha longa demais (máximo 1024 bytes)");
        a.deve_fechar().await;
    }

    #[tokio::test]
    async fn varios_clientes_ao_mesmo_tempo() {
        let s = iniciar().await;
        let mut clientes = Vec::new();
        for i in 0..20 {
            clientes.push(Cliente::entrar(s.endereco, &format!("c{i:02}")).await);
        }
        // Um cliente novo, que não ouviu os anúncios de quem entrou antes.
        let mut ultimo = Cliente::conectar(s.endereco).await;
        ultimo.enviar("/quem").await;
        let online = ultimo.ler().await;
        assert!(online.starts_with("*** online: c00, c01, c02,"), "{online}");
        assert!(online.ends_with("c19, convidado-21"), "{online}");
    }
}

mod step4 {
    use super::*;

    #[tokio::test]
    async fn mensagem_chega_aos_outros_e_nao_ao_autor() {
        let s = iniciar().await;
        let mut ana = Cliente::entrar(s.endereco, "ana").await;
        let mut bob = Cliente::entrar(s.endereco, "bob").await;
        assert_eq!(ana.ler().await, "*** convidado-2 entrou");
        assert_eq!(ana.ler().await, "*** convidado-2 agora é bob");

        ana.enviar("oi, bob").await;
        assert_eq!(bob.ler().await, "ana: oi, bob");
        bob.enviar("oi, ana").await;
        assert_eq!(ana.ler().await, "bob: oi, ana");

        // Se a ana tivesse recebido o próprio "oi", ele viria antes disto.
        ana.enviar("/quem").await;
        assert_eq!(ana.ler().await, "*** online: ana, bob");
    }

    #[tokio::test]
    async fn entrou_e_saiu() {
        let s = iniciar().await;
        let mut ana = Cliente::entrar(s.endereco, "ana").await;

        let mut b = Cliente::conectar(s.endereco).await;
        assert_eq!(ana.ler().await, "*** convidado-2 entrou");
        b.enviar("/sair").await;
        assert_eq!(ana.ler().await, "*** convidado-2 saiu");

        // Fechar a conexão sem /sair também conta.
        let c = Cliente::conectar(s.endereco).await;
        assert_eq!(ana.ler().await, "*** convidado-3 entrou");
        drop(c);
        assert_eq!(ana.ler().await, "*** convidado-3 saiu");
    }

    #[tokio::test]
    async fn todos_recebem_na_ordem() {
        let s = iniciar().await;
        let mut ana = Cliente::entrar(s.endereco, "ana").await;
        let mut bob = Cliente::entrar(s.endereco, "bob").await;
        let mut cia = Cliente::entrar(s.endereco, "cia").await;
        for linha in [
            "*** convidado-2 entrou",
            "*** convidado-2 agora é bob",
            "*** convidado-3 entrou",
            "*** convidado-3 agora é cia",
        ] {
            assert_eq!(ana.ler().await, linha);
        }
        for linha in ["*** convidado-3 entrou", "*** convidado-3 agora é cia"] {
            assert_eq!(bob.ler().await, linha);
        }

        for i in 1..=50 {
            ana.enviar(&format!("mensagem {i}")).await;
        }
        for i in 1..=50 {
            let esperado = format!("ana: mensagem {i}");
            assert_eq!(bob.ler().await, esperado);
            assert_eq!(cia.ler().await, esperado);
        }
    }

    #[tokio::test]
    async fn troca_de_apelido_e_anunciada() {
        let s = iniciar().await;
        let mut ana = Cliente::entrar(s.endereco, "ana").await;
        let mut bob = Cliente::entrar(s.endereco, "bob").await;
        assert_eq!(ana.ler().await, "*** convidado-2 entrou");
        assert_eq!(ana.ler().await, "*** convidado-2 agora é bob");

        bob.enviar("/nick roberto").await;
        assert_eq!(bob.ler().await, "*** agora você é roberto");
        assert_eq!(ana.ler().await, "*** bob agora é roberto");
        bob.enviar("tchau").await;
        assert_eq!(ana.ler().await, "roberto: tchau");
    }
}

mod step5 {
    use super::*;

    #[tokio::test]
    async fn desligar_avisa_todos_e_espera_as_tasks() {
        let s = iniciar().await;
        let mut ana = Cliente::entrar(s.endereco, "ana").await;
        let mut bob = Cliente::entrar(s.endereco, "bob").await;
        assert_eq!(ana.ler().await, "*** convidado-2 entrou");
        assert_eq!(ana.ler().await, "*** convidado-2 agora é bob");

        s.desligar.cancel();
        assert_eq!(ana.ler().await, "*** servidor encerrando");
        assert_eq!(bob.ler().await, "*** servidor encerrando");
        ana.deve_fechar().await;
        bob.deve_fechar().await;
        timeout(ESPERA, s.tarefa)
            .await
            .expect("o servidor não terminou")
            .unwrap();
    }

    #[tokio::test]
    async fn desligar_sem_ninguem_conectado() {
        let s = iniciar().await;
        s.desligar.cancel();
        timeout(ESPERA, s.tarefa)
            .await
            .expect("o servidor não terminou")
            .unwrap();
    }

    #[tokio::test]
    async fn desligar_no_meio_de_uma_linha() {
        let s = iniciar().await;
        let mut ana = Cliente::entrar(s.endereco, "ana").await;
        // Meia linha, sem `\n`: o servidor está esperando o resto.
        ana.escrita.write_all(b"metade de uma").await.unwrap();
        // Dá tempo de o servidor ler esses bytes para o buffer do `Framed`.
        // (Se ele fechasse o socket com bytes ainda não lidos no sistema,
        // o TCP mandaria um RST, e o cliente veria "connection reset".)
        tokio::time::sleep(Duration::from_millis(100)).await;
        s.desligar.cancel();
        assert_eq!(ana.ler().await, "*** servidor encerrando");
        ana.deve_fechar().await;
        timeout(ESPERA, s.tarefa).await.unwrap().unwrap();
    }
}

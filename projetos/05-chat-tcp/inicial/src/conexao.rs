//! Uma task por cliente: lê as linhas do socket, interpreta e responde, e
//! repassa ao cliente o que chega pela sala.

use std::sync::Arc;

use futures_util::{SinkExt, StreamExt};
use tokio::net::TcpStream;
use tokio::sync::broadcast::{self, error::RecvError};
use tokio_util::codec::{Framed, LinesCodec, LinesCodecError};
use tokio_util::sync::CancellationToken;

use crate::protocolo::{self, Comando};
use crate::sala::{Evento, Membro, Sala};

/// Linhas maiores que isso (em bytes) são recusadas e descartadas.
pub const MAX_LINHA: usize = 1024;

/// Id usado nos eventos que o próprio servidor gera para uma conexão.
const SERVIDOR: u64 = 0;

type Linhas = Framed<TcpStream, LinesCodec>;

/// Atende um cliente do começo ao fim.
///
/// Devolve `Err` se a conexão caiu, se chegaram bytes que não são UTF-8 ou
/// uma linha maior que [`MAX_LINHA`]. Em qualquer caso, antes de voltar, o
/// apelido é liberado e a sala fica sabendo que ele saiu.
pub async fn atender(
    stream: TcpStream,
    sala: Arc<Sala>,
    desligar: CancellationToken,
) -> Result<(), LinesCodecError> {
    let mut linhas = Framed::new(stream, LinesCodec::new_with_max_length(MAX_LINHA));
    let mut membro = sala.entrar();

    let resultado = conversar(&mut linhas, &sala, &mut membro, &desligar).await;

    // Chega aqui com Ok ou com Err: a limpeza não pode ficar atrás de um `?`.
    sala.sair(&membro.apelido);
    // No desligamento, todos estão saindo ao mesmo tempo: sem anúncio.
    if !desligar.is_cancelled() {
        sala.anunciar(membro.id, format!("*** {} saiu", membro.apelido));
    }
    resultado
}

/// O laço principal de uma conexão.
async fn conversar(
    linhas: &mut Linhas,
    sala: &Sala,
    membro: &mut Membro,
    desligar: &CancellationToken,
) -> Result<(), LinesCodecError> {
    // Etapa 3: mande as boas-vindas, anuncie a entrada e leia as linhas com
    // `while let Some(linha) = linhas.next().await`, chamando `tratar`.
    // Etapa 4: troque o `while let` por `loop` + `tokio::select!` entre
    // `linhas.next()` e `receber(&mut membro.rx)`.
    // Etapa 5: um terceiro ramo, `desligar.cancelled()`.
    todo!()
}

/// Executa uma linha do cliente. Devolve `false` se a conexão deve acabar.
async fn tratar(
    linha: &str,
    linhas: &mut Linhas,
    sala: &Sala,
    membro: &mut Membro,
) -> Result<bool, LinesCodecError> {
    // Etapa 3: `match protocolo::interpretar(linha)`, um braço por comando.
    // Respostas só para este cliente: `linhas.send(..).await?`.
    // Para todo mundo: `sala.anunciar(membro.id, ..)`.
    todo!()
}

/// O próximo evento da sala. Se esta conexão ficou para trás e o canal
/// descartou eventos, devolve um aviso no lugar deles. `None` quando o
/// canal fechou.
///
/// É cancel safe, porque `broadcast::Receiver::recv` é.
pub async fn receber(rx: &mut broadcast::Receiver<Evento>) -> Option<Evento> {
    // Etapa 4: `rx.recv().await` e um `match` em `RecvError::Lagged(n)` e
    // `RecvError::Closed`.
    todo!()
}

#[cfg(test)]
mod step4_tests {
    use super::*;

    fn evento(texto: &str) -> Evento {
        Evento {
            autor: 7,
            texto: texto.into(),
        }
    }

    #[tokio::test]
    async fn receber_entrega_em_ordem() {
        let (tx, mut rx) = broadcast::channel(8);
        tx.send(evento("a")).unwrap();
        tx.send(evento("b")).unwrap();
        assert_eq!(receber(&mut rx).await, Some(evento("a")));
        assert_eq!(receber(&mut rx).await, Some(evento("b")));
    }

    #[tokio::test]
    async fn receber_avisa_quando_ficou_para_tras() {
        // Capacidade 2: o terceiro `send` descarta o mais antigo, e assim por diante.
        let (tx, mut rx) = broadcast::channel(2);
        for t in ["1", "2", "3", "4", "5"] {
            tx.send(evento(t)).unwrap();
        }
        let aviso = receber(&mut rx).await.unwrap();
        assert_eq!(aviso.autor, SERVIDOR);
        assert_eq!(aviso.texto, "*** você perdeu 3 mensagens");
        // O receptor continua valendo: recebe os que sobraram.
        assert_eq!(receber(&mut rx).await, Some(evento("4")));
        assert_eq!(receber(&mut rx).await, Some(evento("5")));
    }

    #[tokio::test]
    async fn receber_devolve_none_quando_o_canal_fecha() {
        let (tx, mut rx) = broadcast::channel(2);
        tx.send(evento("último")).unwrap();
        drop(tx);
        assert_eq!(receber(&mut rx).await, Some(evento("último")));
        assert_eq!(receber(&mut rx).await, None);
    }
}

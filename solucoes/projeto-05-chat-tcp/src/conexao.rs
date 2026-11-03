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
    linhas
        .send(format!(
            "*** bem-vindo, {}! comandos: /nick NOME, /quem, /sair",
            membro.apelido
        ))
        .await?;
    sala.anunciar(membro.id, format!("*** {} entrou", membro.apelido));

    loop {
        // As três esperas são cancel safe: se uma termina primeiro, as
        // outras não perderam nada (o que já chegou do socket fica no buffer
        // do `Framed`; o que está no canal continua lá).
        tokio::select! {
            () = desligar.cancelled() => {
                linhas.send("*** servidor encerrando").await?;
                return Ok(());
            }
            linha = linhas.next() => match linha {
                // O cliente fechou a conexão.
                None => return Ok(()),
                Some(Err(erro)) => {
                    // Depois de um erro, o `Framed` não lê mais nada (o
                    // próximo `next` dá `None`): a conexão acaba aqui.
                    if let LinesCodecError::MaxLineLengthExceeded = erro {
                        let aviso = format!("*** linha longa demais (máximo {MAX_LINHA} bytes)");
                        linhas.send(aviso).await?;
                    }
                    return Err(erro);
                }
                Some(Ok(linha)) => {
                    if !tratar(&linha, linhas, sala, membro).await? {
                        return Ok(());
                    }
                }
            },
            evento = receber(&mut membro.rx) => match evento {
                // O `Sender` mora na `Sala`, que vive enquanto houver conexões:
                // na prática, `None` não acontece. Se acontecer, encerre.
                None => return Ok(()),
                Some(evento) if evento.autor == membro.id => {}
                Some(evento) => linhas.send(evento.texto).await?,
            },
        }
    }
}

/// Executa uma linha do cliente. Devolve `false` se a conexão deve acabar.
async fn tratar(
    linha: &str,
    linhas: &mut Linhas,
    sala: &Sala,
    membro: &mut Membro,
) -> Result<bool, LinesCodecError> {
    match protocolo::interpretar(linha) {
        Comando::Vazio => {}
        Comando::Mensagem(texto) => {
            sala.anunciar(membro.id, format!("{}: {texto}", membro.apelido));
        }
        Comando::Nick(novo) => match sala.renomear(&membro.apelido, &novo) {
            Ok(()) => {
                let antigo = std::mem::replace(&mut membro.apelido, novo);
                linhas
                    .send(format!("*** agora você é {}", membro.apelido))
                    .await?;
                if antigo != membro.apelido {
                    sala.anunciar(
                        membro.id,
                        format!("*** {antigo} agora é {}", membro.apelido),
                    );
                }
            }
            Err(erro) => linhas.send(format!("*** {erro}")).await?,
        },
        Comando::Quem => {
            let online = sala.quem().join(", ");
            linhas.send(format!("*** online: {online}")).await?;
        }
        Comando::Sair => {
            linhas.send("*** até mais!").await?;
            return Ok(false);
        }
        Comando::Desconhecido(nome) => {
            linhas
                .send(format!("*** comando desconhecido: {nome}"))
                .await?;
        }
    }
    Ok(true)
}

/// O próximo evento da sala. Se esta conexão ficou para trás e o canal
/// descartou eventos, devolve um aviso no lugar deles. `None` quando o
/// canal fechou.
///
/// É cancel safe, porque `broadcast::Receiver::recv` é.
pub async fn receber(rx: &mut broadcast::Receiver<Evento>) -> Option<Evento> {
    match rx.recv().await {
        Ok(evento) => Some(evento),
        Err(RecvError::Lagged(n)) => Some(Evento {
            autor: SERVIDOR,
            texto: if n == 1 {
                "*** você perdeu 1 mensagem".to_string()
            } else {
                format!("*** você perdeu {n} mensagens")
            },
        }),
        Err(RecvError::Closed) => None,
    }
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

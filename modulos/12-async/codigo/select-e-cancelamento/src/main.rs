//! Lições 06 e 07: `select!`, cancelamento, cancellation safety, `pin!` e
//! uma future escrita à mão.

use std::future::Future;
use std::pin::{Pin, pin};
use std::task::{Context, Poll};
use tokio::sync::{mpsc, oneshot};
use tokio::time::{Duration, sleep};

/// Imprime quando é descartado, para mostrar o cancelamento.
struct Avisa(&'static str);

impl Drop for Avisa {
    fn drop(&mut self) {
        println!("drop: {}", self.0);
    }
}

async fn tarefa(nome: &'static str, ms: u64) -> &'static str {
    let _a = Avisa(nome);
    println!("{nome} começou");
    sleep(Duration::from_millis(ms)).await;
    println!("{nome} terminou");
    nome
}

/// Manda 1..=n pelo canal, com uma pausa entre as mensagens.
fn produtor(n: u32, pausa_ms: u64) -> mpsc::Receiver<u32> {
    let (tx, rx) = mpsc::channel(8);
    tokio::spawn(async move {
        for i in 1..=n {
            tx.send(i).await.unwrap();
            sleep(Duration::from_millis(pausa_ms)).await;
        }
    });
    rx
}

/// Lê duas mensagens e devolve o par. NÃO é cancellation safe: se for
/// cancelada entre os dois `recv`, a primeira mensagem se perde.
async fn ler_par(rx: &mut mpsc::Receiver<u32>) -> Option<(u32, u32)> {
    let a = rx.recv().await?;
    let b = rx.recv().await?;
    Some((a, b))
}

/// Versão que perde mensagens: `ler_par` é recriada (e cancelada) a cada volta.
async fn pares_com_perda(mut rx: mpsc::Receiver<u32>, tick_ms: u64) -> Vec<(u32, u32)> {
    let mut tick = tokio::time::interval(Duration::from_millis(tick_ms));
    let mut pares = Vec::new();
    loop {
        tokio::select! {
            par = ler_par(&mut rx) => match par {
                Some(p) => pares.push(p),
                None => return pares,
            },
            _ = tick.tick() => {}
        }
    }
}

/// Versão correta: só `recv` (cancellation safe) dentro do `select!`; o
/// estado parcial fica fora, numa variável que sobrevive às voltas.
async fn pares_sem_perda(mut rx: mpsc::Receiver<u32>, tick_ms: u64) -> Vec<(u32, u32)> {
    let mut tick = tokio::time::interval(Duration::from_millis(tick_ms));
    let mut pares = Vec::new();
    let mut pendente: Option<u32> = None;
    loop {
        tokio::select! {
            msg = rx.recv() => match (msg, pendente.take()) {
                (Some(b), Some(a)) => pares.push((a, b)),
                (Some(a), None) => pendente = Some(a),
                (None, _) => return pares,
            },
            _ = tick.tick() => {}
        }
    }
}

/// Trabalha em ciclos até receber o sinal de parada.
async fn trabalhar_ate_parar(mut parar: oneshot::Receiver<()>) -> u32 {
    let mut ciclos = 0;
    loop {
        tokio::select! {
            biased; // confere a parada primeiro, sempre
            _ = &mut parar => return ciclos,
            _ = sleep(Duration::from_millis(10)) => ciclos += 1,
        }
    }
}

/// Uma future escrita à mão: fica `Pending` algumas vezes antes de terminar.
struct Contagem {
    restante: u32,
}

impl Future for Contagem {
    type Output = &'static str;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.restante == 0 {
            Poll::Ready("fim")
        } else {
            println!("poll: faltam {}", self.restante);
            self.restante -= 1;
            cx.waker().wake_by_ref(); // "me chame de novo"
            Poll::Pending
        }
    }
}

#[tokio::main]
async fn main() {
    println!("== select! cancela quem perde");
    tokio::select! {
        v = tarefa("rapida", 50) => println!("venceu: {v}"),
        v = tarefa("lenta", 500) => println!("venceu: {v}"),
    }
    println!("depois do select");

    println!("\n== cancellation safety");
    println!(
        "com perda: {:?}",
        pares_com_perda(produtor(6, 30), 50).await
    );
    println!(
        "sem perda: {:?}",
        pares_sem_perda(produtor(6, 30), 50).await
    );

    println!("\n== a mesma future em várias voltas: pin!");
    let mut lenta = pin!(async {
        sleep(Duration::from_millis(300)).await;
        "pronto"
    });
    let mut tick = tokio::time::interval(Duration::from_millis(100));
    loop {
        tokio::select! {
            v = &mut lenta => { println!("{v}"); break; }
            _ = tick.tick() => println!("ainda esperando..."),
        }
    }

    println!("\n== parada com oneshot");
    let (parar_tx, parar_rx) = oneshot::channel();
    let trabalhador = tokio::spawn(trabalhar_ate_parar(parar_rx));
    sleep(Duration::from_millis(55)).await;
    parar_tx.send(()).unwrap();
    println!("ciclos completos: {}", trabalhador.await.unwrap());

    println!("\n== uma future escrita à mão");
    println!("{}", Contagem { restante: 3 }.await);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn sem_perda_recebe_tudo() {
        let pares = pares_sem_perda(produtor(6, 3), 5).await;
        assert_eq!(pares, [(1, 2), (3, 4), (5, 6)]);
    }

    #[tokio::test]
    async fn select_devolve_a_mais_rapida() {
        let v = tokio::select! {
            v = tarefa("a", 5) => v,
            v = tarefa("b", 200) => v,
        };
        assert_eq!(v, "a");
    }

    #[tokio::test]
    async fn contagem_termina() {
        assert_eq!(Contagem { restante: 5 }.await, "fim");
    }

    #[tokio::test]
    async fn para_quando_pedido() {
        let (tx, rx) = oneshot::channel();
        tx.send(()).unwrap();
        assert_eq!(trabalhar_ate_parar(rx).await, 0);
    }
}

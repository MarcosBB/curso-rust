//! Lição 05: canais do tokio (`mpsc`, `oneshot`, `broadcast`, `watch`) e
//! estado compartilhado com `std::sync::Mutex` x `tokio::sync::Mutex`.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::sync::{broadcast, mpsc, oneshot, watch};
use tokio::time::{Duration, sleep};

/// Vários produtores, um consumidor, fila limitada a 2 mensagens.
async fn produtores() -> Vec<String> {
    let (tx, mut rx) = mpsc::channel::<String>(2);
    for id in 1..=3 {
        let tx = tx.clone();
        tokio::spawn(async move {
            for i in 1..=2 {
                tx.send(format!("task {id}: msg {i}")).await.unwrap();
            }
        });
    }
    drop(tx); // sem isto, o while abaixo nunca termina
    let mut recebidas = Vec::new();
    while let Some(msg) = rx.recv().await {
        recebidas.push(msg);
    }
    recebidas
}

/// Um pedido com um canal de resposta embutido.
enum Comando {
    Somar(u64),
    Total(oneshot::Sender<u64>),
}

/// Uma task dona de um contador; os outros falam com ela por mensagens.
fn iniciar_contador() -> mpsc::Sender<Comando> {
    let (tx, mut rx) = mpsc::channel(16);
    tokio::spawn(async move {
        let mut total = 0;
        while let Some(cmd) = rx.recv().await {
            match cmd {
                Comando::Somar(n) => total += n,
                Comando::Total(resp) => {
                    let _ = resp.send(total); // quem perguntou pode ter desistido
                }
            }
        }
    });
    tx
}

async fn perguntar_total(contador: &mpsc::Sender<Comando>) -> u64 {
    let (resp_tx, resp_rx) = oneshot::channel();
    contador.send(Comando::Total(resp_tx)).await.unwrap();
    resp_rx.await.unwrap()
}

/// Estado curto e síncrono: `std::sync::Mutex`, solto antes de qualquer `.await`.
async fn contar_visita(visitas: Arc<Mutex<HashMap<String, u32>>>, pagina: String) {
    {
        let mut mapa = visitas.lock().unwrap();
        *mapa.entry(pagina).or_insert(0) += 1;
    } // o guarda morre aqui
    sleep(Duration::from_millis(1)).await;
}

/// Guarda que atravessa `.await`: aí sim, `tokio::sync::Mutex`.
async fn registrar_devagar(log: Arc<tokio::sync::Mutex<Vec<String>>>, msg: String) {
    let mut v = log.lock().await;
    sleep(Duration::from_millis(5)).await; // simula uma escrita lenta
    v.push(msg);
}

#[tokio::main]
async fn main() {
    println!("== mpsc");
    for msg in produtores().await {
        println!("{msg}");
    }

    println!("\n== mpsc + oneshot");
    let contador = iniciar_contador();
    for n in [10, 20, 12] {
        contador.send(Comando::Somar(n)).await.unwrap();
    }
    println!("total: {}", perguntar_total(&contador).await);

    println!("\n== broadcast");
    let (tx, mut rx1) = broadcast::channel::<&str>(16);
    let mut rx2 = tx.subscribe();
    tx.send("aviso geral").unwrap();
    println!("rx1: {}", rx1.recv().await.unwrap());
    println!("rx2: {}", rx2.recv().await.unwrap());

    println!("\n== watch");
    let (tx, mut rx) = watch::channel("iniciando");
    let observador = tokio::spawn(async move {
        while rx.changed().await.is_ok() {
            println!("estado agora: {}", *rx.borrow_and_update());
        }
    });
    for estado in ["rodando", "parando"] {
        sleep(Duration::from_millis(10)).await;
        tx.send(estado).unwrap();
    }
    drop(tx);
    observador.await.unwrap();

    println!("\n== Mutex");
    let visitas = Arc::new(Mutex::new(HashMap::new()));
    let mut handles = Vec::new();
    for pagina in ["/", "/sobre", "/", "/"] {
        handles.push(tokio::spawn(contar_visita(visitas.clone(), pagina.into())));
    }
    for h in handles {
        h.await.unwrap();
    }
    let mut v: Vec<_> = visitas.lock().unwrap().clone().into_iter().collect();
    v.sort();
    println!("visitas: {v:?}");

    let log = Arc::new(tokio::sync::Mutex::new(Vec::new()));
    let (a, b) = tokio::join!(
        tokio::spawn(registrar_devagar(log.clone(), "a".into())),
        tokio::spawn(registrar_devagar(log.clone(), "b".into())),
    );
    a.unwrap();
    b.unwrap();
    println!("log tem {} linhas", log.lock().await.len());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn recebe_todas_as_mensagens() {
        let mut msgs = produtores().await;
        msgs.sort();
        assert_eq!(msgs.len(), 6);
        assert_eq!(msgs[0], "task 1: msg 1");
    }

    #[tokio::test]
    async fn contador_por_mensagens() {
        let contador = iniciar_contador();
        let mut handles = Vec::new();
        for _ in 0..100 {
            let c = contador.clone();
            handles.push(tokio::spawn(async move {
                c.send(Comando::Somar(1)).await.unwrap();
            }));
        }
        for h in handles {
            h.await.unwrap();
        }
        assert_eq!(perguntar_total(&contador).await, 100);
    }

    #[tokio::test]
    async fn visitas_contadas() {
        let visitas = Arc::new(Mutex::new(HashMap::new()));
        let mut handles = Vec::new();
        for _ in 0..50 {
            handles.push(tokio::spawn(contar_visita(visitas.clone(), "/".into())));
        }
        for h in handles {
            h.await.unwrap();
        }
        assert_eq!(visitas.lock().unwrap()["/"], 50);
    }
}

//! Lição 03: timers (`sleep`, `timeout`, `interval`), o que acontece quando
//! você bloqueia o runtime e como `spawn_blocking` resolve.

use std::time::Instant;
use tokio::time::{Duration, interval, sleep, timeout};

async fn trabalho_ruim(id: u32) {
    std::thread::sleep(Duration::from_millis(100)); // bloqueia a thread inteira
    println!("ruim {id} terminou");
}

async fn trabalho_bom(id: u32) {
    sleep(Duration::from_millis(100)).await; // cede a vez enquanto espera
    println!("bom {id} terminou");
}

/// Trabalho de CPU puro: conta os primos menores que `n`.
fn primos_ate(n: u64) -> usize {
    (2..n)
        .filter(|&k| (2..).take_while(|d| d * d <= k).all(|d| k % d != 0))
        .count()
}

/// Roda `primos_ate` numa thread do pool de bloqueio, sem travar o runtime.
async fn primos_em_segundo_plano(n: u64) -> usize {
    tokio::task::spawn_blocking(move || primos_ate(n))
        .await
        .expect("a thread de bloqueio não deve dar panic")
}

// `current_thread`: uma só thread, para o efeito do bloqueio ficar evidente.
#[tokio::main(flavor = "current_thread")]
async fn main() {
    println!("== timeout");
    let r = timeout(Duration::from_millis(100), sleep(Duration::from_secs(5))).await;
    println!("{r:?}");
    let r = timeout(Duration::from_millis(100), async { 42 }).await;
    println!("{r:?}");

    println!("\n== interval");
    let inicio = Instant::now();
    let mut tick = interval(Duration::from_millis(100));
    for _ in 0..3 {
        tick.tick().await;
        println!("tick em {} ms", inicio.elapsed().as_millis());
    }

    println!("\n== bloquear x ceder a vez");
    let inicio = Instant::now();
    tokio::join!(trabalho_ruim(1), trabalho_ruim(2), trabalho_ruim(3));
    println!("std::thread::sleep: {:?}", inicio.elapsed());

    let inicio = Instant::now();
    tokio::join!(trabalho_bom(1), trabalho_bom(2), trabalho_bom(3));
    println!("tokio::time::sleep: {:?}", inicio.elapsed());

    println!("\n== spawn_blocking");
    let inicio = Instant::now();
    let pesado = tokio::spawn(primos_em_segundo_plano(3_000_000));
    sleep(Duration::from_millis(20)).await;
    println!("o runtime continua livre: {:?}", inicio.elapsed());
    let n = pesado.await.unwrap();
    println!("{n} primos em {:?}", inicio.elapsed());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn primos_pequenos() {
        assert_eq!(primos_ate(30), 10);
    }

    #[tokio::test]
    async fn spawn_blocking_devolve_o_valor() {
        assert_eq!(primos_em_segundo_plano(100).await, 25);
    }

    #[tokio::test]
    async fn timeout_estoura() {
        let r = timeout(Duration::from_millis(10), sleep(Duration::from_secs(1))).await;
        assert!(r.is_err());
    }
}

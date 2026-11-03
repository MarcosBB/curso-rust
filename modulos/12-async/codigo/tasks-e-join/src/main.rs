//! Lições 01 e 02: futures preguiçosas, `join!`, `tokio::spawn`, `JoinSet`
//! e milhares de tasks.

use std::time::Instant;
use tokio::task::JoinSet;
use tokio::time::{Duration, sleep};

async fn diga(texto: &str) {
    println!("{texto}");
}

/// Simula uma consulta que demora `ms` milissegundos esperando a rede.
async fn buscar(nome: &str, ms: u64) -> String {
    sleep(Duration::from_millis(ms)).await;
    format!("{nome} ({ms} ms)")
}

/// Espera todas as tasks e devolve os resultados na ordem em que terminaram.
async fn na_ordem_de_chegada(tempos: &[(&'static str, u64)]) -> Vec<&'static str> {
    let mut set = JoinSet::new();
    for &(nome, ms) in tempos {
        set.spawn(async move {
            sleep(Duration::from_millis(ms)).await;
            nome
        });
    }
    let mut ordem = Vec::new();
    while let Some(r) = set.join_next().await {
        ordem.push(r.expect("a task não deve dar panic"));
    }
    ordem
}

/// Dispara `n` tasks que dormem 100 ms cada e soma os ids devolvidos.
async fn muitas_tasks(n: u64) -> u64 {
    let mut handles = Vec::new();
    for i in 0..n {
        handles.push(tokio::spawn(async move {
            sleep(Duration::from_millis(100)).await;
            i
        }));
    }
    let mut soma = 0;
    for h in handles {
        soma += h.await.unwrap();
    }
    soma
}

#[tokio::main]
async fn main() {
    println!("== futures são preguiçosas");
    let f = diga("dois");
    println!("antes do await");
    f.await;

    println!("\n== sequencial x join!");
    let inicio = Instant::now();
    let a = buscar("usuarios", 300).await;
    let b = buscar("pedidos", 200).await;
    println!("sequencial: {a}, {b} em {:?}", inicio.elapsed());

    let inicio = Instant::now();
    let (a, b) = tokio::join!(buscar("usuarios", 300), buscar("pedidos", 200));
    println!("join!:      {a}, {b} em {:?}", inicio.elapsed());

    println!("\n== tokio::spawn");
    let mut nomes = vec![String::from("ana")];
    nomes.push(String::from("bia"));
    let h = tokio::spawn(async move { nomes.join(" e ") });
    println!("a task devolveu {:?}", h.await.unwrap());

    println!("\n== JoinSet");
    let ordem = na_ordem_de_chegada(&[("a", 30), ("b", 10), ("c", 20)]).await;
    println!("ordem de chegada: {ordem:?}");

    println!("\n== 100 000 tasks");
    let inicio = Instant::now();
    let soma = muitas_tasks(100_000).await;
    println!("soma {soma} em {:?}", inicio.elapsed());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn join_roda_as_duas() {
        let (a, b) = tokio::join!(buscar("x", 1), buscar("y", 1));
        assert_eq!(a, "x (1 ms)");
        assert_eq!(b, "y (1 ms)");
    }

    #[tokio::test]
    async fn joinset_entrega_por_ordem_de_termino() {
        let ordem = na_ordem_de_chegada(&[("a", 60), ("b", 10), ("c", 35)]).await;
        assert_eq!(ordem, ["b", "c", "a"]);
    }

    #[tokio::test]
    async fn muitas_tasks_somam_certo() {
        assert_eq!(muitas_tasks(1_000).await, 499_500);
    }
}

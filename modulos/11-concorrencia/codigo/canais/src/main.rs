//! Passagem de mensagens com `std::sync::mpsc` (lição 03).

use std::sync::mpsc;
use std::thread;
use std::time::Duration;

/// Vários produtores, um consumidor: cada worker manda os quadrados do seu
/// intervalo, e o main soma tudo. A ordem de chegada varia; a soma, não.
fn soma_de_quadrados(workers: u64, por_worker: u64) -> u64 {
    let (tx, rx) = mpsc::channel();
    for id in 0..workers {
        let tx = tx.clone();
        thread::spawn(move || {
            let inicio = id * por_worker + 1;
            for n in inicio..inicio + por_worker {
                tx.send(n * n).unwrap();
            }
        });
    }
    // Sem este drop, o `for` abaixo nunca termina: o `tx` original ainda existe.
    drop(tx);
    rx.iter().sum()
}

/// Um pipeline de três estágios ligados por canais:
/// leitura -> contagem de palavras -> agregação (no main).
fn pipeline(texto: &str) -> (usize, usize) {
    let linhas: Vec<String> = texto.lines().map(String::from).collect();

    let (tx_linhas, rx_linhas) = mpsc::sync_channel::<String>(4);
    let (tx_contagens, rx_contagens) = mpsc::channel::<usize>();

    let leitor = thread::spawn(move || {
        for linha in linhas {
            // Com capacidade 4, `send` espera se o próximo estágio atrasar.
            tx_linhas.send(linha).unwrap();
        }
    });

    let contador = thread::spawn(move || {
        for linha in rx_linhas {
            tx_contagens.send(linha.split_whitespace().count()).unwrap();
        }
    });

    let mut n_linhas = 0;
    let mut n_palavras = 0;
    for c in rx_contagens {
        n_linhas += 1;
        n_palavras += c;
    }
    leitor.join().unwrap();
    contador.join().unwrap();
    (n_linhas, n_palavras)
}

fn main() {
    // Fan-in: três produtores, mensagens chegam numa ordem que varia.
    let (tx, rx) = mpsc::channel();
    for id in 1..=3 {
        let tx = tx.clone();
        thread::spawn(move || {
            for i in 1..=2 {
                tx.send(format!("worker {id}: msg {i}")).unwrap();
            }
        });
    }
    drop(tx);
    for msg in rx {
        println!("{msg}");
    }

    println!("soma dos quadrados: {}", soma_de_quadrados(4, 250));

    let texto = "o rato roeu\na roupa do rei\nde Roma\n";
    let (l, p) = pipeline(texto);
    println!("pipeline: {l} linhas, {p} palavras");

    // Esperar com prazo: recv_timeout devolve Err se nada chegar a tempo.
    let (tx, rx) = mpsc::channel::<&str>();
    let h = thread::spawn(move || {
        thread::sleep(Duration::from_millis(50));
        tx.send("atrasado").unwrap();
    });
    match rx.recv_timeout(Duration::from_millis(10)) {
        Ok(m) => println!("chegou: {m}"),
        Err(e) => println!("timeout: {e}"),
    }
    println!("depois: {:?}", rx.recv());
    h.join().unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quadrados() {
        let esperado: u64 = (1..=1000u64).map(|n| n * n).sum();
        assert_eq!(soma_de_quadrados(4, 250), esperado);
    }

    #[test]
    fn pipeline_conta_linhas_e_palavras() {
        assert_eq!(pipeline("a b\nc\n\nd e f"), (4, 6));
        assert_eq!(pipeline(""), (0, 0));
    }
}

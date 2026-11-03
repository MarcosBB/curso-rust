//! Threads com `spawn`, `join`, `move` e `thread::scope` (lição 01).

use std::thread;
use std::time::Duration;

/// Soma `dados` dividindo o trabalho entre `partes` threads com escopo.
/// As threads emprestam fatias de `dados`: nada é copiado nem movido.
fn soma_paralela(dados: &[u64], partes: usize) -> u64 {
    let tamanho = dados.len().div_ceil(partes.max(1)).max(1);
    thread::scope(|s| {
        let handles: Vec<_> = dados
            .chunks(tamanho)
            .map(|pedaco| s.spawn(move || pedaco.iter().sum::<u64>()))
            .collect();
        handles.into_iter().map(|h| h.join().unwrap()).sum()
    })
}

/// Multiplica cada elemento por `fator`, com uma thread por pedaço.
/// Cada thread recebe um `&mut` para um pedaço diferente.
fn escalar_paralelo(dados: &mut [i32], fator: i32, partes: usize) {
    let tamanho = dados.len().div_ceil(partes.max(1)).max(1);
    thread::scope(|s| {
        for pedaco in dados.chunks_mut(tamanho) {
            s.spawn(move || {
                for x in pedaco {
                    *x *= fator;
                }
            });
        }
    });
}

fn main() {
    // 1. spawn + join: a thread devolve um valor pelo JoinHandle.
    let nomes = [String::from("Ana"), String::from("Bia")];
    let h = thread::spawn(move || {
        thread::sleep(Duration::from_millis(10));
        nomes.iter().map(|n| n.len()).sum::<usize>()
    });
    println!("total de letras: {}", h.join().unwrap());

    // 2. Um panic numa thread vira um Err no join.
    let h = thread::spawn(|| {
        let v: Vec<i32> = Vec::new();
        v[0]
    });
    match h.join() {
        Ok(x) => println!("deu {x}"),
        Err(_) => println!("a thread entrou em pânico, e o main continua"),
    }

    // 3. Threads com nome, úteis em mensagens de pânico e em profilers.
    let h = thread::Builder::new()
        .name("contador".into())
        .spawn(|| format!("rodando em '{}'", thread::current().name().unwrap()))
        .unwrap();
    println!("{}", h.join().unwrap());

    // 4. thread::scope: emprestar dados locais, sem move e sem Arc.
    let dados: Vec<u64> = (1..=1_000).collect();
    println!("soma: {}", soma_paralela(&dados, 4));

    let mut v = vec![1, 2, 3, 4, 5, 6, 7];
    escalar_paralelo(&mut v, 10, 3);
    println!("escalado: {v:?}");

    let n = thread::available_parallelism().map_or(1, |n| n.get());
    println!("núcleos disponíveis: {n}");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn soma_igual_a_sequencial() {
        let dados: Vec<u64> = (1..=10_001).collect();
        let esperado: u64 = dados.iter().sum();
        for partes in [1, 2, 3, 7, 64] {
            assert_eq!(soma_paralela(&dados, partes), esperado);
        }
    }

    #[test]
    fn soma_vazia_e_partes_zero() {
        assert_eq!(soma_paralela(&[], 4), 0);
        assert_eq!(soma_paralela(&[5, 6], 0), 11);
    }

    #[test]
    fn escalar_todos_os_elementos() {
        let mut v: Vec<i32> = (1..=10).collect();
        escalar_paralelo(&mut v, 2, 4);
        assert_eq!(v, (1..=10).map(|x| x * 2).collect::<Vec<_>>());
    }
}

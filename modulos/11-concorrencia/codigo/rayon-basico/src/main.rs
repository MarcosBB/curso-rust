//! Paralelismo de dados com `rayon` (lição 07).
//! Rode com `cargo run --release`: em modo debug, os tempos não dizem nada.

use rayon::prelude::*;
use std::collections::HashMap;
use std::time::Instant;

fn eh_primo(n: u64) -> bool {
    if n < 2 {
        return false;
    }
    let mut d = 2;
    while d * d <= n {
        if n.is_multiple_of(d) {
            return false;
        }
        d += 1;
    }
    true
}

fn contar_primos_seq(limite: u64) -> usize {
    (0..limite).filter(|&n| eh_primo(n)).count()
}

/// A mesma cadeia, trocando `into_iter` implícito por `into_par_iter`.
fn contar_primos_par(limite: u64) -> usize {
    (0..limite).into_par_iter().filter(|&n| eh_primo(n)).count()
}

/// Frequência de palavras: cada thread do rayon monta um mapa parcial
/// (`fold`), e os mapas parciais são juntados dois a dois (`reduce`).
fn frequencias(linhas: &[String]) -> HashMap<String, usize> {
    linhas
        .par_iter()
        .fold(HashMap::new, |mut mapa, linha| {
            for p in linha.split_whitespace() {
                *mapa.entry(p.to_string()).or_insert(0) += 1;
            }
            mapa
        })
        .reduce(HashMap::new, |mut a, b| {
            for (p, n) in b {
                *a.entry(p).or_insert(0) += n;
            }
            a
        })
}

fn medir<T>(nome: &str, f: impl FnOnce() -> T) -> T {
    let inicio = Instant::now();
    let r = f();
    println!("{nome:>12}: {:>8.1?}", inicio.elapsed());
    r
}

fn main() {
    println!("threads do rayon: {}", rayon::current_num_threads());

    let limite = 3_000_000;
    let a = medir("sequencial", || contar_primos_seq(limite));
    let b = medir("paralelo", || contar_primos_par(limite));
    assert_eq!(a, b);
    println!("{a} primos abaixo de {limite}");

    let linhas: Vec<String> = (0..200_000)
        .map(|i| format!("linha {} tem a palavra {}", i % 7, i % 3))
        .collect();
    let freq = medir("frequências", || frequencias(&linhas));
    println!("'palavra' aparece {} vezes", freq["palavra"]);

    let mut numeros: Vec<u64> = (0..5_000_000).map(|i| (i * 7_919) % 1_000_003).collect();
    medir("par_sort", || numeros.par_sort_unstable());
    println!(
        "menor: {}, maior: {}",
        numeros[0],
        numeros[numeros.len() - 1]
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn paralelo_igual_a_sequencial() {
        for limite in [0, 1, 2, 100, 10_000] {
            assert_eq!(contar_primos_par(limite), contar_primos_seq(limite));
        }
        assert_eq!(contar_primos_par(100), 25);
    }

    #[test]
    fn frequencias_somam_tudo() {
        let linhas = vec!["a b a".to_string(), "b c".to_string(), String::new()];
        let f = frequencias(&linhas);
        assert_eq!(f["a"], 2);
        assert_eq!(f["b"], 2);
        assert_eq!(f["c"], 1);
        assert_eq!(f.len(), 3);
    }
}

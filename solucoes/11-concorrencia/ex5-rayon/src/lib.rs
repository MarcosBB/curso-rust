use rayon::prelude::*;
use std::cmp::Reverse;

fn eh_primo(n: u64) -> bool {
    n >= 2
        && (2..)
            .take_while(|d| d * d <= n)
            .all(|d| !n.is_multiple_of(d))
}

/// Todos os primos menores que `n`, em ordem crescente.
pub fn primos_ate(n: u64) -> Vec<u64> {
    // `collect` de um iterador paralelo preserva a ordem original.
    (0..n).into_par_iter().filter(|&x| eh_primo(x)).collect()
}

/// Número de passos da sequência de Collatz de `n` até chegar a 1.
pub fn passos_collatz(mut n: u64) -> u32 {
    let mut passos = 0;
    while n > 1 {
        n = if n.is_multiple_of(2) {
            n / 2
        } else {
            3 * n + 1
        };
        passos += 1;
    }
    passos
}

/// O número em `1..limite` com a sequência de Collatz mais longa, e o número
/// de passos. Em caso de empate, o menor número. `None` se `limite <= 1`.
pub fn maior_collatz(limite: u64) -> Option<(u64, u32)> {
    (1..limite)
        .into_par_iter()
        .map(|n| (n, passos_collatz(n)))
        // A chave desempata pelo menor `n`, então o resultado não depende
        // de como o rayon dividiu o trabalho.
        .max_by_key(|&(n, p)| (p, Reverse(n)))
}

/// Soma dos quadrados, em paralelo.
pub fn soma_quadrados(v: &[i64]) -> i64 {
    v.par_iter().map(|x| x * x).sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn primos() {
        assert_eq!(primos_ate(30), [2, 3, 5, 7, 11, 13, 17, 19, 23, 29]);
        assert!(primos_ate(2).is_empty());
        let seq: Vec<u64> = (0..50_000).filter(|&x| eh_primo(x)).collect();
        assert_eq!(primos_ate(50_000), seq);
    }

    #[test]
    fn collatz() {
        assert_eq!(passos_collatz(1), 0);
        assert_eq!(passos_collatz(6), 8);
        assert_eq!(maior_collatz(10), Some((9, 19)));
        assert_eq!(maior_collatz(1), None);
        assert_eq!(maior_collatz(1_000_000), Some((837_799, 524)));
    }

    #[test]
    fn quadrados() {
        let v: Vec<i64> = (-100..=100).collect();
        let seq: i64 = v.iter().map(|x| x * x).sum();
        assert_eq!(soma_quadrados(&v), seq);
        assert_eq!(soma_quadrados(&[]), 0);
    }
}

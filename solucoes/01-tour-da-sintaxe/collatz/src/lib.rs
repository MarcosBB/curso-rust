/// Quantidade de termos da sequência de Collatz que começa em `n`,
/// contando o próprio `n` e o 1 final.
pub fn tamanho_collatz(mut n: u64) -> u32 {
    let mut termos = 1;
    while n != 1 {
        n = if n.is_multiple_of(2) {
            n / 2
        } else {
            3 * n + 1
        };
        termos += 1;
    }
    termos
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn comeca_em_um() {
        assert_eq!(tamanho_collatz(1), 1);
    }

    #[test]
    fn potencia_de_dois() {
        assert_eq!(tamanho_collatz(8), 4); // 8, 4, 2, 1
    }

    #[test]
    fn onze() {
        assert_eq!(tamanho_collatz(11), 15);
    }

    #[test]
    fn vinte_e_sete() {
        assert_eq!(tamanho_collatz(27), 112);
    }
}

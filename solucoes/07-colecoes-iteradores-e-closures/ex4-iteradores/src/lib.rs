use std::iter;

/// Sequência de Collatz a partir de `n`, terminando em 1 (inclusive).
/// Para `n == 0`, a sequência é vazia.
pub struct Collatz {
    proximo: Option<u64>,
}

pub fn collatz(n: u64) -> Collatz {
    Collatz {
        proximo: (n > 0).then_some(n),
    }
}

impl Iterator for Collatz {
    type Item = u64;

    fn next(&mut self) -> Option<u64> {
        let atual = self.proximo?;
        self.proximo = match atual {
            1 => None,
            x if x % 2 == 0 => Some(x / 2),
            x => Some(3 * x + 1),
        };
        Some(atual)
    }
}

/// Bônus: a mesma sequência com `iter::successors`, sem struct.
pub fn collatz_successors(n: u64) -> impl Iterator<Item = u64> {
    iter::successors((n > 0).then_some(n), |&x| match x {
        1 => None,
        x if x % 2 == 0 => Some(x / 2),
        x => Some(3 * x + 1),
    })
}

/// Pares de elementos consecutivos de um slice: (v[0], v[1]), (v[1], v[2]), ...
pub struct Pares<'a, T> {
    resto: &'a [T],
}

pub fn pares<T>(v: &[T]) -> Pares<'_, T> {
    Pares { resto: v }
}

impl<'a, T> Iterator for Pares<'a, T> {
    type Item = (&'a T, &'a T);

    fn next(&mut self) -> Option<(&'a T, &'a T)> {
        let [a, b, ..] = self.resto else {
            return None;
        };
        self.resto = &self.resto[1..];
        Some((a, b))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn collatz_de_6() {
        assert_eq!(
            collatz(6).collect::<Vec<_>>(),
            [6, 3, 10, 5, 16, 8, 4, 2, 1]
        );
    }

    #[test]
    fn collatz_casos_de_borda() {
        assert_eq!(collatz(1).collect::<Vec<_>>(), [1]);
        assert_eq!(collatz(0).count(), 0);
    }

    #[test]
    fn collatz_de_27() {
        assert_eq!(collatz(27).count(), 112);
        assert_eq!(collatz(27).max(), Some(9232));
    }

    #[test]
    fn bonus_successors_igual() {
        for n in 0..50 {
            assert!(collatz(n).eq(collatz_successors(n)), "n = {n}");
        }
    }

    #[test]
    fn pares_consecutivos() {
        let v = [1, 4, 9, 16];
        let p: Vec<(&i32, &i32)> = pares(&v).collect();
        assert_eq!(p, [(&1, &4), (&4, &9), (&9, &16)]);
        let difs: Vec<i32> = pares(&v).map(|(a, b)| b - a).collect();
        assert_eq!(difs, [3, 5, 7]);
    }

    #[test]
    fn pares_curtos() {
        assert_eq!(pares(&[42]).count(), 0);
        assert_eq!(pares::<i32>(&[]).count(), 0);
    }

    #[test]
    fn pares_com_strings_nao_copia() {
        let nomes = vec![
            String::from("ana"),
            String::from("bia"),
            String::from("caio"),
        ];
        let maior = pares(&nomes).max_by_key(|(a, b)| a.len() + b.len());
        assert_eq!(maior, Some((&nomes[1], &nomes[2])));
        assert!(std::ptr::eq(maior.unwrap().0, &nomes[1]));
    }
}

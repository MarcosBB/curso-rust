//! Iteradores escritos à mão (lição 08).

use std::iter;

/// Conta de `atual` até 1. Só `next` é obrigatório.
pub struct Regressiva {
    atual: u32,
}

pub fn regressiva(de: u32) -> Regressiva {
    Regressiva { atual: de }
}

impl Iterator for Regressiva {
    type Item = u32;

    fn next(&mut self) -> Option<u32> {
        if self.atual == 0 {
            return None;
        }
        self.atual -= 1;
        Some(self.atual + 1)
    }
}

/// Com `next_back`, `rev()` passa a funcionar: conta de 1 até `atual`.
/// `inicio` guarda quantos já saíram por trás.
pub struct RegressivaDupla {
    inicio: u32,
    atual: u32,
}

pub fn regressiva_dupla(de: u32) -> RegressivaDupla {
    RegressivaDupla {
        inicio: 0,
        atual: de,
    }
}

impl Iterator for RegressivaDupla {
    type Item = u32;

    fn next(&mut self) -> Option<u32> {
        if self.atual == self.inicio {
            return None;
        }
        self.atual -= 1;
        Some(self.atual + 1)
    }
}

impl DoubleEndedIterator for RegressivaDupla {
    fn next_back(&mut self) -> Option<u32> {
        if self.atual == self.inicio {
            return None;
        }
        self.inicio += 1;
        Some(self.inicio)
    }
}

/// Fibonacci em `u64`: termina quando o próximo termo estouraria.
pub struct Fibonacci {
    a: u64,
    b: u64,
}

pub fn fibonacci() -> Fibonacci {
    Fibonacci { a: 0, b: 1 }
}

impl Iterator for Fibonacci {
    type Item = u64;

    fn next(&mut self) -> Option<u64> {
        let atual = self.a;
        let proximo = self.a.checked_add(self.b)?;
        self.a = self.b;
        self.b = proximo;
        Some(atual)
    }
}

/// Palavras de um texto, como fatias dele (sem cópia).
pub struct Palavras<'a> {
    resto: &'a str,
}

pub fn palavras(texto: &str) -> Palavras<'_> {
    Palavras { resto: texto }
}

impl<'a> Iterator for Palavras<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<&'a str> {
        let texto = self.resto.trim_start();
        if texto.is_empty() {
            return None;
        }
        let fim = texto.find(' ').unwrap_or(texto.len());
        let (palavra, resto) = texto.split_at(fim);
        self.resto = resto;
        Some(palavra)
    }
}

/// Uma coleção própria que delega para o `Vec` interno.
#[derive(Debug, Default)]
pub struct Playlist {
    musicas: Vec<String>,
}

impl Playlist {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn adicionar(&mut self, musica: &str) {
        self.musicas.push(musica.to_string());
    }

    pub fn iter(&self) -> std::slice::Iter<'_, String> {
        self.musicas.iter()
    }
}

impl<'a> IntoIterator for &'a Playlist {
    type Item = &'a String;
    type IntoIter = std::slice::Iter<'a, String>;

    fn into_iter(self) -> Self::IntoIter {
        self.musicas.iter()
    }
}

impl IntoIterator for Playlist {
    type Item = String;
    type IntoIter = std::vec::IntoIter<String>;

    fn into_iter(self) -> Self::IntoIter {
        self.musicas.into_iter()
    }
}

/// Collatz sem struct: `successors` calcula cada termo a partir do anterior.
pub fn collatz(n: u64) -> impl Iterator<Item = u64> {
    iter::successors(Some(n), |&x| {
        if x <= 1 {
            None
        } else if x % 2 == 0 {
            Some(x / 2)
        } else {
            Some(3 * x + 1)
        }
    })
}

/// Múltiplos de `de` até `ate`: uma cadeia de adaptadores devolvida
/// como `impl Iterator`. O `move` leva `de` para dentro da closure.
pub fn multiplos(de: u32, ate: u32) -> impl Iterator<Item = u32> {
    (1..=ate).filter(move |n| n % de == 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn regressiva_e_metodos_de_graca() {
        assert_eq!(regressiva(3).collect::<Vec<_>>(), [3, 2, 1]);
        let soma: u32 = regressiva(10).filter(|n| n % 2 == 0).map(|n| n * n).sum();
        assert_eq!(soma, 220);
        assert_eq!(regressiva(0).count(), 0);
    }

    #[test]
    fn regressiva_dupla_anda_pelos_dois_lados() {
        assert_eq!(regressiva_dupla(3).rev().collect::<Vec<_>>(), [1, 2, 3]);
        let mut it = regressiva_dupla(4);
        assert_eq!(
            (
                it.next(),
                it.next_back(),
                it.next(),
                it.next_back(),
                it.next()
            ),
            (Some(4), Some(1), Some(3), Some(2), None)
        );
    }

    #[test]
    fn fibonacci_para_no_estouro() {
        assert_eq!(fibonacci().take(6).collect::<Vec<_>>(), [0, 1, 1, 2, 3, 5]);
        assert_eq!(fibonacci().count(), 92);
        assert_eq!(fibonacci().last(), Some(4_660_046_610_375_530_309));
    }

    #[test]
    fn palavras_apontam_para_o_texto() {
        let texto = String::from("  borrow   checker feliz ");
        let v: Vec<&str> = palavras(&texto).collect();
        assert_eq!(v, ["borrow", "checker", "feliz"]);
        assert!(std::ptr::eq(v[0].as_ptr(), texto[2..].as_ptr()));
    }

    #[test]
    fn playlist_nas_tres_formas() {
        let mut p = Playlist::new();
        p.adicionar("Asa Branca");
        p.adicionar("Garota de Ipanema");
        let mut n = 0;
        for m in &p {
            n += m.len();
        }
        assert_eq!(n, 27);
        assert_eq!(p.iter().count(), 2);
        let todas: Vec<String> = p.into_iter().collect();
        assert_eq!(todas, ["Asa Branca", "Garota de Ipanema"]);
    }

    #[test]
    fn sem_struct() {
        assert_eq!(
            collatz(6).collect::<Vec<_>>(),
            [6, 3, 10, 5, 16, 8, 4, 2, 1]
        );
        assert_eq!(collatz(27).count(), 112);
        assert_eq!(multiplos(3, 10).collect::<Vec<_>>(), [3, 6, 9]);
    }
}

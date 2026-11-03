//! Contagem de um texto: linhas, palavras e frequência de cada palavra.

use std::cmp::Reverse;
use std::collections::HashMap;

/// O que se sabe de um texto (ou de vários, depois de [`Contagem::juntar`]).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Contagem {
    pub linhas: usize,
    pub palavras: usize,
    /// Palavra (em minúsculas) → quantas vezes apareceu.
    pub frequencias: HashMap<String, usize>,
}

/// Separa o texto em palavras: sequências de letras e dígitos, em minúsculas.
///
/// `"Olá, mundo! olá"` → `["olá", "mundo", "olá"]`.
pub fn palavras(texto: &str) -> impl Iterator<Item = String> {
    texto
        .split(|c: char| !c.is_alphanumeric())
        .filter(|p| !p.is_empty())
        .map(str::to_lowercase)
}

/// Conta um texto inteiro.
pub fn contar(texto: &str) -> Contagem {
    let mut c = Contagem {
        linhas: texto.lines().count(),
        ..Contagem::default()
    };
    for p in palavras(texto) {
        c.palavras += 1;
        *c.frequencias.entry(p).or_insert(0) += 1;
    }
    c
}

impl Contagem {
    /// Soma `outra` nesta contagem. A ordem não importa: `a.juntar(b)` e
    /// `b.juntar(a)` dão o mesmo resultado, e é isso que deixa juntar as
    /// contagens parciais das threads em qualquer ordem.
    pub fn juntar(&mut self, outra: Contagem) {
        self.linhas += outra.linhas;
        self.palavras += outra.palavras;
        for (p, n) in outra.frequencias {
            *self.frequencias.entry(p).or_insert(0) += n;
        }
    }

    /// As `n` palavras mais frequentes, da mais para a menos frequente.
    /// Empates saem em ordem alfabética, para o resultado ser sempre o mesmo.
    pub fn mais_frequentes(&self, n: usize) -> Vec<(&str, usize)> {
        let mut todas: Vec<(&str, usize)> = self
            .frequencias
            .iter()
            .map(|(p, &q)| (p.as_str(), q))
            .collect();
        todas.sort_unstable_by_key(|&(p, q)| (Reverse(q), p));
        todas.truncate(n);
        todas
    }
}

#[cfg(test)]
mod step1_tests {
    use super::*;

    #[test]
    fn separa_palavras_e_ignora_pontuacao() {
        let v: Vec<String> = palavras("Olá, mundo! olá... (de novo) 2026").collect();
        assert_eq!(v, ["olá", "mundo", "olá", "de", "novo", "2026"]);
    }

    #[test]
    fn conta_linhas_e_palavras() {
        let c = contar("um dois\ntrês\n\nquatro cinco seis\n");
        assert_eq!(c.linhas, 4);
        assert_eq!(c.palavras, 6);
    }

    #[test]
    fn texto_vazio() {
        assert_eq!(contar(""), Contagem::default());
    }

    #[test]
    fn frequencias_sem_diferenciar_maiusculas() {
        let c = contar("Rust rust RUST é bom");
        assert_eq!(c.frequencias["rust"], 3);
        assert_eq!(c.frequencias.len(), 3);
    }

    #[test]
    fn juntar_soma_tudo() {
        let mut a = contar("a b\nb");
        a.juntar(contar("b c"));
        assert_eq!((a.linhas, a.palavras), (3, 5));
        assert_eq!(a.frequencias["b"], 3);
        assert_eq!(a.frequencias["c"], 1);
    }

    #[test]
    fn juntar_nao_depende_da_ordem() {
        let (x, y) = (contar("a a b"), contar("b c\nd"));
        let mut xy = x.clone();
        xy.juntar(y.clone());
        let mut yx = y;
        yx.juntar(x);
        assert_eq!(xy, yx);
    }

    #[test]
    fn mais_frequentes_com_desempate_alfabetico() {
        let c = contar("c b a b c c d");
        assert_eq!(c.mais_frequentes(3), [("c", 3), ("b", 2), ("a", 1)]);
        assert_eq!(c.mais_frequentes(10).len(), 4);
    }
}

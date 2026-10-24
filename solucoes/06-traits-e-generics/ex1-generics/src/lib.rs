//! Exercício 1: faça compilar. Cada função ganhou o bound que o corpo usa.

use std::cmp::Ordering;
use std::fmt::Display;

/// O maior elemento, ou `None` se a lista estiver vazia.
pub fn maior<T: PartialOrd>(lista: &[T]) -> Option<&T> {
    let mut maior = lista.first()?;
    for item in lista {
        if item > maior {
            maior = item;
        }
    }
    Some(maior)
}

/// O menor dos dois (o primeiro, se forem iguais).
pub fn menor_de<T: Ord>(a: T, b: T) -> T {
    match a.cmp(&b) {
        Ordering::Greater => b,
        Ordering::Less | Ordering::Equal => a,
    }
}

/// Duas cópias de um valor emprestado.
pub fn duplicar<T: Clone>(x: &T) -> (T, T) {
    (x.clone(), x.clone())
}

/// `["a", "b"]` vira `["1. a", "2. b"]`.
pub fn numerar<T: Display>(itens: &[T]) -> Vec<String> {
    let mut saida = Vec::new();
    for (i, item) in itens.iter().enumerate() {
        saida.push(format!("{}. {item}", i + 1));
    }
    saida
}

/// Uma pilha genérica (LIFO).
#[derive(Debug)]
pub struct Pilha<T> {
    itens: Vec<T>,
}

// À mão, e não com `#[derive(Default)]`: o derive exigiria `T: Default`,
// e uma pilha vazia de qualquer `T` não precisa disso.
impl<T> Default for Pilha<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Pilha<T> {
    pub fn new() -> Self {
        Pilha { itens: Vec::new() }
    }

    pub fn empilhar(&mut self, item: T) {
        self.itens.push(item);
    }

    pub fn desempilhar(&mut self) -> Option<T> {
        self.itens.pop()
    }

    pub fn topo(&self) -> Option<&T> {
        self.itens.last()
    }

    pub fn len(&self) -> usize {
        self.itens.len()
    }

    pub fn is_empty(&self) -> bool {
        self.itens.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maior_funciona_com_varios_tipos() {
        assert_eq!(maior(&[3, 7, 2]), Some(&7));
        assert_eq!(maior(&['r', 'u', 's', 't']), Some(&'u'));
        assert_eq!(maior(&["pera", "uva", "maçã"]), Some(&"uva"));
        assert_eq!(maior(&[1.5, -2.0]), Some(&1.5));
        assert_eq!(maior::<i32>(&[]), None);
    }

    #[test]
    fn menor_de_dois() {
        assert_eq!(menor_de(3, 2), 2);
        assert_eq!(menor_de("b", "a"), "a");
        assert_eq!(menor_de(String::from("x"), String::from("x")), "x");
    }

    #[test]
    fn duplicar_clona() {
        let s = String::from("oi");
        assert_eq!(duplicar(&s), (String::from("oi"), String::from("oi")));
        assert_eq!(duplicar(&5), (5, 5));
    }

    #[test]
    fn numerar_usa_display() {
        assert_eq!(numerar(&["a", "b"]), ["1. a", "2. b"]);
        assert_eq!(numerar(&[10, 20]), ["1. 10", "2. 20"]);
    }

    #[test]
    fn pilha() {
        let mut p = Pilha::new();
        assert!(p.is_empty());
        p.empilhar("a");
        p.empilhar("b");
        assert_eq!(p.topo(), Some(&"b"));
        assert_eq!(p.desempilhar(), Some("b"));
        assert_eq!(p.len(), 1);
        // `File` não implementa `Default`, e a pilha vazia funciona mesmo assim.
        let vazia: Pilha<std::fs::File> = Pilha::default();
        assert_eq!(vazia.len(), 0);
    }
}

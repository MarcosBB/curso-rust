//! Exercício 2: um ponteiro inteligente que conta os descartes.

use std::cell::Cell;
use std::ops::{Deref, DerefMut};

/// Envolve um valor e soma 1 em `drops` quando é descartado.
pub struct Rastreado<'a, T> {
    valor: T,
    drops: &'a Cell<u32>,
}

impl<'a, T> Rastreado<'a, T> {
    pub fn new(valor: T, drops: &'a Cell<u32>) -> Self {
        Rastreado { valor, drops }
    }
}

impl<T> Deref for Rastreado<'_, T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.valor
    }
}

impl<T> DerefMut for Rastreado<'_, T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.valor
    }
}

impl<T> Drop for Rastreado<'_, T> {
    fn drop(&mut self) {
        self.drops.update(|n| n + 1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tamanho(s: &str) -> usize {
        s.len()
    }

    #[test]
    fn deref_e_coercao() {
        let drops = Cell::new(0);
        let r = Rastreado::new(String::from("oi"), &drops);
        assert_eq!(*r, "oi");
        assert_eq!(tamanho(&r), 2); // &Rastreado<String> -> &String -> &str
        assert!(r.starts_with('o'));
    }

    #[test]
    fn deref_mut() {
        let drops = Cell::new(0);
        let mut v = Rastreado::new(vec![1, 2], &drops);
        v.push(3);
        v[0] = 10;
        assert_eq!(*v, [10, 2, 3]);
    }

    #[test]
    fn conta_os_drops() {
        let drops = Cell::new(0);
        {
            let _a = Rastreado::new(1, &drops);
            let b = Rastreado::new(2, &drops);
            drop(b);
            assert_eq!(drops.get(), 1);
        }
        assert_eq!(drops.get(), 2);
    }

    #[test]
    fn mover_nao_descarta() {
        let drops = Cell::new(0);
        let lista: Vec<_> = (0..3).map(|i| Rastreado::new(i, &drops)).collect();
        let soma: i32 = lista.iter().map(|r| **r).sum();
        assert_eq!(soma, 3);
        assert_eq!(drops.get(), 0);
        drop(lista);
        assert_eq!(drops.get(), 3);
    }
}

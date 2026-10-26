//! Exercício 3: dois pânicos de `RefCell`, consertados.

use std::cell::RefCell;
use std::collections::HashMap;

#[derive(Default)]
pub struct Banco {
    contas: RefCell<HashMap<String, i64>>,
}

impl Banco {
    pub fn depositar(&self, nome: &str, valor: i64) {
        *self
            .contas
            .borrow_mut()
            .entry(nome.to_string())
            .or_insert(0) += valor;
    }

    pub fn saldo(&self, nome: &str) -> Option<i64> {
        self.contas.borrow().get(nome).copied()
    }

    /// Credita `taxa` por cento em todas as contas com saldo positivo.
    pub fn aplicar_juros(&self, taxa: i64) {
        // Calcula tudo com o empréstimo de leitura e o solta antes de depositar.
        let juros: Vec<(String, i64)> = self
            .contas
            .borrow()
            .iter()
            .filter(|&(_, &saldo)| saldo > 0)
            .map(|(nome, saldo)| (nome.clone(), saldo * taxa / 100))
            .collect();
        for (nome, valor) in juros {
            self.depositar(&nome, valor);
        }
    }

    /// Devolve o saldo, abrindo a conta com 0 se ela não existir.
    pub fn saldo_ou_abrir(&self, nome: &str) -> i64 {
        // `saldo` encerra o empréstimo antes do `match`.
        match self.saldo(nome) {
            Some(s) => s,
            None => {
                self.depositar(nome, 0);
                0
            }
        }
    }

    /// Este já funcionava na edition 2024: o `Ref` do `if let` é descartado
    /// antes do `else`.
    pub fn primeiro_deposito(&self, nome: &str, valor: i64) -> bool {
        if let Some(_saldo) = self.contas.borrow().get(nome) {
            false
        } else {
            self.depositar(nome, valor);
            true
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn juros() {
        let b = Banco::default();
        b.depositar("ana", 1000);
        b.depositar("bia", -50);
        b.aplicar_juros(10);
        assert_eq!(b.saldo("ana"), Some(1100));
        assert_eq!(b.saldo("bia"), Some(-50));
    }

    #[test]
    fn abre_conta() {
        let b = Banco::default();
        assert_eq!(b.saldo_ou_abrir("caio"), 0);
        assert_eq!(b.saldo("caio"), Some(0));
        b.depositar("caio", 5);
        assert_eq!(b.saldo_ou_abrir("caio"), 5);
    }

    #[test]
    fn primeiro() {
        let b = Banco::default();
        assert!(b.primeiro_deposito("ana", 10));
        assert!(!b.primeiro_deposito("ana", 99));
        assert_eq!(b.saldo("ana"), Some(10));
    }
}

//! Um ponteiro inteligente feito à mão (`Deref`, `DerefMut`, `Drop`)
//! e uma guarda RAII de transação. Lição 02.

use std::ops::{Deref, DerefMut};

/// Um "Box" de brinquedo: guarda o valor dentro de si (na stack, ao
/// contrário da `Box` de verdade) e avisa quando é descartado.
struct MeuBox<T: std::fmt::Debug>(T);

impl<T: std::fmt::Debug> MeuBox<T> {
    fn new(x: T) -> MeuBox<T> {
        MeuBox(x)
    }
}

impl<T: std::fmt::Debug> Deref for MeuBox<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.0
    }
}

impl<T: std::fmt::Debug> DerefMut for MeuBox<T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.0
    }
}

impl<T: std::fmt::Debug> Drop for MeuBox<T> {
    fn drop(&mut self) {
        println!("  drop MeuBox({:?})", self.0);
    }
}

fn saudar(nome: &str) {
    println!("olá, {nome}");
}

/// Guarda de transação: se ninguém chamar `confirmar`, o `drop` desfaz.
struct Transacao<'a> {
    log: &'a mut Vec<String>,
    confirmada: bool,
}

impl<'a> Transacao<'a> {
    fn iniciar(log: &'a mut Vec<String>) -> Transacao<'a> {
        log.push(String::from("begin"));
        Transacao {
            log,
            confirmada: false,
        }
    }

    fn executar(&mut self, comando: &str) {
        self.log.push(comando.to_string());
    }

    /// Consome a transação: depois de confirmar, não dá mais para usá-la.
    fn confirmar(mut self) {
        self.log.push(String::from("commit"));
        self.confirmada = true;
    } // `self` é descartado aqui, e o `drop` vê `confirmada == true`
}

impl Drop for Transacao<'_> {
    fn drop(&mut self) {
        if !self.confirmada {
            self.log.push(String::from("rollback"));
        }
    }
}

fn transferir(log: &mut Vec<String>, valor: u32) -> Result<(), String> {
    let mut t = Transacao::iniciar(log);
    t.executar(&format!("debita {valor}"));
    if valor > 100 {
        return Err(format!("limite excedido: {valor}")); // `t` cai aqui: rollback
    }
    t.executar(&format!("credita {valor}"));
    t.confirmar();
    Ok(())
}

fn main() {
    let x = MeuBox::new(5);
    println!("*x + 1 = {}", *x + 1); // *x é *Deref::deref(&x)

    let mut nome = MeuBox::new(String::from("Ana"));
    saudar(&nome); // &MeuBox<String> -> &String -> &str
    nome.push_str(" Maria"); // DerefMut: chama String::push_str
    println!("tamanho: {}", nome.len());

    println!("fim do main:");
    let mut log = Vec::new();
    let _ = transferir(&mut log, 50);
    let r = transferir(&mut log, 500);
    println!("{r:?}");
    println!("{log:?}");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rollback_quando_nao_confirma() {
        let mut log = Vec::new();
        assert!(transferir(&mut log, 500).is_err());
        assert_eq!(log, ["begin", "debita 500", "rollback"]);
    }

    #[test]
    fn commit_sem_rollback() {
        let mut log = Vec::new();
        transferir(&mut log, 10).unwrap();
        assert_eq!(log, ["begin", "debita 10", "credita 10", "commit"]);
    }
}

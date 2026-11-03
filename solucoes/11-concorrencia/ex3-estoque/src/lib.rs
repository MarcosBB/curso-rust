use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Um estoque que pode ser clonado e usado por várias threads.
/// Os clones compartilham os mesmos itens.
#[derive(Clone, Default)]
pub struct Estoque {
    itens: Arc<Mutex<HashMap<String, u32>>>,
}

impl Estoque {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn repor(&self, item: &str, qtd: u32) {
        *self
            .itens
            .lock()
            .unwrap()
            .entry(item.to_string())
            .or_insert(0) += qtd;
    }

    /// Tira `qtd` unidades de `item`, se houver. Um item desconhecido é
    /// cadastrado com 0 unidades. Devolve `true` se a reserva foi feita.
    pub fn reservar(&self, item: &str, qtd: u32) -> bool {
        // Um guarda só, para a verificação e a alteração: ninguém mexe no
        // item entre as duas, e não há um segundo `lock()` dentro do `match`.
        let mut itens = self.itens.lock().unwrap();
        match itens.get_mut(item) {
            Some(n) if *n >= qtd => {
                *n -= qtd;
                true
            }
            Some(_) => false,
            None => {
                itens.insert(item.to_string(), 0);
                false
            }
        }
    }

    pub fn quantidade(&self, item: &str) -> Option<u32> {
        self.itens.lock().unwrap().get(item).copied()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn reserva_simples() {
        let e = Estoque::new();
        e.repor("café", 3);
        assert!(e.reservar("café", 2));
        assert!(!e.reservar("café", 2));
        assert_eq!(e.quantidade("café"), Some(1));
    }

    #[test]
    fn item_desconhecido_e_cadastrado() {
        let e = Estoque::new();
        assert!(!e.reservar("chá", 1));
        assert_eq!(e.quantidade("chá"), Some(0));
    }

    #[test]
    fn varias_threads_nunca_vendem_demais() {
        let e = Estoque::new();
        e.repor("café", 500);
        let mut handles = Vec::new();
        for _ in 0..8 {
            let e = e.clone();
            handles.push(thread::spawn(move || {
                (0..100).filter(|_| e.reservar("café", 1)).count()
            }));
        }
        let vendidos: usize = handles.into_iter().map(|h| h.join().unwrap()).sum();
        assert_eq!(vendidos, 500);
        assert_eq!(e.quantidade("café"), Some(0));
    }
}

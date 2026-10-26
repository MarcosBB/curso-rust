//! Exercício 4: organograma com chefe `Weak` e equipe `Rc`.

use std::cell::RefCell;
use std::rc::{Rc, Weak};

pub struct Funcionario {
    pub nome: String,
    chefe: RefCell<Weak<Funcionario>>,
    equipe: RefCell<Vec<Rc<Funcionario>>>,
}

impl Funcionario {
    pub fn novo(nome: &str) -> Rc<Funcionario> {
        Rc::new(Funcionario {
            nome: nome.to_string(),
            chefe: RefCell::new(Weak::new()),
            equipe: RefCell::new(Vec::new()),
        })
    }

    /// Cria um funcionário na equipe de `chefe`.
    pub fn contratar(chefe: &Rc<Funcionario>, nome: &str) -> Rc<Funcionario> {
        let novo = Funcionario::novo(nome);
        *novo.chefe.borrow_mut() = Rc::downgrade(chefe);
        chefe.equipe.borrow_mut().push(Rc::clone(&novo));
        novo
    }

    /// O chefe direto, se ainda existir.
    pub fn chefe(&self) -> Option<Rc<Funcionario>> {
        self.chefe.borrow().upgrade()
    }

    /// Os nomes do próprio funcionário até o topo: `["dev", "gerente", "ceo"]`.
    pub fn cadeia_de_comando(&self) -> Vec<String> {
        let mut nomes = vec![self.nome.clone()];
        let mut atual = self.chefe();
        while let Some(f) = atual {
            nomes.push(f.nome.clone());
            atual = f.chefe();
        }
        nomes
    }

    /// Quantas pessoas estão abaixo, direta ou indiretamente.
    pub fn tamanho_da_equipe(&self) -> usize {
        self.equipe
            .borrow()
            .iter()
            .map(|f| 1 + f.tamanho_da_equipe())
            .sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cadeia() {
        let ceo = Funcionario::novo("ceo");
        let gerente = Funcionario::contratar(&ceo, "gerente");
        let dev = Funcionario::contratar(&gerente, "dev");
        assert_eq!(dev.cadeia_de_comando(), ["dev", "gerente", "ceo"]);
        assert_eq!(ceo.cadeia_de_comando(), ["ceo"]);
    }

    #[test]
    fn equipe() {
        let ceo = Funcionario::novo("ceo");
        let g1 = Funcionario::contratar(&ceo, "g1");
        let g2 = Funcionario::contratar(&ceo, "g2");
        Funcionario::contratar(&g1, "a");
        Funcionario::contratar(&g1, "b");
        Funcionario::contratar(&g2, "c");
        assert_eq!(ceo.tamanho_da_equipe(), 5);
        assert_eq!(g1.tamanho_da_equipe(), 2);
    }

    #[test]
    fn contadores() {
        let ceo = Funcionario::novo("ceo");
        let g = Funcionario::contratar(&ceo, "g");
        Funcionario::contratar(&g, "a");
        // `g`: a variável e a equipe do ceo; observado pelo chefe de `a`.
        assert_eq!(Rc::strong_count(&g), 2);
        assert_eq!(Rc::weak_count(&g), 1);
        // O ceo só tem a variável como dona.
        assert_eq!(Rc::strong_count(&ceo), 1);
    }

    #[test]
    fn sem_vazamento() {
        let ceo = Funcionario::novo("ceo");
        let dev = Funcionario::contratar(&ceo, "dev");
        let observador = Rc::downgrade(&ceo);
        drop(ceo);
        assert!(observador.upgrade().is_none(), "o ceo foi liberado");
        assert!(dev.chefe().is_none());
        assert_eq!(dev.cadeia_de_comando(), ["dev"]);
    }
}

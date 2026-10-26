//! Ciclos de `Rc` que vazam, e a quebra com `Weak`. Lição 05.

use std::cell::RefCell;
use std::rc::{Rc, Weak};

// ---------- Um ciclo que vaza ----------

struct No {
    nome: &'static str,
    proximo: RefCell<Option<Rc<No>>>,
}

impl Drop for No {
    fn drop(&mut self) {
        println!("  drop {}", self.nome);
    }
}

fn ciclo_que_vaza() {
    let a = Rc::new(No {
        nome: "a",
        proximo: RefCell::new(None),
    });
    let b = Rc::new(No {
        nome: "b",
        proximo: RefCell::new(Some(Rc::clone(&a))),
    });
    *a.proximo.borrow_mut() = Some(Rc::clone(&b)); // a -> b -> a
    println!(
        "a: {} donos, b: {} donos",
        Rc::strong_count(&a),
        Rc::strong_count(&b)
    );
} // `a` e `b` saem de escopo, mas cada nó ainda tem 1 dono: o outro

// ---------- O mesmo par, com a volta em Weak ----------

struct NoFraco {
    nome: &'static str,
    proximo: RefCell<Weak<NoFraco>>,
}

impl Drop for NoFraco {
    fn drop(&mut self) {
        println!("  drop {}", self.nome);
    }
}

fn ciclo_com_weak() {
    let a = Rc::new(NoFraco {
        nome: "a",
        proximo: RefCell::new(Weak::new()),
    });
    let b = Rc::new(NoFraco {
        nome: "b",
        proximo: RefCell::new(Rc::downgrade(&a)),
    });
    *a.proximo.borrow_mut() = Rc::downgrade(&b);
    println!(
        "a: {} forte, {} fraco; b: {} forte, {} fraco",
        Rc::strong_count(&a),
        Rc::weak_count(&a),
        Rc::strong_count(&b),
        Rc::weak_count(&b)
    );
    if let Some(prox) = a.proximo.borrow().upgrade() {
        println!("depois de a vem {}", prox.nome);
    }
}

// ---------- Uma árvore de pastas: filhas fortes, pai fraco ----------

struct Pasta {
    nome: String,
    pai: RefCell<Weak<Pasta>>,
    filhas: RefCell<Vec<Rc<Pasta>>>,
}

impl Pasta {
    fn nova(nome: &str) -> Rc<Pasta> {
        Rc::new(Pasta {
            nome: nome.to_string(),
            pai: RefCell::new(Weak::new()),
            filhas: RefCell::new(Vec::new()),
        })
    }

    /// Cria uma subpasta dentro de `pai` e a devolve.
    fn criar_em(pai: &Rc<Pasta>, nome: &str) -> Rc<Pasta> {
        let filha = Pasta::nova(nome);
        *filha.pai.borrow_mut() = Rc::downgrade(pai);
        pai.filhas.borrow_mut().push(Rc::clone(&filha));
        filha
    }

    /// O caminho completo, subindo pelos pais enquanto eles existirem.
    fn caminho(&self) -> String {
        match self.pai.borrow().upgrade() {
            Some(pai) => format!("{}/{}", pai.caminho(), self.nome),
            None => self.nome.clone(),
        }
    }
}

impl Drop for Pasta {
    fn drop(&mut self) {
        println!("  drop pasta {}", self.nome);
    }
}

fn arvore() {
    let fotos;
    {
        let raiz = Pasta::nova("home");
        let ana = Pasta::criar_em(&raiz, "ana");
        fotos = Pasta::criar_em(&ana, "fotos");
        println!("{}", fotos.caminho());
        println!(
            "ana: {} forte (raiz e a variável), {} fraco (fotos)",
            Rc::strong_count(&ana),
            Rc::weak_count(&ana)
        );
    } // `raiz` cai: home não tem mais dono forte, e leva ana junto
    println!("fora do bloco: {}", fotos.caminho());
    println!(
        "pai de fotos ainda existe? {}",
        fotos.pai.borrow().upgrade().is_some()
    );
}

fn main() {
    println!("== ciclo com Rc");
    ciclo_que_vaza();
    println!("fim do ciclo_que_vaza (nenhum drop acima)");

    println!("== ciclo com Weak");
    ciclo_com_weak();
    println!("fim do ciclo_com_weak");

    println!("== árvore");
    arvore();
    println!("fim");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn caminho_sobe_pelos_pais() {
        let raiz = Pasta::nova("r");
        let a = Pasta::criar_em(&raiz, "a");
        let b = Pasta::criar_em(&a, "b");
        assert_eq!(b.caminho(), "r/a/b");
        assert_eq!(Rc::strong_count(&a), 2);
        assert_eq!(Rc::weak_count(&a), 1);
    }

    #[test]
    fn filha_nao_mantem_o_pai_vivo() {
        let raiz = Pasta::nova("r");
        let a = Pasta::criar_em(&raiz, "a");
        drop(raiz);
        assert!(a.pai.borrow().upgrade().is_none());
        assert_eq!(a.caminho(), "a");
    }
}

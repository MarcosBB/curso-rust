//! `Rc` (lição 03) e `Cell`, `RefCell`, `OnceCell` (lição 04).

use std::cell::{Cell, OnceCell, RefCell};
use std::rc::Rc;

// ---------- Lição 03: Rc ----------

/// Uma configuração lida uma vez e compartilhada por vários componentes.
#[derive(Debug, Clone)]
struct Config {
    nome: String,
    nivel: u8,
}

struct Servidor {
    config: Rc<Config>,
}

struct Monitor {
    config: Rc<Config>,
}

fn rc_basico() {
    let config = Rc::new(Config {
        nome: String::from("prod"),
        nivel: 2,
    });
    println!("donos: {}", Rc::strong_count(&config));

    let servidor = Servidor {
        config: Rc::clone(&config),
    };
    {
        let monitor = Monitor {
            config: Rc::clone(&config),
        };
        println!("donos: {}", Rc::strong_count(&config));
        println!(
            "mesmo valor? {}",
            Rc::ptr_eq(&servidor.config, &monitor.config)
        );
    } // `monitor` sai de escopo: o contador desce
    println!("donos: {}", Rc::strong_count(&config));
    println!("nível do servidor: {}", servidor.config.nivel);
}

fn rc_make_mut() {
    let mut a = Rc::new(Config {
        nome: String::from("dev"),
        nivel: 1,
    });
    let b = Rc::clone(&a);
    // Há dois donos: make_mut clona o Config e `a` passa a apontar para a cópia.
    Rc::make_mut(&mut a).nivel = 9;
    println!("a: {} {}, b: {} {}", a.nome, a.nivel, b.nome, b.nivel);
    println!("donos de b: {}", Rc::strong_count(&b));

    // Com um dono só, try_unwrap devolve o valor de dentro.
    let sozinho = Rc::try_unwrap(b).expect("só há um dono");
    println!("de volta: {sozinho:?}");
}

// ---------- Lição 04: Cell, RefCell, OnceCell ----------

/// A trait do exercício 3 do Módulo 06, agora com `&self`.
trait Logger {
    fn registrar(&self, msg: &str);
}

#[derive(Default)]
struct MemoriaLogger {
    linhas: RefCell<Vec<String>>,
    chamadas: Cell<u32>,
}

impl Logger for MemoriaLogger {
    fn registrar(&self, msg: &str) {
        self.chamadas.set(self.chamadas.get() + 1);
        self.linhas.borrow_mut().push(msg.to_string());
    }
}

struct Pedido<'a> {
    id: u32,
    log: &'a dyn Logger,
}

impl Pedido<'_> {
    fn pagar(&self) {
        self.log.registrar(&format!("pedido {} pago", self.id));
    }
}

/// Um texto com a contagem de palavras calculada só na primeira vez.
struct Documento {
    texto: String,
    palavras: OnceCell<usize>,
}

impl Documento {
    fn new(texto: &str) -> Documento {
        Documento {
            texto: texto.to_string(),
            palavras: OnceCell::new(),
        }
    }

    fn palavras(&self) -> usize {
        *self.palavras.get_or_init(|| {
            println!("  (contando...)");
            self.texto.split_whitespace().count()
        })
    }
}

/// Estado compartilhado e mutável por dois donos: `Rc<RefCell<T>>`.
struct Carrinho {
    itens: Rc<RefCell<Vec<String>>>,
}

impl Carrinho {
    fn adicionar(&self, item: &str) {
        self.itens.borrow_mut().push(item.to_string());
    }
}

fn mutabilidade_interior() {
    let log = MemoriaLogger::default();
    let p1 = Pedido { id: 1, log: &log };
    let p2 = Pedido { id: 2, log: &log };
    p1.pagar();
    p2.pagar();
    println!("{} chamadas: {:?}", log.chamadas.get(), log.linhas.borrow());

    let doc = Documento::new("um dois três quatro");
    println!("palavras: {}", doc.palavras());
    println!("palavras: {}", doc.palavras());

    let lista = Rc::new(RefCell::new(Vec::new()));
    let celular = Carrinho {
        itens: Rc::clone(&lista),
    };
    let site = Carrinho {
        itens: Rc::clone(&lista),
    };
    celular.adicionar("café");
    site.adicionar("pão");
    println!("carrinho: {:?}", lista.borrow());

    // try_borrow_mut: o erro vira Result, em vez de pânico.
    let leitura = lista.borrow();
    match lista.try_borrow_mut() {
        Ok(_) => println!("consegui escrever"),
        Err(e) => println!("não deu: {e}"),
    }
    drop(leitura);
    lista.borrow_mut().clear();
    println!("vazio? {}", lista.borrow().is_empty());
}

fn main() {
    println!("== Rc");
    rc_basico();
    rc_make_mut();
    println!("== Cell, RefCell, OnceCell");
    mutabilidade_interior();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn logger_com_self_compartilhado() {
        let log = MemoriaLogger::default();
        Pedido { id: 7, log: &log }.pagar();
        assert_eq!(log.chamadas.get(), 1);
        assert_eq!(*log.linhas.borrow(), ["pedido 7 pago"]);
    }

    #[test]
    fn carrinhos_veem_a_mesma_lista() {
        let lista = Rc::new(RefCell::new(Vec::new()));
        let a = Carrinho {
            itens: Rc::clone(&lista),
        };
        a.adicionar("x");
        assert_eq!(lista.borrow().len(), 1);
        assert_eq!(Rc::strong_count(&lista), 2);
    }

    #[test]
    #[should_panic(expected = "already borrowed")]
    fn dois_emprestimos_dao_panic() {
        let c = RefCell::new(0);
        let _leitura = c.borrow();
        *c.borrow_mut() += 1;
    }
}

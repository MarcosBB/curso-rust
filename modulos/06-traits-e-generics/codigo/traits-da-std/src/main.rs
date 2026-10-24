//! Exemplos da lição 04: as traits da std e o `derive`.

use std::collections::HashSet;
use std::hash::{Hash, Hasher};

/// Tudo derivado: compara campo a campo, na ordem de declaração.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct Versao(u32, u32, u32);

/// Em enums, a ordem derivada é a ordem das variantes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Prioridade {
    Baixa,
    Media,
    Alta,
}

/// Igualdade à mão: dois usuários são o mesmo se o `id` for o mesmo.
/// Por isso o `Hash` também é à mão, e usa só o `id`.
#[derive(Debug, Clone)]
struct Usuario {
    id: u32,
    nome: String,
}

impl PartialEq for Usuario {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for Usuario {}

impl Hash for Usuario {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

/// `Default` derivado: cada campo recebe o padrão do seu tipo.
#[derive(Debug, Default)]
struct Opcoes {
    verboso: bool,
    tentativas: u32,
    modo: Modo,
    nome: String,
}

#[derive(Debug, Default, PartialEq)]
enum Modo {
    Rapido,
    #[default]
    Seguro,
}

/// Um tipo com `new()` sem argumentos também deve ter `Default`
/// (o clippy avisa com `new_without_default`).
#[derive(Debug)]
struct Contador {
    valor: u32,
}

impl Contador {
    fn new() -> Self {
        Self { valor: 0 }
    }
}

impl Default for Contador {
    fn default() -> Self {
        Self::new()
    }
}

fn main() {
    let mut versoes = vec![Versao(1, 10, 0), Versao(1, 2, 3), Versao(0, 9, 9)];
    versoes.sort();
    println!("{versoes:?}");
    println!("{}", Versao(1, 2, 3) < Versao(1, 10, 0));

    println!("{}", Prioridade::Alta > Prioridade::Baixa);
    println!("{:?}", Prioridade::Media.max(Prioridade::Baixa));

    let mut vistos = HashSet::new();
    vistos.insert(Usuario {
        id: 1,
        nome: String::from("Ana"),
    });
    let novo = vistos.insert(Usuario {
        id: 1,
        nome: String::from("Ana Maria"),
    });
    println!("inseriu de novo? {novo}; total: {}", vistos.len());
    for u in &vistos {
        println!("{} ({})", u.nome, u.id);
    }

    let o = Opcoes {
        tentativas: 3,
        ..Default::default()
    };
    println!("{o:?}");
    if o.verboso || o.modo == Modo::Rapido || !o.nome.is_empty() {
        println!("nunca acontece com os padrões");
    }
    println!("tentativas: {}", o.tentativas);
    println!("{:?}", Contador::default().valor);

    // `f64` só tem ordem parcial: NaN não é menor, maior nem igual a nada.
    let nan: f64 = "NaN".parse().unwrap();
    let outro = nan;
    println!("{:?}", 1.0_f64.partial_cmp(&nan));
    println!("{}", nan == outro);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ordem_dos_campos() {
        assert!(Versao(2, 0, 0) > Versao(1, 99, 99));
    }

    #[test]
    fn mesmo_id_mesmo_usuario() {
        let a = Usuario {
            id: 7,
            nome: String::from("a"),
        };
        let b = Usuario {
            id: 7,
            nome: String::from("b"),
        };
        assert_eq!(a, b);
        let mut conjunto = HashSet::new();
        conjunto.insert(a);
        assert!(conjunto.contains(&b));
    }

    #[test]
    fn default_do_enum() {
        assert_eq!(Modo::default(), Modo::Seguro);
        assert_ne!(Modo::Rapido, Modo::Seguro);
    }
}

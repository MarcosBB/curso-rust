//! Exemplos da lição 02: definir e implementar traits.

use std::fmt;

/// Algo que sabe se resumir em uma linha.
trait Resumo {
    /// Obrigatório: cada tipo diz quem é o autor.
    fn autor(&self) -> String;

    /// Método padrão: quem implementa pode aceitar este ou escrever o seu.
    fn resumir(&self) -> String {
        format!("(Leia mais de {}...)", self.autor())
    }
}

struct Post {
    titulo: String,
    autor: String,
}

struct Tweet {
    usuario: String,
}

impl Resumo for Post {
    fn autor(&self) -> String {
        self.autor.clone()
    }

    // Substitui o método padrão.
    fn resumir(&self) -> String {
        format!("{}, por {}", self.titulo, self.autor)
    }
}

impl Resumo for Tweet {
    fn autor(&self) -> String {
        format!("@{}", self.usuario)
    }
    // `resumir` vem do método padrão.
}

/// `Display` escrito à mão.
struct Ponto {
    x: i32,
    y: i32,
}

impl fmt::Display for Ponto {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}

/// Um `Display` que respeita largura e alinhamento (`{:>6}`), graças a `f.pad`.
struct Sigla(&'static str);

impl fmt::Display for Sigla {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.pad(self.0)
    }
}

/// Newtype: um tipo local que embrulha um tipo de fora, para poder
/// implementar nele uma trait de fora (orphan rule).
struct Lista(Vec<String>);

impl fmt::Display for Lista {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}]", self.0.join(", "))
    }
}

mod saida {
    /// Uma trait nossa implementada para um tipo da std (`str`).
    pub trait Gritar {
        fn gritar(&self) -> String;
    }

    impl Gritar for str {
        fn gritar(&self) -> String {
            format!("{}!", self.to_uppercase())
        }
    }
}

// Sem este `use`, `"oi".gritar()` não compila (E0599).
use saida::Gritar;

fn main() {
    let post = Post {
        titulo: String::from("Traits em Rust"),
        autor: String::from("Ana"),
    };
    let tweet = Tweet {
        usuario: String::from("rustlang"),
    };
    println!("{}", post.resumir());
    println!("{}", tweet.resumir());

    let p = Ponto { x: 3, y: -1 };
    println!("p = {p}");
    println!("[{p:>12}]"); // `write!` ignora a largura
    println!("[{:>6}]", Sigla("BR")); // `f.pad` respeita

    let nomes = Lista(vec![String::from("a"), String::from("b")]);
    println!("{nomes}");

    println!("{}", "cuidado".gritar());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_da_to_string_de_graca() {
        let p = Ponto { x: 1, y: 2 };
        assert_eq!(p.to_string(), "(1, 2)");
    }

    #[test]
    fn metodo_padrao_usa_o_obrigatorio() {
        let t = Tweet {
            usuario: String::from("ana"),
        };
        assert_eq!(t.resumir(), "(Leia mais de @ana...)");
    }

    #[test]
    fn pad_respeita_largura() {
        assert_eq!(format!("{:<4}|", Sigla("BR")), "BR  |");
    }
}

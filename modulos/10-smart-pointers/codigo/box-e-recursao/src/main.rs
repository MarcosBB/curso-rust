//! `Box<T>`: tamanhos e um tipo recursivo (árvore de expressões). Lição 01.

use std::fmt;

/// Uma expressão aritmética. Cada variante com filhos guarda os filhos
/// numa `Box`: sem ela, `Expr` conteria `Expr`, e o tamanho seria infinito
/// (E0072).
#[derive(Debug)]
enum Expr {
    Num(f64),
    Soma(Box<Expr>, Box<Expr>),
    Mult(Box<Expr>, Box<Expr>),
    Neg(Box<Expr>),
}

use Expr::{Mult, Neg, Num, Soma};

impl Expr {
    fn avaliar(&self) -> f64 {
        match self {
            Num(x) => *x,
            Soma(a, b) => a.avaliar() + b.avaliar(),
            Mult(a, b) => a.avaliar() * b.avaliar(),
            Neg(a) => -a.avaliar(),
        }
    }

    /// Troca `Neg(Neg(e))` por `e`, consumindo a árvore.
    /// Mostra que `*caixa` move o valor para fora da `Box`.
    fn simplificar(self) -> Expr {
        match self {
            Neg(dentro) => match *dentro {
                Neg(e) => e.simplificar(),
                outro => Neg(Box::new(outro.simplificar())),
            },
            Soma(a, b) => Soma(Box::new(a.simplificar()), Box::new(b.simplificar())),
            Mult(a, b) => Mult(Box::new(a.simplificar()), Box::new(b.simplificar())),
            Num(x) => Num(x),
        }
    }
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Num(x) => write!(f, "{x}"),
            Soma(a, b) => write!(f, "({a} + {b})"),
            Mult(a, b) => write!(f, "({a} * {b})"),
            Neg(a) => write!(f, "-{a}"),
        }
    }
}

/// Atalhos para montar árvores sem escrever `Box::new` toda hora.
fn soma(a: Expr, b: Expr) -> Expr {
    Soma(Box::new(a), Box::new(b))
}
fn mult(a: Expr, b: Expr) -> Expr {
    Mult(Box::new(a), Box::new(b))
}
fn neg(a: Expr) -> Expr {
    Neg(Box::new(a))
}

fn tamanhos() {
    println!("i32:              {:>2} bytes", size_of::<i32>());
    println!("Box<i32>:         {:>2} bytes", size_of::<Box<i32>>());
    println!(
        "Option<Box<i32>>: {:>2} bytes",
        size_of::<Option<Box<i32>>>()
    );
    println!("Box<[i32]>:       {:>2} bytes", size_of::<Box<[i32]>>());
    println!("Box<str>:         {:>2} bytes", size_of::<Box<str>>());
    println!("Vec<i32>:         {:>2} bytes", size_of::<Vec<i32>>());
    println!("Expr:             {:>2} bytes", size_of::<Expr>());
}

fn main() {
    tamanhos();

    // 2 * (3 + -(-4))
    let e = mult(Num(2.0), soma(Num(3.0), neg(neg(Num(4.0)))));
    println!("{e} = {}", e.avaliar());
    let s = e.simplificar();
    println!("{s} = {}", s.avaliar());

    // `*b` move o valor para fora (só a Box permite isso).
    let b = Box::new(String::from("dentro da caixa"));
    let s: String = *b;
    println!("{s}");

    // Box<[T]> e Box<str>: sem a capacidade extra do Vec e da String.
    let fatia: Box<[i32]> = vec![1, 2, 3].into_boxed_slice();
    let texto: Box<str> = "fixo".into();
    println!("{fatia:?} {texto}");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn avalia() {
        let e = soma(Num(1.0), mult(Num(2.0), Num(3.0)));
        assert_eq!(e.avaliar(), 7.0);
        assert_eq!(e.to_string(), "(1 + (2 * 3))");
    }

    #[test]
    fn simplifica_dupla_negacao() {
        let e = neg(neg(neg(Num(5.0)))).simplificar();
        assert_eq!(e.to_string(), "-5");
    }
}

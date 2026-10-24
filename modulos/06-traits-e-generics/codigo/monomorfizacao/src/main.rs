//! Monomorfização x dispatch dinâmico (lições 01 e 06).
//!
//! `mostra` é genérica: o compilador gera uma cópia para cada tipo usado.
//! `mostra_dyn` recebe um trait object: existe uma cópia só, que consulta
//! a vtable em tempo de execução.
//!
//! Para ver os símbolos gerados (macOS/Linux):
//!
//! ```text
//! cargo build
//! nm -C target/debug/monomorfizacao | grep mostra
//! ```

use std::fmt::Display;

fn mostra<T: Display>(x: T) -> String {
    format!("<{x}>")
}

fn mostra_dyn(x: &dyn Display) -> String {
    format!("<{x}>")
}

fn main() {
    println!("{} {} {}", mostra(1), mostra("a"), mostra(2.5));
    println!(
        "{} {} {}",
        mostra_dyn(&1),
        mostra_dyn(&"a"),
        mostra_dyn(&2.5)
    );
}

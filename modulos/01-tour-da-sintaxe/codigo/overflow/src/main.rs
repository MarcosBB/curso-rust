//! Overflow de inteiros: o comportamento padrão e as quatro escolhas explícitas.
//!
//! Rode das duas formas e compare a última linha:
//!     cargo run              (perfil dev: panic)
//!     cargo run --release    (perfil release: dá a volta)

fn main() {
    // O `parse` esconde o valor do compilador. Com um literal (`250 + 10`),
    // o lint `arithmetic_overflow` barraria o build.
    let x: u8 = "250".parse().unwrap();

    println!("x = {x}, u8::MAX = {}", u8::MAX);
    println!("checked_add(10)     = {:?}", x.checked_add(10));
    println!("checked_add(5)      = {:?}", x.checked_add(5));
    println!("wrapping_add(10)    = {}", x.wrapping_add(10));
    println!("saturating_add(10)  = {}", x.saturating_add(10));
    println!("overflowing_add(10) = {:?}", x.overflowing_add(10));

    // O operador `+` comum: panic em debug, volta para 4 em release.
    println!("x + 10              = {}", x + 10);
}

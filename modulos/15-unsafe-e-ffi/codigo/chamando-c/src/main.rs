use chamando_c::{Ponto, aplicar, distancia, media, saudacao};
use std::ffi::{CStr, c_char};

// Funções da libc (e da libm), que o Rust já liga em todo programa.
unsafe extern "C" {
    // `sqrt` aceita qualquer `f64` e não mexe em memória: pode ser `safe`.
    safe fn sqrt(x: f64) -> f64;
    // `strlen` lê através do ponteiro: fica `unsafe` (o padrão).
    fn strlen(s: *const c_char) -> usize;
}

extern "C" fn dobrar(x: i32) -> i32 {
    x * 2
}

fn main() {
    println!("sqrt(2) = {}", sqrt(2.0));

    let texto: &CStr = c"unsafe"; // literal de string C, com '\0' no fim
    // SAFETY: `texto` é uma string C válida e terminada em '\0'.
    let n = unsafe { strlen(texto.as_ptr()) };
    println!(
        "strlen = {n} (bytes com o zero: {})",
        texto.count_bytes() + 1
    );

    println!("media = {}", media(&[7.0, 8.0, 9.5]));
    let origem = Ponto { x: 0.0, y: 0.0 };
    println!(
        "distancia = {}",
        distancia(origem, Ponto { x: 1.0, y: 1.0 })
    );

    let mut v = [1, 2, 3];
    aplicar(&mut v, dobrar);
    println!("aplicar(dobrar) = {v:?}");

    println!("{}", saudacao("Rust").unwrap());
    println!("{:?}", saudacao("com\0zero"));
}

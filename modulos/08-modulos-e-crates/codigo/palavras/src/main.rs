//! Executável principal: `cargo run -- texto...`

fn main() {
    let texto = std::env::args().skip(1).collect::<Vec<_>>().join(" ");
    println!("{} palavras", palavras::contar(&texto));
    if let Some(p) = palavras::mais_longa(&texto) {
        println!("mais longa: {p}");
    }
}

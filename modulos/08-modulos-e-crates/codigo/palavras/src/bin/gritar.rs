//! Segundo executável: `cargo run --bin gritar -- texto...`

fn main() {
    let texto = std::env::args().skip(1).collect::<Vec<_>>().join(" ");
    println!(
        "{} ({} palavras)",
        texto.to_uppercase(),
        palavras::contar(&texto)
    );
}

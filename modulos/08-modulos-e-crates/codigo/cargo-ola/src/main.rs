//! Subcomando de exemplo (lição 10). Instalado como `cargo-ola`, vira `cargo ola`.

fn main() {
    // Chamado como `cargo ola x y`, o Cargo executa `cargo-ola ola x y`:
    // o primeiro argumento depois do programa é o nome do subcomando.
    let args: Vec<String> = std::env::args().skip(1).collect();
    let resto = match args.first().map(String::as_str) {
        Some("ola") => &args[1..],
        _ => &args[..],
    };
    println!("olá do subcomando!");
    println!("argumentos: {args:?}");
    println!("seus argumentos: {resto:?}");
}

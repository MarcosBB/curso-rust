//! Exemplo de uso da biblioteca: `cargo run --example frase`

fn main() {
    let frase = "o rato roeu a roupa do rei de Roma";
    println!("{frase:?}");
    println!("palavras: {}", palavras::contar(frase));
    println!("mais longa: {:?}", palavras::mais_longa(frase));
}

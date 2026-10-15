//! Solução do exercício 1: três E0382, três correções diferentes.

// (b) A função só lê o texto, então recebe um empréstimo (`&str`)
// em vez de tomar posse da `String`.
fn contar_letras(texto: &str) -> usize {
    texto.chars().filter(|c| c.is_alphabetic()).count()
}

// (a) Reordenar: usamos `nome` ANTES de entregá-la ao vetor.
// O vetor precisa ser dono da String; não há motivo para copiar.
fn parte_a() -> Vec<String> {
    let mut nomes = Vec::new();
    let nome = String::from("Ferris");
    println!("adicionei {nome}");
    nomes.push(nome);
    nomes
}

// (b) Emprestar: `&frase` deixa `contar_letras` ler sem tomar posse.
fn parte_b() {
    let frase = String::from("Olá, mundo!");
    let n = contar_letras(&frase);
    println!("{frase:?} tem {n} letras");
}

// (c) Clonar: queremos duas listas independentes, então pagar por
// uma cópia de verdade é exatamente o que o enunciado pede.
fn parte_c() {
    let original = vec![String::from("a"), String::from("b")];
    let mut copia = original.clone();
    copia.push(String::from("c"));
    println!("original: {original:?}");
    println!("cópia:    {copia:?}");
}

fn main() {
    let nomes = parte_a();
    println!("{nomes:?}");
    parte_b();
    parte_c();
}

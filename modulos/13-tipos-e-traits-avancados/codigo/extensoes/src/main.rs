use extensoes::formato::{Csv, Formato, Json, nome_do_arquivo};
use extensoes::iter::IteratorExt;
use extensoes::texto::TextoExt;

fn main() {
    for frase in ["Ame a ema", "Rust", "A base do teto desaba"] {
        println!("{frase:?}: palíndromo? {}", frase.eh_palindromo());
    }
    println!("{}", "o rato roeu a roupa".capitalizar_palavras());

    let texto = "o rato roeu a roupa do rei de roma o rato";
    let contagem = texto.split_whitespace().contagem();
    println!("'rato' aparece {} vezes", contagem["rato"]);
    println!(
        "moda das letras de 'abracadabra': {:?}",
        "abracadabra".chars().moda()
    );

    let campos = [("nome", "Ana"), ("cidade", "Recife")];
    let formatos: [&dyn Formato; 2] = [&Json, &Csv];
    for f in formatos {
        println!("--- .{}\n{}", f.extensao(), f.serializar(&campos));
    }
    println!("{}", nome_do_arquivo("relatorio", &Json));
}

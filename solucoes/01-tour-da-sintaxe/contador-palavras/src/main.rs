use contador_palavras::{contar, em_ordem};

fn main() {
    let texto = "O rato roeu a roupa do rei de Roma. O rei, irritado, roeu o resto!";
    let contagem = contar(texto);
    for (palavra, quantidade) in em_ordem(&contagem) {
        println!("{palavra:>10} {quantidade}");
    }
}

//! Linha de comando da loja: usa a biblioteca `nucleo` do mesmo workspace.

use nucleo::Estoque;

fn main() {
    let mut estoque = Estoque::new();
    estoque.repor("café", 10);
    estoque.repor("pão", 4);

    for (produto, qtd) in [("café", 3), ("pão", 5), ("leite", 1)] {
        match estoque.vender(produto, qtd) {
            Ok(()) => println!("vendido: {qtd} x {produto}"),
            Err(e) => println!("erro: {e}"),
        }
    }
    print!("{}", estoque.relatorio());
}

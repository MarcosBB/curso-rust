//! `cargo run --example relatorio --features relatorio`

use vendas::Venda;

fn main() {
    let vendas = [
        Venda::nova("café", 1250),
        Venda::nova("pão", 375),
        Venda::nova("queijo", 2990),
    ];
    print!("{}", vendas::relatorio(&vendas));
}

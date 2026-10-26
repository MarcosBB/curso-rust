use carteira::moeda as moeda_v1; // a versão que a carteira usa (reexportada por ela)
use moeda::Real; // a versão 2, dependência direta do app

fn main() {
    let total = carteira::somar(moeda_v1::Real(1050), moeda_v1::Real(250));
    let total = Real::centavos(total.0); // conversão explícita entre as versões
    println!("total: {total}");
}

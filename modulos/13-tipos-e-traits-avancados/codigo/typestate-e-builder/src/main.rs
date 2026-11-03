use std::time::Duration;
use typestate_e_builder::ids::Id;
use typestate_e_builder::pedido::Pedido;
use typestate_e_builder::requisicao::Requisicao;

struct Usuario;
struct Produto;

fn main() {
    let u: Id<Usuario> = Id::new(7);
    let p: Id<Produto> = Id::new(7);
    println!("{u:?} e {p:?}: mesmo número, tipos diferentes");

    let mut pedido = Pedido::new();
    pedido.adicionar("livro", 4_990);
    pedido.adicionar("caneta", 350);
    println!("status: {}", pedido.status());

    let pago = match pedido.pagar() {
        Ok(pago) => pago,
        Err(carrinho) => {
            println!("carrinho vazio: {} itens", carrinho.itens().len());
            return;
        }
    };
    println!(
        "status: {}, total: {} centavos",
        pago.status(),
        pago.total_centavos()
    );

    let enviado = pago.enviar("BR123");
    println!(
        "status: {}, rastreio: {}",
        enviado.status(),
        enviado.rastreio()
    );

    let req = Requisicao::get()
        .url("https://exemplo.com/api/pedidos")
        .cabecalho("Accept", "application/json")
        .tempo_limite(Duration::from_secs(5))
        .build();
    println!("{req:?}");
}

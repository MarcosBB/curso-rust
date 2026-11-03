//! Testes de integração: aqui o derive roda como para qualquer usuário da crate.

use linha_csv::{LinhaCsv, para_csv};
use std::fmt::Display;

#[derive(LinhaCsv)]
struct Produto {
    #[csv(renomear = "código")]
    codigo: u32,
    nome: String,
    preco: f64,
    #[csv(pular)]
    _rascunho: bool,
}

#[derive(LinhaCsv)]
struct Par<T: Display> {
    chave: &'static str,
    valor: T,
}

#[test]
fn cabecalho_respeita_renomear_e_pular() {
    assert_eq!(Produto::cabecalho(), ["código", "nome", "preco"]);
}

#[test]
fn linha_converte_cada_campo() {
    let p = Produto {
        codigo: 7,
        nome: "Café, 500g".into(),
        preco: 18.5,
        _rascunho: true,
    };
    assert_eq!(p.linha(), ["7", "Café, 500g", "18.5"]);
    assert_eq!(para_csv(&[p]), "código,nome,preco\n7,\"Café, 500g\",18.5\n");
}

#[test]
fn funciona_com_genericos() {
    let p = Par {
        chave: "x",
        valor: 1.5,
    };
    assert_eq!(Par::<f64>::cabecalho(), ["chave", "valor"]);
    assert_eq!(p.linha(), ["x", "1.5"]);
}

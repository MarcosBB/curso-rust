//! Testes da API pública, como um usuário da biblioteca.

mod common;

use contador::{Contagem, contar, formatar};

#[test]
fn conta_o_poema() {
    assert_eq!(
        contar(common::POEMA),
        Contagem {
            linhas: 2,
            palavras: 8,
            bytes: 46,
        }
    );
}

#[test]
fn formata_como_o_wc() {
    assert_eq!(formatar(contar(common::POEMA)), "2 8 46");
}

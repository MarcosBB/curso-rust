//! Testes das macros de atributo e function-like, usadas como um usuário usaria.

use minhas_macros::{cor, cronometrar};

#[cronometrar]
fn dobro(inicio: u64) -> u64 {
    // Um parâmetro chamado `inicio` não colide com a variável da macro.
    inicio * 2
}

#[cronometrar]
fn classificar(n: i32) -> &'static str {
    if n < 0 {
        // O `return` sai da closure gerada pela macro, e o tempo é impresso.
        return "negativo";
    }
    if n == 0 {
        return "zero";
    }
    "positivo"
}

#[test]
fn cronometrar_preserva_o_resultado() {
    assert_eq!(dobro(21), 42);
    assert_eq!(classificar(-3), "negativo");
    assert_eq!(classificar(0), "zero");
    assert_eq!(classificar(8), "positivo");
}

#[test]
fn cor_vira_array_na_compilacao() {
    const BRANCO: [u8; 3] = cor!("#ffffff");
    assert_eq!(BRANCO, [255, 255, 255]);
    assert_eq!(cor!("#3b82f6"), [0x3b, 0x82, 0xf6]);
}

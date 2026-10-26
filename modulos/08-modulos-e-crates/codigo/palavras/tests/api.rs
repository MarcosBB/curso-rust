//! Teste de integração: usa só a API pública, como uma crate externa (Módulo 09).

#[test]
fn mais_longa_pega_a_primeira_no_empate() {
    assert_eq!(palavras::mais_longa("ab cd e"), Some("ab"));
    assert_eq!(palavras::mais_longa("   "), None);
}

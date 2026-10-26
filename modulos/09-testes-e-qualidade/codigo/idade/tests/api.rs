//! Teste de integração: usa a biblioteca como um usuário externo, só pela API pública.

#[test]
fn api_publica_funciona() {
    assert_eq!(idade::idade("18").unwrap(), 18);
    assert_eq!(idade::porcentagem(1, 4), 25);
}

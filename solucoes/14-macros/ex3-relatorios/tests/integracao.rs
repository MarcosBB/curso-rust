use relatorios::linha;

#[test]
fn linha_fora_da_crate() {
    assert_eq!(linha!("frete", 12.5), "frete: R$ 12.50");
}

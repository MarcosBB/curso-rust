mod common;

use common::rodar_com_stdin;

#[test]
fn traco_le_da_entrada_padrao() {
    let out = rodar_com_stdin(&["-"], "a\nb\n");
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(String::from_utf8_lossy(&out.stdout), "2\n");
}

#[test]
fn entrada_vazia_tem_zero_linhas() {
    let out = rodar_com_stdin(&["-"], "");
    assert_eq!(String::from_utf8_lossy(&out.stdout), "0\n");
}

//! Testes do executável: rodam o `contador` de verdade, como um usuário no terminal.

mod common;

use common::{POEMA, contador, rodar_com_stdin, stdout};
use std::fs;

#[test]
fn le_da_entrada_padrao() {
    let saida = rodar_com_stdin(&[], POEMA);
    assert_eq!(saida.status.code(), Some(0));
    assert_eq!(stdout(&saida), "2 8 46\n");
}

#[test]
fn le_de_um_arquivo() {
    // A pasta (e o arquivo dentro dela) somem quando `pasta` sai de escopo.
    let pasta = tempfile::tempdir().unwrap();
    let caminho = pasta.path().join("poema.txt");
    fs::write(&caminho, POEMA).unwrap();

    let saida = contador().arg(&caminho).output().unwrap();
    assert_eq!(saida.status.code(), Some(0));
    assert_eq!(stdout(&saida), "2 8 46\n");
}

#[test]
fn arquivo_inexistente_sai_com_1() {
    let saida = contador().arg("nao-existe.txt").output().unwrap();
    assert_eq!(saida.status.code(), Some(1));
    assert!(saida.stdout.is_empty());
    let erro = String::from_utf8_lossy(&saida.stderr);
    assert!(
        erro.starts_with("erro: não consegui ler nao-existe.txt"),
        "stderr foi: {erro}"
    );
}

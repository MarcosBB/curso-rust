//! Testes do executável: rodam o `contapal` de verdade numa pasta temporária.

use std::fs;
use std::process::{Command, Output};

fn contapal(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_contapal"))
        .args(args)
        .output()
        .unwrap()
}

fn texto(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}

fn pasta_de_exemplo() -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    fs::create_dir(dir.path().join("sub")).unwrap();
    fs::write(dir.path().join("a.txt"), "o rato roeu\na roupa do rei\n").unwrap();
    fs::write(dir.path().join("sub/b.txt"), "o rei de Roma\n").unwrap();
    dir
}

#[test]
fn os_tres_modos_imprimem_o_mesmo() {
    let dir = pasta_de_exemplo();
    let raiz = dir.path().to_str().unwrap();
    let esperado = "arquivos: 2 lidos, 0 com erro\n\
                    linhas:   3\n\
                    palavras: 11\n\
                    mais frequentes:\n       2  o\n       2  rei\n       1  a\n";
    for modo in ["seq", "canais", "rayon"] {
        let out = contapal(&["-m", modo, "-j", "2", "-n", "3", raiz]);
        assert_eq!(out.status.code(), Some(0), "modo {modo}");
        assert_eq!(texto(&out.stdout), esperado, "modo {modo}");
        assert!(out.stderr.is_empty());
    }
}

#[test]
fn arquivo_ruim_vira_aviso_e_codigo_1() {
    let dir = pasta_de_exemplo();
    let bin = dir.path().join("foto.jpg");
    fs::write(&bin, [0xff, 0xd8, 0xff, 0xe0]).unwrap();
    let out = contapal(&["-n", "0", dir.path().to_str().unwrap()]);
    assert_eq!(out.status.code(), Some(1));
    assert_eq!(
        texto(&out.stderr),
        format!(
            "contapal: {}: o arquivo não é UTF-8 válido\n",
            bin.display()
        )
    );
    assert!(texto(&out.stdout).starts_with("arquivos: 2 lidos, 1 com erro\n"));
}

#[test]
fn uso_errado_sai_com_2() {
    let out = contapal(&["-m", "turbo", "."]);
    assert_eq!(out.status.code(), Some(2));
    let erro = texto(&out.stderr);
    assert!(erro.starts_with("contapal: modo inválido: turbo"));
    assert!(erro.contains("uso: contapal"));
}

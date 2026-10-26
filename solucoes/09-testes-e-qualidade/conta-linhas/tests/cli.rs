mod common;

use common::conta_linhas;

#[test]
fn conta_as_linhas() {
    let pasta = tempfile::tempdir().unwrap();
    let path = pasta.path().join("tres.txt");
    std::fs::write(&path, "um\ndois\ntrês\n").unwrap();

    let out = conta_linhas().arg(&path).output().unwrap();
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(String::from_utf8_lossy(&out.stdout), "3\n");
    assert!(out.stderr.is_empty());
}

#[test]
fn arquivo_inexistente_sai_com_1() {
    let out = conta_linhas().arg("nao-existe.txt").output().unwrap();
    assert_eq!(out.status.code(), Some(1));
    assert!(out.stdout.is_empty());
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.starts_with("erro: não consegui ler nao-existe.txt: "),
        "stderr foi: {stderr}"
    );
}

#[test]
fn sem_argumento_sai_com_2() {
    let out = conta_linhas().output().unwrap();
    assert_eq!(out.status.code(), Some(2));
    assert_eq!(
        String::from_utf8_lossy(&out.stderr),
        "uso: conta-linhas <arquivo>\n"
    );
}

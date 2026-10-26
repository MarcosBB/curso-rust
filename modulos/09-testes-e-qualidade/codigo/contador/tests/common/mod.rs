//! Funções auxiliares compartilhadas pelos testes de integração.
//!
//! Cada arquivo de `tests/` compila este módulo de novo, e nem todo arquivo
//! usa todas as funções. Por isso o `allow` (e não `expect`): em `api.rs` o
//! lint dispara, em `cli.rs` não.
#![allow(dead_code, reason = "nem todo arquivo de teste usa todos os helpers")]

use std::io::Write;
use std::process::{Command, Output, Stdio};

/// Um texto pequeno, com acento, usado em vários testes.
pub const POEMA: &str = "Minha terra tem palmeiras\nonde canta o sabiá\n";

/// O executável `contador`, que o Cargo compilou antes dos testes.
pub fn contador() -> Command {
    Command::new(env!("CARGO_BIN_EXE_contador"))
}

/// Roda o `contador` com `args`, mandando `entrada` pela entrada padrão.
pub fn rodar_com_stdin(args: &[&str], entrada: &str) -> Output {
    let mut filho = contador()
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("o executável existe");
    filho
        .stdin
        .take()
        .expect("pedimos stdin com Stdio::piped()")
        .write_all(entrada.as_bytes())
        .expect("escrever na stdin do filho");
    filho.wait_with_output().expect("esperar o filho")
}

/// A saída padrão como texto.
pub fn stdout(saida: &Output) -> String {
    String::from_utf8_lossy(&saida.stdout).into_owned()
}

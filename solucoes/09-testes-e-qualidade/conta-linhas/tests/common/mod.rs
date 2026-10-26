//! Helpers compartilhados pelos testes de integração.
//!
//! `cli.rs` não usa `rodar_com_stdin`, então, para aquela crate, a função é
//! código morto. `expect` não serve: em `stdin.rs` ela é usada, e lá a
//! expectativa ficaria não cumprida.
#![allow(dead_code, reason = "nem todo arquivo de teste usa todos os helpers")]

use std::io::Write;
use std::process::{Command, Output, Stdio};

/// O executável que o Cargo compilou para os testes.
pub fn conta_linhas() -> Command {
    Command::new(env!("CARGO_BIN_EXE_conta-linhas"))
}

/// Roda o programa com `args`, mandando `entrada` pela entrada padrão.
pub fn rodar_com_stdin(args: &[&str], entrada: &str) -> Output {
    let mut filho = conta_linhas()
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    filho
        .stdin
        .take()
        .unwrap()
        .write_all(entrada.as_bytes())
        .unwrap();
    filho.wait_with_output().unwrap()
}

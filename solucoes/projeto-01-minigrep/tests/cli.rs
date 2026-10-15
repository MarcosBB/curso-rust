//! Testes de integração: rodam o executável de verdade, como um usuário.
//! O Cargo compila o binário antes e informa o caminho em
//! `CARGO_BIN_EXE_minigrep`. O Módulo 09 explica os detalhes.

use std::env;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::process::{self, Command, Output, Stdio};

fn minigrep() -> Command {
    Command::new(env!("CARGO_BIN_EXE_minigrep"))
}

/// Roda o minigrep com `args`, mandando `input` pela entrada padrão.
fn run_with_stdin(args: &[&str], input: &str) -> Output {
    let mut child = minigrep()
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(input.as_bytes())
        .unwrap();
    child.wait_with_output().unwrap()
}

/// Cria um arquivo temporário com um nome único para este teste.
fn temp_file(name: &str, contents: &str) -> PathBuf {
    let path = env::temp_dir().join(format!("minigrep-{}-{name}", process::id()));
    fs::write(&path, contents).unwrap();
    path
}

fn stdout(out: &Output) -> String {
    String::from_utf8_lossy(&out.stdout).to_string()
}

fn stderr(out: &Output) -> String {
    String::from_utf8_lossy(&out.stderr).to_string()
}

#[test]
fn reads_stdin_and_numbers_lines() {
    let out = run_with_stdin(&["-n", "to"], "Rust\nto be\nnot\nto go\n");
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(stdout(&out), "2:to be\n4:to go\n");
}

#[test]
fn no_match_is_exit_1() {
    let out = run_with_stdin(&["zzz"], "Rust\nto be\n");
    assert_eq!(out.status.code(), Some(1));
    assert_eq!(stdout(&out), "");
}

#[test]
fn ignore_case_env_var() {
    let mut child = minigrep()
        .arg("RUST")
        .env("IGNORE_CASE", "1")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(b"rust\nC++\nPython\n")
        .unwrap();
    let out = child.wait_with_output().unwrap();
    assert_eq!(stdout(&out), "rust\n");
}

#[test]
fn count_and_invert() {
    let out = run_with_stdin(&["-c", "-v", "o"], "Rust\nto be\nnot\nC\n");
    assert_eq!(stdout(&out), "2\n");
}

#[test]
fn missing_pattern_is_exit_2_with_usage() {
    let out = minigrep().output().unwrap();
    assert_eq!(out.status.code(), Some(2));
    assert!(stderr(&out).contains("faltou o padrão"));
    assert!(stderr(&out).contains("uso: minigrep"));
}

#[test]
fn unknown_flag_is_exit_2() {
    let out = minigrep().args(["-x", "foo"]).output().unwrap();
    assert_eq!(out.status.code(), Some(2));
    assert!(stderr(&out).contains("opção desconhecida: -x"));
}

#[test]
fn missing_file_is_exit_2() {
    let out = minigrep().args(["x", "/nao/existe.txt"]).output().unwrap();
    assert_eq!(out.status.code(), Some(2));
    assert!(stderr(&out).contains("/nao/existe.txt"));
}

#[test]
fn multiple_files_are_prefixed() {
    let a = temp_file("multi-a.txt", "um gato\num cão\n");
    let b = temp_file("multi-b.txt", "outro gato\n");
    let out = minigrep()
        .args(["gato", a.to_str().unwrap(), b.to_str().unwrap()])
        .output()
        .unwrap();
    fs::remove_file(&a).unwrap();
    fs::remove_file(&b).unwrap();

    assert_eq!(out.status.code(), Some(0));
    let expected = format!("{}:um gato\n{}:outro gato\n", a.display(), b.display());
    assert_eq!(stdout(&out), expected);
}

#[test]
fn continues_after_bad_file() {
    let good = temp_file("continua.txt", "linha boa\n");
    let out = minigrep()
        .args(["boa", "/nao/existe.txt", good.to_str().unwrap()])
        .output()
        .unwrap();
    fs::remove_file(&good).unwrap();

    // O arquivo bom foi processado, mas o código de saída avisa do erro.
    assert_eq!(out.status.code(), Some(2));
    assert_eq!(stdout(&out), format!("{}:linha boa\n", good.display()));
    assert!(stderr(&out).contains("/nao/existe.txt"));
}

#[test]
fn closed_pipe_is_not_a_panic() {
    let mut child = minigrep()
        .arg("a")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    // Fecha a leitura da saída antes de o minigrep escrever qualquer coisa,
    // como o `head` faz depois de ler as linhas de que precisa.
    drop(child.stdout.take());
    child.stdin.take().unwrap().write_all(b"a\na\na\n").unwrap();
    let out = child.wait_with_output().unwrap();

    assert_eq!(out.status.code(), Some(0));
    assert!(!stderr(&out).contains("panicked"), "{}", stderr(&out));
}

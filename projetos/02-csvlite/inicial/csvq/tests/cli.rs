//! Testes do executável `csvq`: rodam o programa de verdade.

use std::io::Write;
use std::process::{Command, Output, Stdio};

/// O `alunos.csv` da raiz do workspace.
const ALUNOS: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../alunos.csv");

/// Roda o `csvq` com os argumentos e o texto na entrada padrão.
fn csvq(args: &[&str], stdin: &str) -> Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_csvq"))
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
        .write_all(stdin.as_bytes())
        .unwrap();
    child.wait_with_output().unwrap()
}

fn stdout(out: &Output) -> &str {
    std::str::from_utf8(&out.stdout).unwrap()
}

fn stderr(out: &Output) -> &str {
    std::str::from_utf8(&out.stderr).unwrap()
}

#[test]
fn selects_columns_keeping_quotes() {
    let out = csvq(&["-d", ";", "-c", "nome,nota", ALUNOS], "");
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(
        stdout(&out),
        "nome;nota\nAna;9,5\nBruno;7\n\"Carla \"\"Cacá\"\" Souza\";8\n"
    );
}

#[test]
fn whole_file_round_trips() {
    let text = std::fs::read_to_string(ALUNOS).unwrap();
    let out = csvq(&["-d", ";"], &text);
    assert_eq!(stdout(&out), text);
}

#[test]
fn count_records_not_lines() {
    let out = csvq(&["-d", ";", "--count", ALUNOS], "");
    assert_eq!(stdout(&out), "3\n");
}

#[test]
fn reads_stdin_with_dash() {
    let out = csvq(&["-c", "b", "-"], "a,b\n1,2\n");
    assert_eq!(stdout(&out), "b\n2\n");
}

#[test]
fn parse_error_goes_to_stderr_with_code_2() {
    let out = csvq(&[], "a,b\n\"x\"y,2\n");
    assert_eq!(out.status.code(), Some(2));
    assert_eq!(stdout(&out), "");
    assert_eq!(
        stderr(&out),
        "csvq: linha 2, coluna 4: caractere 'y' depois de fechar aspas\n"
    );
}

#[test]
fn wrong_delimiter_gives_field_count_error() {
    let out = csvq(&["-c", "nome", ALUNOS], "");
    assert_eq!(out.status.code(), Some(2));
    assert_eq!(
        stderr(&out),
        "csvq: linha 2, coluna 1: esperava 1 campo, encontrei 2\n"
    );
}

#[test]
fn unknown_column() {
    let out = csvq(&["-c", "idade"], "nome,nota\nAna,9\n");
    assert_eq!(out.status.code(), Some(2));
    assert_eq!(stderr(&out), "csvq: coluna inexistente: idade\n");
}

#[test]
fn usage_error_shows_usage() {
    let out = csvq(&["-x"], "");
    assert_eq!(out.status.code(), Some(2));
    assert_eq!(
        stderr(&out),
        "csvq: opção desconhecida: -x\nuso: csvq [-d SEP] [-c COL,COL...] [--count] [ARQUIVO]\n"
    );
}

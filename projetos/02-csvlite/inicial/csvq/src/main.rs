//! `csvq`: seleciona colunas e conta registros de um arquivo CSV.
//!
//! ```text
//! uso: csvq [-d SEP] [-c COL,COL...] [--count] [ARQUIVO]
//! ```
//!
//! Sem `ARQUIVO`, ou com `-`, lê da entrada padrão. Toda a lógica de CSV
//! está na crate `csvlite`; aqui só se traduz argumentos e erros.

// Só no projeto inicial: enquanto as funções forem `todo!()`, imports e
// variantes aparecem como "não usados". Apague esta linha na etapa 7.
#![allow(unused, clippy::needless_pass_by_value)]

use std::env;
use std::fmt;
use std::fs;
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use csvlite::{Options, Table, write_record};

const USAGE: &str = "uso: csvq [-d SEP] [-c COL,COL...] [--count] [ARQUIVO]";

/// O que a linha de comando pediu.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Args {
    delimiter: char,
    /// `None`: todas as colunas.
    columns: Option<Vec<String>>,
    count: bool,
    /// `None`: entrada padrão.
    path: Option<PathBuf>,
}

/// Tudo o que pode dar errado. Todos saem com código 2.
#[derive(Debug)]
enum CliError {
    Usage(String),
    Read { path: PathBuf, source: io::Error },
    Parse(csvlite::Error),
    UnknownColumn(String),
    Write(io::Error),
}

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CliError::Usage(msg) => write!(f, "{msg}"),
            CliError::Read { path, source } => write!(f, "{}: {source}", path.display()),
            CliError::Parse(e) => write!(f, "{e}"),
            CliError::UnknownColumn(name) => write!(f, "coluna inexistente: {name}"),
            CliError::Write(e) => write!(f, "erro ao escrever a saída: {e}"),
        }
    }
}

/// Lê os argumentos (sem o nome do programa).
fn parse_args(args: impl IntoIterator<Item = String>) -> Result<Args, CliError> {
    // Etapa 7: `while let Some(arg) = args.next()` + `match arg.as_str()`,
    // como no Projeto 1. `-d` e `-c` consomem o argumento seguinte.
    todo!()
}

/// Lê o arquivo, ou a entrada padrão.
fn read_input(path: Option<&Path>) -> Result<String, CliError> {
    // Etapa 7: `None` é a entrada padrão.
    todo!()
}

/// Faz o trabalho: lê a tabela e escreve o resultado em `out`.
fn run(args: &Args, input: &str, out: &mut impl Write) -> Result<(), CliError> {
    // Etapa 7: `Table::parse`, os índices das colunas pedidas (na ordem
    // pedida) e `write_record` para o cabeçalho e cada registro.
    todo!()
}

fn main() -> ExitCode {
    // Etapa 7: `parse_args(env::args().skip(1))`, `read_input`, `run` com
    // `io::stdout().lock()`. Erro: "csvq: <erro>" no stderr (mais a linha de
    // uso, se for erro de uso) e código 2. Pipe fechado: código 0.
    todo!()
}

#[cfg(test)]
mod step7_tests {
    use super::*;
    use std::assert_matches;

    fn args(list: &[&str]) -> Result<Args, CliError> {
        parse_args(list.iter().map(ToString::to_string))
    }

    fn run_on(list: &[&str], input: &str) -> Result<String, CliError> {
        let mut out = Vec::new();
        run(&args(list)?, input, &mut out)?;
        Ok(String::from_utf8(out).unwrap())
    }

    #[test]
    fn defaults() {
        let a = args(&[]).unwrap();
        assert_eq!(a.delimiter, ',');
        assert_eq!(a.columns, None);
        assert!(!a.count);
        assert_eq!(a.path, None);
    }

    #[test]
    fn all_options() {
        let a = args(&["-d", ";", "-c", "nome,nota", "--count", "x.csv"]).unwrap();
        assert_eq!(a.delimiter, ';');
        assert_eq!(a.columns, Some(vec!["nome".into(), "nota".into()]));
        assert!(a.count);
        assert_eq!(a.path, Some(PathBuf::from("x.csv")));
    }

    #[test]
    fn dash_means_stdin() {
        assert_eq!(args(&["-"]).unwrap().path, None);
    }

    #[test]
    fn usage_errors() {
        assert_matches!(args(&["-d", ";;"]), Err(CliError::Usage(_)));
        assert_matches!(args(&["-d"]), Err(CliError::Usage(_)));
        assert_matches!(args(&["-c"]), Err(CliError::Usage(_)));
        assert_matches!(args(&["-x"]), Err(CliError::Usage(_)));
        assert_matches!(args(&["a.csv", "b.csv"]), Err(CliError::Usage(_)));
    }

    #[test]
    fn selects_and_reorders_columns() {
        let out = run_on(&["-c", "b,a"], "a,b,c\n1,2,3\n").unwrap();
        assert_eq!(out, "b,a\n2,1\n");
    }

    #[test]
    fn empty_input_writes_nothing() {
        assert_eq!(run_on(&[], "").unwrap(), "");
        assert_eq!(run_on(&["--count"], "").unwrap(), "0\n");
    }

    #[test]
    fn unknown_column() {
        assert_matches!(
            run_on(&["-c", "x"], "a\n1\n"),
            Err(CliError::UnknownColumn(name)) if name == "x"
        );
    }
}

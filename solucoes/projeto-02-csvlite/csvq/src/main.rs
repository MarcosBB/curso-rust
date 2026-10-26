//! `csvq`: seleciona colunas e conta registros de um arquivo CSV.
//!
//! ```text
//! uso: csvq [-d SEP] [-c COL,COL...] [--count] [ARQUIVO]
//! ```
//!
//! Sem `ARQUIVO`, ou com `-`, lê da entrada padrão. Toda a lógica de CSV
//! está na crate `csvlite`; aqui só se traduz argumentos e erros.

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
    let mut parsed = Args {
        delimiter: ',',
        columns: None,
        count: false,
        path: None,
    };
    let mut args = args.into_iter();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "-d" => {
                let value = args.next().unwrap_or_default();
                let mut chars = value.chars();
                parsed.delimiter = match (chars.next(), chars.next()) {
                    (Some(c), None) => c,
                    _ => {
                        return Err(CliError::Usage(format!(
                            "o separador precisa ser um caractere: {value:?}"
                        )));
                    }
                };
            }
            "-c" => {
                let Some(value) = args.next() else {
                    return Err(CliError::Usage("faltou a lista de colunas".into()));
                };
                parsed.columns = Some(value.split(',').map(String::from).collect());
            }
            "--count" => parsed.count = true,
            "-" => parsed.path = None,
            flag if flag.starts_with('-') => {
                return Err(CliError::Usage(format!("opção desconhecida: {flag}")));
            }
            _ if parsed.path.is_some() => {
                return Err(CliError::Usage("só um arquivo por vez".into()));
            }
            _ => parsed.path = Some(PathBuf::from(arg)),
        }
    }
    Ok(parsed)
}

/// Lê o arquivo, ou a entrada padrão.
fn read_input(path: Option<&Path>) -> Result<String, CliError> {
    let Some(path) = path else {
        let mut text = String::new();
        io::stdin()
            .read_to_string(&mut text)
            .map_err(|source| CliError::Read {
                path: PathBuf::from("(entrada padrão)"),
                source,
            })?;
        return Ok(text);
    };
    fs::read_to_string(path).map_err(|source| CliError::Read {
        path: path.to_path_buf(),
        source,
    })
}

/// Faz o trabalho: lê a tabela e escreve o resultado em `out`.
fn run(args: &Args, input: &str, out: &mut impl Write) -> Result<(), CliError> {
    let opts = Options {
        delimiter: args.delimiter,
        ..Options::default()
    };
    let table = Table::parse(input, opts).map_err(CliError::Parse)?;

    if args.count {
        return writeln!(out, "{}", table.rows().len()).map_err(CliError::Write);
    }
    if table.headers().is_empty() {
        return Ok(()); // entrada vazia: nada a escrever
    }

    // Índices das colunas pedidas, na ordem pedida.
    let indices: Vec<usize> = match &args.columns {
        None => (0..table.headers().len()).collect(),
        Some(names) => names
            .iter()
            .map(|name| {
                table
                    .headers()
                    .iter()
                    .position(|h| h == name)
                    .ok_or_else(|| CliError::UnknownColumn(name.clone()))
            })
            .collect::<Result<_, _>>()?,
    };

    let records = std::iter::once(table.headers()).chain(table.rows());
    for record in records {
        let fields: Vec<&str> = indices
            .iter()
            .map(|&i| record.get(i).unwrap_or_default())
            .collect();
        write_record(out, &fields, args.delimiter).map_err(CliError::Write)?;
    }
    Ok(())
}

fn main() -> ExitCode {
    let result = parse_args(env::args().skip(1)).and_then(|args| {
        let input = read_input(args.path.as_deref())?;
        run(&args, &input, &mut io::stdout().lock())
    });
    match result {
        Ok(()) => ExitCode::SUCCESS,
        // Pipe fechado (`csvq ... | head`) não é erro, como no Projeto 1.
        Err(CliError::Write(e)) if e.kind() == io::ErrorKind::BrokenPipe => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("csvq: {e}");
            if matches!(e, CliError::Usage(_)) {
                eprintln!("{USAGE}");
            }
            ExitCode::from(2)
        }
    }
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

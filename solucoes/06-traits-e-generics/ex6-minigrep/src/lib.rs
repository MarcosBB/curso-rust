//! minigrep: busca de texto em arquivos, no estilo do `grep`.
//!
//! Versão do Módulo 06: `run` saiu do `main.rs` e veio para cá, recebendo
//! o destino da saída como `&mut impl Write`. O `main` passa o stdout (ou
//! um arquivo, com `-o`), e os testes passam um `Vec<u8>`. A biblioteca
//! continua sem decidir **para onde** escrever.

pub mod config;
pub mod search;

use std::error::Error;
use std::fmt;
use std::fs;
use std::io::{self, Read, Write};
use std::path::PathBuf;

use config::{Config, Input, OutputOptions};
use search::Match;

/// Códigos de saída, na convenção do `grep`.
pub const FOUND: u8 = 0;
pub const NOT_FOUND: u8 = 1;
pub const ERROR: u8 = 2;

/// Erros ao ler uma entrada. Não são fatais: o programa avisa e segue para
/// a próxima entrada.
#[derive(Debug)]
pub enum RunError {
    /// Não deu para ler o arquivo (não existe, sem permissão, é diretório...).
    Read { path: PathBuf, source: io::Error },
    /// O arquivo existe, mas não é texto UTF-8.
    NotUtf8 { path: PathBuf },
    /// Falha ao ler a entrada padrão.
    Stdin(io::Error),
}

impl fmt::Display for RunError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            // A causa entra no texto de propósito. O `main` imprime só `{e}`,
            // sem percorrer `source()`, e o usuário espera ver o motivo:
            // "poema.txt: No such file or directory (os error 2)".
            // Quem percorrer a cadeia vai ver a causa duas vezes: é a troca
            // que fizemos (veja o M05, lição 04).
            RunError::Read { path, source } => write!(f, "{}: {source}", path.display()),
            RunError::NotUtf8 { path } => {
                write!(f, "{}: o arquivo não é UTF-8 válido", path.display())
            }
            RunError::Stdin(source) => write!(f, "entrada padrão: {source}"),
        }
    }
}

impl Error for RunError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            RunError::Read { source, .. } | RunError::Stdin(source) => Some(source),
            RunError::NotUtf8 { .. } => None,
        }
    }
}

/// Lê uma entrada inteira para a memória.
///
/// O `String` devolvido é do chamador: as `Match` de `search::search`
/// vão apontar para dentro dele.
pub fn read_input(input: &Input) -> Result<String, RunError> {
    match input {
        Input::Stdin => {
            let mut contents = String::new();
            io::stdin()
                .read_to_string(&mut contents)
                .map_err(RunError::Stdin)?;
            Ok(contents)
        }
        Input::File(path) => fs::read_to_string(path).map_err(|source| {
            // `read_to_string` avisa "não é UTF-8" com `InvalidData`.
            if source.kind() == io::ErrorKind::InvalidData {
                RunError::NotUtf8 { path: path.clone() }
            } else {
                RunError::Read {
                    path: path.clone(),
                    source,
                }
            }
        }),
    }
}

/// Formata o resultado de uma entrada, uma `String` por linha de saída.
///
/// `name` é o prefixo `arquivo:` (`run` passa `Some` quando há mais de
/// uma entrada). Pode ser qualquer `Display`: um `&Input`, um `&str`...
/// Com `count_only`, a saída é uma linha só, com a contagem.
pub fn render<N: fmt::Display>(
    matches: &[Match<'_>],
    name: Option<N>,
    opts: OutputOptions,
) -> Vec<String> {
    let prefix = match name {
        Some(name) => format!("{name}:"),
        None => String::new(),
    };

    if opts.count_only {
        return vec![format!("{prefix}{}", matches.len())];
    }

    let mut lines = Vec::new();
    for m in matches {
        if opts.line_numbers {
            lines.push(format!("{prefix}{}:{}", m.line_no, m.line));
        } else {
            lines.push(format!("{prefix}{}", m.line));
        }
    }
    lines
}

/// Processa todas as entradas, escreve o resultado em `out` e devolve o
/// código de saída.
///
/// Só a escrita em `out` é fatal (o `?` devolve o `io::Error`). Um arquivo
/// que não abre vira aviso no stderr, e o laço continua.
pub fn run(cfg: &Config, out: &mut impl Write) -> io::Result<u8> {
    let show_names = cfg.inputs.len() > 1;
    let mut found = false;
    let mut had_error = false;

    for input in &cfg.inputs {
        let contents = match read_input(input) {
            Ok(contents) => contents,
            Err(e) => {
                eprintln!("minigrep: {e}");
                had_error = true;
                continue;
            }
        };

        let matches = search::search(&cfg.pattern, &contents, cfg.search);
        found = found || !matches.is_empty();

        // `Option<&Input>`: sem `String` temporária, sem o E0716 da etapa 6.
        let name = if show_names { Some(input) } else { None };
        for line in render(&matches, name, cfg.output) {
            writeln!(out, "{line}")?;
        }
    }

    // Se a saída tiver buffer, um erro na última escrita só aparece no
    // flush. Sem ele, o erro se perderia em silêncio.
    out.flush()?;

    if had_error {
        Ok(ERROR)
    } else if found {
        Ok(FOUND)
    } else {
        Ok(NOT_FOUND)
    }
}

#[cfg(test)]
mod step4_tests {
    use super::*;
    use std::env;
    use std::process;

    #[test]
    fn missing_file_is_read_error() {
        let input = Input::File(PathBuf::from("/nao/existe.txt"));
        let Err(RunError::Read { path, source }) = read_input(&input) else {
            panic!("esperava RunError::Read");
        };
        assert_eq!(path, PathBuf::from("/nao/existe.txt"));
        assert_eq!(source.kind(), io::ErrorKind::NotFound);
    }

    #[test]
    fn reads_existing_file() {
        let path = env::temp_dir().join(format!("minigrep-lib-ok-{}.txt", process::id()));
        fs::write(&path, "um\ndois\n").unwrap();
        let contents = read_input(&Input::File(path.clone())).unwrap();
        fs::remove_file(&path).unwrap();
        assert_eq!(contents, "um\ndois\n");
    }

    #[test]
    fn invalid_utf8_is_reported() {
        let path = env::temp_dir().join(format!("minigrep-lib-bin-{}.txt", process::id()));
        fs::write(&path, [0xff, 0xfe, 0x00]).unwrap();
        let result = read_input(&Input::File(path.clone()));
        fs::remove_file(&path).unwrap();
        assert!(matches!(result, Err(RunError::NotUtf8 { .. })));
    }

    #[test]
    fn run_error_messages() {
        let not_utf8 = RunError::NotUtf8 {
            path: PathBuf::from("foto.png"),
        };
        assert_eq!(
            not_utf8.to_string(),
            "foto.png: o arquivo não é UTF-8 válido"
        );
        assert!(not_utf8.source().is_none());

        let read = RunError::Read {
            path: PathBuf::from("a.txt"),
            source: io::Error::from(io::ErrorKind::NotFound),
        };
        assert!(read.to_string().starts_with("a.txt: "));
        assert!(read.source().is_some());
    }
}

#[cfg(test)]
mod step5_tests {
    use super::*;

    fn sample() -> Vec<Match<'static>> {
        vec![
            Match {
                line_no: 2,
                line: "to be",
            },
            Match {
                line_no: 4,
                line: "to go",
            },
        ]
    }

    #[test]
    fn render_plain() {
        assert_eq!(
            render(&sample(), None::<&str>, OutputOptions::default()),
            ["to be", "to go"]
        );
    }

    #[test]
    fn render_with_name_and_line_numbers() {
        let opts = OutputOptions {
            line_numbers: true,
            ..Default::default()
        };
        assert_eq!(
            render(&sample(), Some("a.txt"), opts),
            ["a.txt:2:to be", "a.txt:4:to go"]
        );
    }

    #[test]
    fn render_count() {
        let opts = OutputOptions {
            count_only: true,
            ..Default::default()
        };
        assert_eq!(render(&sample(), None::<&str>, opts), ["2"]);
        assert_eq!(render(&[], Some("b.txt"), opts), ["b.txt:0"]);
    }
}

/// Testes do Módulo 06: `run` testado sem rodar o executável.
#[cfg(test)]
mod run_tests {
    use super::*;
    use crate::search::SearchOptions;
    use std::env;
    use std::process;

    fn temp_file(name: &str, contents: &str) -> PathBuf {
        let path = env::temp_dir().join(format!("minigrep-run-{}-{name}", process::id()));
        fs::write(&path, contents).unwrap();
        path
    }

    fn config(pattern: &str, inputs: Vec<Input>) -> Config {
        Config {
            pattern: pattern.to_string(),
            inputs,
            search: SearchOptions::default(),
            output: OutputOptions::default(),
            output_file: None,
        }
    }

    /// Roda e devolve (código, saída como texto).
    fn run_to_string(cfg: &Config) -> (u8, String) {
        let mut out: Vec<u8> = Vec::new();
        let code = run(cfg, &mut out).unwrap();
        (code, String::from_utf8(out).unwrap())
    }

    #[test]
    fn found_one_file() {
        let a = temp_file("um.txt", "Rust\nto be\nnot\nto go\n");
        let (code, out) = run_to_string(&config("to", vec![Input::File(a.clone())]));
        fs::remove_file(&a).unwrap();
        assert_eq!(code, FOUND);
        assert_eq!(out, "to be\nto go\n");
    }

    #[test]
    fn not_found() {
        let a = temp_file("nada.txt", "Rust\n");
        let (code, out) = run_to_string(&config("zzz", vec![Input::File(a.clone())]));
        fs::remove_file(&a).unwrap();
        assert_eq!(code, NOT_FOUND);
        assert_eq!(out, "");
    }

    #[test]
    fn names_and_line_numbers_with_two_files() {
        let a = temp_file("a.txt", "um gato\n");
        let b = temp_file("b.txt", "nada\noutro gato\n");
        let mut cfg = config("gato", vec![Input::File(a.clone()), Input::File(b.clone())]);
        cfg.output.line_numbers = true;
        let (code, out) = run_to_string(&cfg);
        fs::remove_file(&a).unwrap();
        fs::remove_file(&b).unwrap();
        assert_eq!(code, FOUND);
        assert_eq!(
            out,
            format!("{}:1:um gato\n{}:2:outro gato\n", a.display(), b.display())
        );
    }

    #[test]
    fn bad_file_is_error_but_continues() {
        let good = temp_file("bom.txt", "linha boa\n");
        let cfg = config(
            "boa",
            vec![
                Input::File("/nao/existe.txt".into()),
                Input::File(good.clone()),
            ],
        );
        let (code, out) = run_to_string(&cfg);
        fs::remove_file(&good).unwrap();
        assert_eq!(code, ERROR);
        assert_eq!(out, format!("{}:linha boa\n", good.display()));
    }

    /// Um `Write` que sempre falha, para testar o caminho de erro de `run`.
    struct Quebrado;

    impl Write for Quebrado {
        fn write(&mut self, _buf: &[u8]) -> io::Result<usize> {
            Err(io::Error::from(io::ErrorKind::BrokenPipe))
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    #[test]
    fn write_error_is_returned() {
        let a = temp_file("quebrado.txt", "x\n");
        let result = run(&config("x", vec![Input::File(a.clone())]), &mut Quebrado);
        fs::remove_file(&a).unwrap();
        assert_eq!(result.unwrap_err().kind(), io::ErrorKind::BrokenPipe);
    }
}

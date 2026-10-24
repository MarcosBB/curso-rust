//! minigrep: busca de texto em arquivos, no estilo do `grep`.
//!
//! A biblioteca não imprime nada: ela lê, busca e formata, e devolve dados
//! (`String`, `Vec`, erros). Quem escreve na tela e decide o código de saída
//! é o `main.rs`. Assim, tudo aqui pode ser testado sem rodar o programa.

pub mod config;
pub mod search;

use std::error::Error;
use std::fmt;
use std::fs;
use std::io::{self, Read};
use std::path::PathBuf;

use config::{Input, OutputOptions};
use search::Match;

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
/// `name` é o prefixo `arquivo:` (o `main` passa `Some` quando há mais de
/// uma entrada). Com `count_only`, a saída é uma linha só, com a contagem.
pub fn render(matches: &[Match<'_>], name: Option<&str>, opts: OutputOptions) -> Vec<String> {
    let prefix = match name {
        Some(name) => format!("{name}:"),
        None => String::new(),
    };

    if opts.count_only {
        return vec![format!("{prefix}{}", matches.len())];
    }

    matches
        .iter()
        .map(|m| {
            if opts.line_numbers {
                format!("{prefix}{}:{}", m.line_no, m.line)
            } else {
                format!("{prefix}{}", m.line)
            }
        })
        .collect()
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
            render(&sample(), None, OutputOptions::default()),
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
        assert_eq!(render(&sample(), None, opts), ["2"]);
        assert_eq!(render(&[], Some("b.txt"), opts), ["b.txt:0"]);
    }
}

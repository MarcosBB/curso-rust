//! minigrep: busca de texto em arquivos, no estilo do `grep`.
//!
//! A biblioteca não imprime nada: ela lê, busca e formata, e devolve dados
//! (`String`, `Vec`, erros). Quem escreve na tela e decide o código de saída
//! é o `main.rs`. Assim, tudo aqui pode ser testado sem rodar o programa.

// Só no projeto inicial: os corpos ainda são `todo!()`, então imports,
// parâmetros e variáveis aparecem como "não usados". Este atributo cala
// esses avisos. Apague esta linha quando terminar a etapa 5.
#![allow(unused)]

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
        // Mensagens esperadas (veja os testes da etapa 4):
        //   Read    -> "<caminho>: <causa>"
        //   NotUtf8 -> "<caminho>: o arquivo não é UTF-8 válido"
        //   Stdin   -> "entrada padrão: <causa>"
        todo!()
    }
}

impl Error for RunError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        // Devolva o `io::Error` guardado, quando houver.
        todo!()
    }
}

/// Lê uma entrada inteira para a memória.
///
/// O `String` devolvido é do chamador: as `Match` de `search::search`
/// vão apontar para dentro dele.
pub fn read_input(input: &Input) -> Result<String, RunError> {
    // Etapa 4: `io::stdin().read_to_string(..)` para `Stdin`,
    // `fs::read_to_string(..)` para `File`, convertendo o erro com `map_err`.
    todo!()
}

/// Formata o resultado de uma entrada, uma `String` por linha de saída.
///
/// `name` é o prefixo `arquivo:` (o `main` passa `Some` quando há mais de
/// uma entrada). Com `count_only`, a saída é uma linha só, com a contagem.
pub fn render(matches: &[Match<'_>], name: Option<&str>, opts: OutputOptions) -> Vec<String> {
    // Etapa 5: monte cada linha com `format!`.
    todo!()
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

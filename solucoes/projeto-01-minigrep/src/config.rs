//! Leitura dos argumentos da linha de comando.

use std::error::Error;
use std::fmt;
use std::path::PathBuf;

use crate::search::SearchOptions;

/// De onde vem o texto: da entrada padrão ou de um arquivo.
#[derive(Debug, PartialEq, Eq)]
pub enum Input {
    Stdin,
    File(PathBuf),
}

impl Input {
    /// Nome mostrado como prefixo na saída quando há mais de uma entrada.
    pub fn name(&self) -> String {
        match self {
            Input::Stdin => String::from("(entrada padrão)"),
            Input::File(path) => path.display().to_string(),
        }
    }
}

/// Opções que mudam só a forma de mostrar o resultado.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct OutputOptions {
    /// `-n`: mostra o número da linha.
    pub line_numbers: bool,
    /// `-c`: mostra só a quantidade de linhas.
    pub count_only: bool,
}

/// Tudo o que o programa precisa saber, já validado.
#[derive(Debug, PartialEq, Eq)]
pub struct Config {
    pub pattern: String,
    pub inputs: Vec<Input>,
    pub search: SearchOptions,
    pub output: OutputOptions,
}

/// Erros de uso: o usuário chamou o programa do jeito errado.
#[derive(Debug, PartialEq, Eq)]
pub enum ArgsError {
    MissingPattern,
    UnknownFlag(String),
}

impl fmt::Display for ArgsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ArgsError::MissingPattern => write!(f, "faltou o padrão de busca"),
            ArgsError::UnknownFlag(flag) => write!(f, "opção desconhecida: {flag}"),
        }
    }
}

// Nenhum dos dois casos tem uma causa interna, então o `source()` padrão
// (que devolve `None`) serve.
impl Error for ArgsError {}

impl Config {
    /// Monta a configuração a partir dos argumentos (sem o nome do programa).
    ///
    /// `ignore_case_env` diz se a variável `IGNORE_CASE` está definida. Ela
    /// vem de fora para que os testes não precisem mexer no ambiente do
    /// processo (`env::set_var` é `unsafe` na edition 2024).
    pub fn from_args(args: &[String], ignore_case_env: bool) -> Result<Config, ArgsError> {
        let mut search = SearchOptions {
            ignore_case: ignore_case_env,
            ..Default::default()
        };
        let mut output = OutputOptions::default();
        let mut positional: Vec<&str> = Vec::new();
        let mut only_positional = false;

        for arg in args {
            // `-` sozinho não é opção: é o nome da entrada padrão.
            if only_positional || arg == "-" || !arg.starts_with('-') {
                positional.push(arg);
                continue;
            }
            match arg.as_str() {
                "--" => only_positional = true,
                "-i" | "--ignore-case" => search.ignore_case = true,
                "-v" | "--invert-match" => search.invert = true,
                "-n" | "--line-number" => output.line_numbers = true,
                "-c" | "--count" => output.count_only = true,
                _ => return Err(ArgsError::UnknownFlag(arg.clone())),
            }
        }

        let Some(pattern) = positional.first() else {
            return Err(ArgsError::MissingPattern);
        };

        let mut inputs = Vec::new();
        for &file in &positional[1..] {
            if file == "-" {
                inputs.push(Input::Stdin);
            } else {
                inputs.push(Input::File(PathBuf::from(file)));
            }
        }
        if inputs.is_empty() {
            inputs.push(Input::Stdin);
        }

        Ok(Config {
            pattern: pattern.to_string(),
            inputs,
            search,
            output,
        })
    }
}

#[cfg(test)]
mod step3_tests {
    use super::*;

    /// Converte `&["a", "b"]` em `Vec<String>`, como `env::args()` entregaria.
    fn args(list: &[&str]) -> Vec<String> {
        let mut out = Vec::new();
        for s in list {
            out.push(s.to_string());
        }
        out
    }

    #[test]
    fn pattern_and_files() {
        let cfg = Config::from_args(&args(&["foo", "a.txt", "b.txt"]), false).unwrap();
        assert_eq!(cfg.pattern, "foo");
        assert_eq!(
            cfg.inputs,
            vec![Input::File("a.txt".into()), Input::File("b.txt".into())]
        );
        assert_eq!(cfg.search, SearchOptions::default());
        assert_eq!(cfg.output, OutputOptions::default());
    }

    #[test]
    fn short_flags() {
        let cfg = Config::from_args(&args(&["-i", "-n", "-v", "-c", "foo"]), false).unwrap();
        assert!(cfg.search.ignore_case && cfg.search.invert);
        assert!(cfg.output.line_numbers && cfg.output.count_only);
    }

    #[test]
    fn long_flags_anywhere() {
        let cfg = Config::from_args(
            &args(&["foo", "--ignore-case", "a.txt", "--line-number"]),
            false,
        )
        .unwrap();
        assert!(cfg.search.ignore_case && cfg.output.line_numbers);
        assert_eq!(cfg.inputs, vec![Input::File("a.txt".into())]);
    }

    #[test]
    fn stdin_when_no_files() {
        let cfg = Config::from_args(&args(&["foo"]), false).unwrap();
        assert_eq!(cfg.inputs, vec![Input::Stdin]);
    }

    #[test]
    fn dash_is_stdin() {
        let cfg = Config::from_args(&args(&["foo", "a.txt", "-"]), false).unwrap();
        assert_eq!(cfg.inputs, vec![Input::File("a.txt".into()), Input::Stdin]);
    }

    #[test]
    fn double_dash_ends_flags() {
        let cfg = Config::from_args(&args(&["--", "-v", "-n"]), false).unwrap();
        assert_eq!(cfg.pattern, "-v");
        assert_eq!(cfg.inputs, vec![Input::File("-n".into())]);
        assert!(!cfg.search.invert);
    }

    #[test]
    fn env_turns_on_ignore_case() {
        let cfg = Config::from_args(&args(&["foo"]), true).unwrap();
        assert!(cfg.search.ignore_case);
    }

    #[test]
    fn missing_pattern() {
        assert_eq!(
            Config::from_args(&args(&["-i"]), false),
            Err(ArgsError::MissingPattern)
        );
    }

    #[test]
    fn unknown_flag() {
        assert_eq!(
            Config::from_args(&args(&["-x", "foo"]), false),
            Err(ArgsError::UnknownFlag("-x".to_string()))
        );
    }

    #[test]
    fn args_error_messages() {
        assert_eq!(
            ArgsError::MissingPattern.to_string(),
            "faltou o padrão de busca"
        );
        assert_eq!(
            ArgsError::UnknownFlag("-x".to_string()).to_string(),
            "opção desconhecida: -x"
        );
    }
}

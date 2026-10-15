//! Lição 05, lado da biblioteca: um tipo de erro concreto, gerado com `thiserror`.
//!
//! O formato do arquivo é uma chave por linha (`host = localhost`, `port = 8080`).
//! Linhas vazias e linhas que começam com `#` são ignoradas.

use std::fs;
use std::io;
use std::num::ParseIntError;

use thiserror::Error;

#[derive(Debug, PartialEq)]
pub struct Config {
    pub host: String,
    pub port: u16,
}

/// Erros da biblioteca. O chamador pode casar cada variante com `match`.
#[derive(Debug, Error)]
pub enum ConfigError {
    /// `#[source]` marca a causa sem gerar `From`: a variante tem um campo extra (`path`).
    #[error("não consegui ler {path}")]
    Io {
        path: String,
        #[source]
        source: io::Error,
    },

    #[error("linha {line}: esperava `chave = valor`")]
    MissingEquals { line: usize },

    /// A mensagem NÃO repete a causa: ela já sai pelo `source()`.
    #[error("linha {line}: porta inválida")]
    BadPort {
        line: usize,
        #[source]
        source: ParseIntError,
    },

    #[error("a porta {0} é reservada (use 1024 ou mais)")]
    PortTooLow(u16),

    #[error("chave obrigatória ausente: `{0}`")]
    MissingKey(&'static str),
}

/// Lê e interpreta o arquivo de configuração.
pub fn load(path: &str) -> Result<Config, ConfigError> {
    let text = fs::read_to_string(path).map_err(|source| ConfigError::Io {
        path: path.to_string(),
        source,
    })?;
    parse(&text)
}

/// Interpreta o texto de uma configuração.
pub fn parse(text: &str) -> Result<Config, ConfigError> {
    let mut host = None;
    let mut port = None;

    for (i, raw) in text.lines().enumerate() {
        let line = i + 1;
        let raw = raw.trim();
        if raw.is_empty() || raw.starts_with('#') {
            continue;
        }
        let (key, value) = raw
            .split_once('=')
            .ok_or(ConfigError::MissingEquals { line })?;
        match key.trim() {
            "host" => host = Some(value.trim().to_string()),
            "port" => {
                let p: u16 = value
                    .trim()
                    .parse()
                    .map_err(|source| ConfigError::BadPort { line, source })?;
                if p < 1024 {
                    return Err(ConfigError::PortTooLow(p));
                }
                port = Some(p);
            }
            _ => {} // chaves desconhecidas são ignoradas
        }
    }

    Ok(Config {
        host: host.ok_or(ConfigError::MissingKey("host"))?,
        port: port.ok_or(ConfigError::MissingKey("port"))?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error;

    #[test]
    fn config_valida() {
        let c = parse("# comentário\nhost = localhost\nport = 8080\n").unwrap();
        assert_eq!(
            c,
            Config {
                host: "localhost".to_string(),
                port: 8080
            }
        );
    }

    #[test]
    fn porta_invalida_guarda_a_linha_e_a_causa() {
        let e = parse("host = a\nport = 80a").unwrap_err();
        assert!(matches!(e, ConfigError::BadPort { line: 2, .. }));
        assert_eq!(e.to_string(), "linha 2: porta inválida");
        assert_eq!(
            e.source().unwrap().to_string(),
            "invalid digit found in string"
        );
    }

    #[test]
    fn outros_erros() {
        assert!(matches!(
            parse("host a"),
            Err(ConfigError::MissingEquals { line: 1 })
        ));
        assert!(matches!(
            parse("host = a\nport = 80"),
            Err(ConfigError::PortTooLow(80))
        ));
        assert!(matches!(
            parse("port = 8080"),
            Err(ConfigError::MissingKey("host"))
        ));
    }

    #[test]
    fn arquivo_inexistente_guarda_o_caminho() {
        let e = load("nao-existe.conf").unwrap_err();
        assert_eq!(e.to_string(), "não consegui ler nao-existe.conf");
        assert!(matches!(e, ConfigError::Io { .. }));
    }
}

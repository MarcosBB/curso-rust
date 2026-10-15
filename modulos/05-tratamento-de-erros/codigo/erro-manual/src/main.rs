//! Lição 04: um tipo de erro próprio, escrito à mão.
//!
//! Rode com um dos arquivos de `exemplos/`:
//!
//!     cargo run -- exemplos/porta-ok.txt
//!     cargo run -- exemplos/porta-invalida.txt
//!     cargo run -- exemplos/porta-baixa.txt
//!     cargo run -- nao-existe.txt

use std::error::Error;
use std::fmt;
use std::fs;
use std::io;
use std::num::ParseIntError;
use std::process::ExitCode;

/// Tudo o que pode dar errado ao ler a porta do arquivo de configuração.
#[derive(Debug)]
enum ConfigError {
    /// Não deu para ler o arquivo.
    Io(io::Error),
    /// O conteúdo não é um número de 0 a 65535.
    BadPort(ParseIntError),
    /// O número é válido, mas é uma porta reservada.
    PortTooLow(u16),
}

// Receita 1: Display, a mensagem para humanos.
// Minúsculas, sem ponto final e sem repetir a causa (ela sai pelo source()).
impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigError::Io(_) => write!(f, "não consegui ler o arquivo de configuração"),
            ConfigError::BadPort(_) => write!(f, "porta inválida"),
            ConfigError::PortTooLow(p) => write!(f, "a porta {p} é reservada (use 1024 ou mais)"),
        }
    }
}

// Receita 2: Error, que liga o erro à sua causa.
impl Error for ConfigError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            ConfigError::Io(e) => Some(e),
            ConfigError::BadPort(e) => Some(e),
            ConfigError::PortTooLow(_) => None,
        }
    }
}

// Receita 3: From, para o `?` converter os erros de baixo sozinho.
impl From<io::Error> for ConfigError {
    fn from(e: io::Error) -> Self {
        ConfigError::Io(e)
    }
}

impl From<ParseIntError> for ConfigError {
    fn from(e: ParseIntError) -> Self {
        ConfigError::BadPort(e)
    }
}

fn read_port(path: &str) -> Result<u16, ConfigError> {
    let text = fs::read_to_string(path)?; // io::Error -> ConfigError::Io
    let port: u16 = text.trim().parse()?; // ParseIntError -> ConfigError::BadPort
    if port < 1024 {
        return Err(ConfigError::PortTooLow(port));
    }
    Ok(port)
}

/// Imprime o erro e toda a cadeia de causas, uma por linha.
fn report(e: &dyn Error) {
    eprintln!("erro: {e}");
    let mut cause = e.source();
    while let Some(c) = cause {
        eprintln!("  causado por: {c}");
        cause = c.source();
    }
}

fn main() -> ExitCode {
    let Some(path) = std::env::args().nth(1) else {
        eprintln!("uso: erro-manual <arquivo>");
        return ExitCode::from(2);
    };

    match read_port(&path) {
        Ok(port) => {
            println!("porta configurada: {port}");
            ExitCode::SUCCESS
        }
        Err(e) => {
            report(&e);
            ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn exemplo(nome: &str) -> String {
        format!("{}/exemplos/{nome}", env!("CARGO_MANIFEST_DIR"))
    }

    #[test]
    fn porta_valida() {
        assert_eq!(read_port(&exemplo("porta-ok.txt")).unwrap(), 8080);
    }

    #[test]
    fn porta_invalida_tem_causa() {
        let e = read_port(&exemplo("porta-invalida.txt")).unwrap_err();
        assert!(matches!(e, ConfigError::BadPort(_)));
        assert_eq!(e.to_string(), "porta inválida");
        assert_eq!(
            e.source().unwrap().to_string(),
            "invalid digit found in string"
        );
    }

    #[test]
    fn porta_reservada_nao_tem_causa() {
        let e = read_port(&exemplo("porta-baixa.txt")).unwrap_err();
        assert!(matches!(e, ConfigError::PortTooLow(80)));
        assert!(e.source().is_none());
    }

    #[test]
    fn arquivo_inexistente() {
        let e = read_port(&exemplo("nao-existe.txt")).unwrap_err();
        let ConfigError::Io(io_err) = e else {
            panic!("esperava ConfigError::Io, veio {e:?}");
        };
        assert_eq!(io_err.kind(), io::ErrorKind::NotFound);
    }
}

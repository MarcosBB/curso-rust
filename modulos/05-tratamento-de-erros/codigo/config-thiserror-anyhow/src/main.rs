//! Lição 05, lado da aplicação: `anyhow` para propagar e reportar.
//!
//!     cargo run -- exemplos/ok.conf
//!     cargo run -- exemplos/porta-invalida.conf
//!     cargo run -- exemplos/porta-reservada.conf
//!     cargo run -- nao-existe.conf

use std::process::ExitCode;

use anyhow::{Context, Result, bail};
use config_thiserror_anyhow::{ConfigError, load};

fn run() -> Result<()> {
    let Some(path) = std::env::args().nth(1) else {
        bail!("uso: config-thiserror-anyhow <arquivo>");
    };

    // A biblioteca já põe o caminho na mensagem; aqui só dizemos o que estávamos fazendo.
    let config = load(&path).context("falha ao carregar a configuração")?;

    println!("servidor em {}:{}", config.host, config.port);
    Ok(())
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            // {:#} imprime a cadeia inteira numa linha, separada por ": "
            eprintln!("erro: {e:#}");

            // downcast_ref recupera o tipo concreto da biblioteca, se for ele
            if let Some(ConfigError::PortTooLow(_)) = e.downcast_ref::<ConfigError>() {
                eprintln!("dica: portas abaixo de 1024 exigem privilégios de administrador");
            }
            ExitCode::FAILURE
        }
    }
}

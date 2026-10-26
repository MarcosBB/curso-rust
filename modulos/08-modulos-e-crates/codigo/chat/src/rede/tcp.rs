//! Conexões TCP (simuladas).

use super::{PORTA_PADRAO, host_valido};

/// Uma conexão aberta. Os campos são privados: só [`conectar`] cria uma.
#[derive(Debug)]
pub struct Conexao {
    host: String,
    porta: u16,
}

impl Conexao {
    /// O endereço no formato `host:porta`.
    pub fn endereco(&self) -> String {
        format!("{}:{}", self.host, self.porta)
    }
}

/// Abre uma conexão com `host`, na porta indicada ou na [`PORTA_PADRAO`].
pub fn conectar(host: &str, porta: Option<u16>) -> Result<Conexao, String> {
    if !host_valido(host) {
        return Err(format!("host inválido: {host:?}"));
    }
    Ok(Conexao {
        host: host.to_string(),
        porta: porta.unwrap_or(PORTA_PADRAO),
    })
}

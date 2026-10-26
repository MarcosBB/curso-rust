//! O cliente de chat, construído sobre `rede`.

use crate::rede::tcp::{self, Conexao};

/// Conecta ao servidor e devolve a conexão e a mensagem de boas-vindas.
pub fn iniciar(servidor: &str) -> Result<(Conexao, String), String> {
    let conexao = tcp::conectar(servidor, None)?;
    let boas_vindas = crate::registrar(&format!("conectado a {}", conexao.endereco()));
    Ok((conexao, boas_vindas))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inicia_na_porta_padrao() {
        let (conexao, msg) = iniciar("exemplo.com").unwrap();
        assert_eq!(conexao.endereco(), "exemplo.com:7000");
        assert_eq!(msg, "[chat] conectado a exemplo.com:7000");
    }

    #[test]
    fn recusa_host_invalido() {
        assert!(iniciar("sem espaço").is_err());
    }
}

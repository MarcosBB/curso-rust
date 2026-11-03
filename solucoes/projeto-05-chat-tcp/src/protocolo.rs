//! O protocolo de linhas: o que o cliente manda e como o servidor entende.
//!
//! Tudo aqui é síncrono e puro: nenhuma rede, nenhum `async`.

use thiserror::Error;

/// Tamanho máximo de um apelido, em caracteres.
pub const MAX_APELIDO: usize = 20;

/// O que uma linha recebida do cliente quer dizer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Comando {
    /// Linha em branco (ou só com espaços): ignorada.
    Vazio,
    /// Texto comum, para todo mundo.
    Mensagem(String),
    /// `/nick NOME`: trocar de apelido. O nome ainda não foi validado.
    Nick(String),
    /// `/quem`: quem está na sala.
    Quem,
    /// `/sair` (ou `/quit`): encerrar a conexão.
    Sair,
    /// Qualquer outra coisa que comece com `/`.
    Desconhecido(String),
}

/// Por que um apelido foi recusado.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ErroApelido {
    #[error("o apelido não pode ser vazio")]
    Vazio,
    #[error("o apelido pode ter no máximo {MAX_APELIDO} caracteres")]
    Longo,
    #[error("caractere não permitido no apelido: {0:?}")]
    CaractereInvalido(char),
    #[error("o apelido {0} já está em uso")]
    EmUso(String),
}

/// Interpreta uma linha (já sem o `\n`).
///
/// Espaços nas pontas são ignorados. Uma linha que começa com `/` é um
/// comando: o nome vai até o primeiro espaço, e o resto é o argumento.
pub fn interpretar(linha: &str) -> Comando {
    let linha = linha.trim();
    if linha.is_empty() {
        return Comando::Vazio;
    }
    if !linha.starts_with('/') {
        return Comando::Mensagem(linha.to_string());
    }
    let (nome, argumento) = linha.split_once(' ').unwrap_or((linha, ""));
    match nome {
        "/nick" => Comando::Nick(argumento.trim().to_string()),
        "/quem" => Comando::Quem,
        "/sair" | "/quit" => Comando::Sair,
        _ => Comando::Desconhecido(nome.to_string()),
    }
}

/// Um apelido válido tem de 1 a [`MAX_APELIDO`] caracteres, só letras,
/// dígitos, `-` e `_`. (Se ele já está em uso, quem decide é a [`Sala`].)
///
/// [`Sala`]: crate::Sala
pub fn validar_apelido(apelido: &str) -> Result<(), ErroApelido> {
    if apelido.is_empty() {
        return Err(ErroApelido::Vazio);
    }
    if apelido.chars().count() > MAX_APELIDO {
        return Err(ErroApelido::Longo);
    }
    match apelido
        .chars()
        .find(|&c| !(c.is_alphanumeric() || c == '-' || c == '_'))
    {
        Some(c) => Err(ErroApelido::CaractereInvalido(c)),
        None => Ok(()),
    }
}

#[cfg(test)]
mod step1_tests {
    use super::*;

    #[test]
    fn mensagens_e_linhas_vazias() {
        assert_eq!(interpretar(""), Comando::Vazio);
        assert_eq!(interpretar("   "), Comando::Vazio);
        assert_eq!(interpretar("oi"), Comando::Mensagem("oi".into()));
        assert_eq!(
            interpretar("  bom dia, gente  "),
            Comando::Mensagem("bom dia, gente".into())
        );
    }

    #[test]
    fn comandos() {
        assert_eq!(interpretar("/quem"), Comando::Quem);
        assert_eq!(interpretar("/quem agora"), Comando::Quem);
        assert_eq!(interpretar("/sair"), Comando::Sair);
        assert_eq!(interpretar("/quit"), Comando::Sair);
        assert_eq!(interpretar("/nick ana"), Comando::Nick("ana".into()));
        assert_eq!(interpretar("/nick   ana  "), Comando::Nick("ana".into()));
        assert_eq!(interpretar("/nick"), Comando::Nick(String::new()));
        assert_eq!(
            interpretar("/nick ana maria"),
            Comando::Nick("ana maria".into())
        );
        assert_eq!(
            interpretar("/xyz 1 2"),
            Comando::Desconhecido("/xyz".into())
        );
        assert_eq!(interpretar("/"), Comando::Desconhecido("/".into()));
    }

    #[test]
    fn apelidos_validos() {
        for a in [
            "ana",
            "Zé",
            "bob_2",
            "maria-clara",
            "a",
            "x".repeat(20).as_str(),
        ] {
            assert_eq!(validar_apelido(a), Ok(()), "{a}");
        }
    }

    #[test]
    fn apelidos_invalidos() {
        assert_eq!(validar_apelido(""), Err(ErroApelido::Vazio));
        assert_eq!(validar_apelido(&"x".repeat(21)), Err(ErroApelido::Longo));
        // 20 caracteres com acento ainda valem: o limite é em caracteres, não em bytes.
        assert_eq!(validar_apelido(&"é".repeat(20)), Ok(()));
        assert_eq!(
            validar_apelido("ana maria"),
            Err(ErroApelido::CaractereInvalido(' '))
        );
        assert_eq!(
            validar_apelido("ana:"),
            Err(ErroApelido::CaractereInvalido(':'))
        );
    }

    #[test]
    fn mensagens_de_erro() {
        assert_eq!(
            ErroApelido::Longo.to_string(),
            "o apelido pode ter no máximo 20 caracteres"
        );
        assert_eq!(
            ErroApelido::EmUso("ana".into()).to_string(),
            "o apelido ana já está em uso"
        );
    }
}

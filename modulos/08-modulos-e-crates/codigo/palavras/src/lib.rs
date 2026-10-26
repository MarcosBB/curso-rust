//! Biblioteca do pacote `palavras`: usada pelos dois executáveis,
//! pelo exemplo e pelos testes de integração (lição 04).

/// Conta as palavras separadas por espaço em branco.
pub fn contar(texto: &str) -> usize {
    texto.split_whitespace().count()
}

/// A palavra mais longa (a primeira, em caso de empate).
pub fn mais_longa(texto: &str) -> Option<&str> {
    texto
        .split_whitespace()
        .fold(None, |maior: Option<&str>, p| match maior {
            Some(m) if m.len() >= p.len() => Some(m),
            _ => Some(p),
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn conta_palavras() {
        assert_eq!(contar("  um dois\ttrês "), 3);
        assert_eq!(contar(""), 0);
    }
}

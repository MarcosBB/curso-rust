//! Conta linhas, palavras e bytes de um texto, como o `wc` do Unix.
//! Exemplo da lição 04 do Módulo 09 (testes de integração).

/// O resultado de uma contagem.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Contagem {
    pub linhas: usize,
    pub palavras: usize,
    pub bytes: usize,
}

/// Conta as linhas, as palavras (separadas por espaço em branco) e os bytes.
///
/// # Examples
///
/// ```
/// let c = contador::contar("um dois\ntrês\n");
/// assert_eq!((c.linhas, c.palavras), (2, 3));
/// ```
pub fn contar(texto: &str) -> Contagem {
    Contagem {
        linhas: texto.lines().count(),
        palavras: texto.split_whitespace().count(),
        bytes: texto.len(),
    }
}

/// Formata uma contagem como o `wc`: linhas, palavras e bytes, separados por espaço.
pub fn formatar(c: Contagem) -> String {
    format!("{} {} {}", c.linhas, c.palavras, c.bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn texto_vazio() {
        assert_eq!(contar(""), Contagem::default());
    }

    #[test]
    fn bytes_contam_utf8() {
        // "ção" tem 3 caracteres, mas 5 bytes
        assert_eq!(contar("ção").bytes, 5);
    }
}

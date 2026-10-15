/// Tira a pontuação das pontas e passa para minúsculas.
pub fn normalizar(palavra: &str) -> String {
    palavra
        .trim_matches(|c: char| !c.is_alphanumeric())
        .to_lowercase()
}

/// Separa o texto em palavras normalizadas, descartando as que ficam vazias.
pub fn palavras(texto: &str) -> Vec<String> {
    let mut resultado: Vec<String> = texto.split_whitespace().map(normalizar).collect();
    resultado.retain(|p| !p.is_empty());
    resultado
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normaliza() {
        assert_eq!(normalizar("Olá,"), "olá");
        assert_eq!(normalizar("(Rust)"), "rust");
        assert_eq!(normalizar("d'água"), "d'água");
        assert_eq!(normalizar("--"), "");
    }

    #[test]
    fn separa_palavras() {
        assert_eq!(palavras("O rato, o RATO!"), vec!["o", "rato", "o", "rato"]);
    }

    #[test]
    fn descarta_vazias() {
        assert!(palavras("  -- ... ").is_empty());
    }
}

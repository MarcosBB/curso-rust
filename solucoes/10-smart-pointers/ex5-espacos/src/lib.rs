//! Exercício 5: normalizar espaços sem alocar quando não precisa.

use std::borrow::Cow;

/// Tira os espaços das pontas e troca cada sequência de espaços em branco
/// internos por um único `' '`. Só aloca se o texto mudar.
pub fn normalizar_espacos(s: &str) -> Cow<'_, str> {
    let aparado = s.trim();
    if ja_normalizado(aparado) {
        // `trim` devolve uma fatia de `s`: ainda é um empréstimo.
        return Cow::Borrowed(aparado);
    }
    let mut saida = String::with_capacity(aparado.len());
    for (i, palavra) in aparado.split_whitespace().enumerate() {
        if i > 0 {
            saida.push(' ');
        }
        saida.push_str(palavra);
    }
    Cow::Owned(saida)
}

/// `true` se todo espaço em branco é um `' '` sozinho.
fn ja_normalizado(s: &str) -> bool {
    let mut anterior_era_espaco = false;
    for c in s.chars() {
        if c.is_whitespace() {
            if c != ' ' || anterior_era_espaco {
                return false;
            }
            anterior_era_espaco = true;
        } else {
            anterior_era_espaco = false;
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ja_normal_nao_aloca() {
        let r = normalizar_espacos("bom dia");
        assert_eq!(r, "bom dia");
        assert!(matches!(r, Cow::Borrowed(_)));
    }

    #[test]
    fn so_pontas_nao_aloca() {
        let r = normalizar_espacos("  bom dia \n");
        assert_eq!(r, "bom dia");
        assert!(matches!(r, Cow::Borrowed(_)));
    }

    #[test]
    fn espacos_repetidos_alocam() {
        let r = normalizar_espacos("bom   dia,\t\tmundo");
        assert_eq!(r, "bom dia, mundo");
        assert!(matches!(r, Cow::Owned(_)));
    }

    #[test]
    fn tab_sozinho_tambem_e_trocado() {
        let r = normalizar_espacos("a\tb");
        assert_eq!(r, "a b");
        assert!(matches!(r, Cow::Owned(_)));
    }

    #[test]
    fn vazio() {
        assert_eq!(normalizar_espacos("   "), "");
        assert_eq!(normalizar_espacos(""), "");
    }

    #[test]
    fn devolve_string_quando_precisa() {
        let dono: String = normalizar_espacos(" x  y ").into_owned();
        assert_eq!(dono, "x y");
    }
}

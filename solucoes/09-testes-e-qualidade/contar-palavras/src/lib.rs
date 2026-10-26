//! Solução do exercício 6 do Módulo 09: duas formas de contar palavras.

/// Com o iterador da std.
pub fn contar_split(texto: &str) -> usize {
    texto.split_whitespace().count()
}

/// Com um laço manual sobre os caracteres.
pub fn contar_manual(texto: &str) -> usize {
    let mut total = 0;
    let mut dentro = false;
    for c in texto.chars() {
        if c.is_whitespace() {
            dentro = false;
        } else if !dentro {
            dentro = true;
            total += 1;
        }
    }
    total
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn as_duas_concordam() {
        for texto in [
            "",
            "um",
            "  dois  espaços ",
            "linha\nnova\tcom tab",
            "ação é ótima",
        ] {
            assert_eq!(
                contar_split(texto),
                contar_manual(texto),
                "texto: {texto:?}"
            );
        }
    }
}

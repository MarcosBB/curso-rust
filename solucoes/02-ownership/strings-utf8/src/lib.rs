//! Solução do exercício 5: funções de texto que nunca dão panic com UTF-8.

/// O primeiro `char` (valor escalar Unicode), se houver.
pub fn primeiro_char(s: &str) -> Option<char> {
    s.chars().next()
}

/// Inverte a ordem dos `char`s. Não é a mesma coisa que inverter
/// o que uma pessoa enxerga como "letras" (veja o teste do emoji).
pub fn inverter(s: &str) -> String {
    s.chars().rev().collect()
}

/// Os primeiros `max_chars` caracteres de `s`, sem copiar nada.
/// `char_indices` dá o índice em bytes onde cada char começa, que é
/// sempre uma fronteira válida para fatiar.
pub fn truncar(s: &str, max_chars: usize) -> &str {
    match s.char_indices().nth(max_chars) {
        Some((fim, _)) => &s[..fim],
        None => s,
    }
}

/// Posição de `alvo` contada em chars (e não em bytes, como em `find`).
pub fn posicao_em_chars(s: &str, alvo: char) -> Option<usize> {
    s.chars().position(|c| c == alvo)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn primeiro_char_funciona_com_acentos_e_vazio() {
        assert_eq!(primeiro_char("olá"), Some('o'));
        assert_eq!(primeiro_char("ação"), Some('a'));
        assert_eq!(primeiro_char("éramos"), Some('é'));
        assert_eq!(primeiro_char(""), None);
    }

    #[test]
    fn inverter_por_char() {
        assert_eq!(inverter("olá"), "álo");
        assert_eq!(inverter("çãé"), "éãç");
        assert_eq!(inverter(""), "");
    }

    #[test]
    fn inverter_separa_o_modificador_de_tom() {
        // "👍🏽" são dois chars: o polegar e o modificador de tom de pele.
        // Invertidos, o modificador vem primeiro e o emoji "se desmancha".
        assert_eq!(inverter("👍🏽"), "\u{1F3FD}\u{1F44D}");
    }

    #[test]
    fn truncar_conta_chars_e_nao_bytes() {
        assert_eq!(truncar("olá", 2), "ol");
        assert_eq!(truncar("olá", 3), "olá");
        assert_eq!(truncar("çãé", 2), "çã");
        assert_eq!(truncar("👍🏽", 1), "👍");
    }

    #[test]
    fn truncar_nos_extremos() {
        assert_eq!(truncar("olá", 0), "");
        assert_eq!(truncar("olá", 10), "olá");
        assert_eq!(truncar("", 3), "");
    }

    #[test]
    fn posicao_em_chars_x_find() {
        let s = "pão de queijo";
        assert_eq!(s.find('d'), Some(5)); // bytes: "ã" ocupa 2
        assert_eq!(posicao_em_chars(s, 'd'), Some(4));
        assert_eq!(posicao_em_chars(s, 'z'), None);
    }
}

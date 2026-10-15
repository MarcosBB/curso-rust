//! Solução do exercício 4: só as assinaturas mudaram.

/// Só lê: `&str`.
pub fn contar_vogais(s: &str) -> usize {
    s.chars().filter(|c| "aeiouAEIOU".contains(*c)).count()
}

/// Lê o nome e cria um texto novo: parâmetro `&str`, retorno `String`.
pub fn saudacao(nome: &str) -> String {
    format!("Olá, {nome}!")
}

/// Mesmo caso: `to_uppercase` já cria uma `String` nova.
pub fn gritar(s: &str) -> String {
    format!("{}!", s.to_uppercase())
}

/// Só lê a lista: uma fatia `&[String]`. `&Vec<String>` também compilaria,
/// mas `&[String]` aceita mais coisas (vetor, array, pedaço de vetor), pelo
/// mesmo motivo que `&str` é melhor que `&String`.
pub fn total_de_letras(palavras: &[String]) -> usize {
    let mut total = 0;
    for p in palavras {
        total += p.chars().count();
    }
    total
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vogais_com_literal() {
        assert_eq!(contar_vogais("banana"), 3);
    }

    #[test]
    fn vogais_sem_perder_a_string() {
        let s = String::from("Ferris");
        assert_eq!(contar_vogais(&s), 2);
        assert_eq!(s, "Ferris");
    }

    #[test]
    fn saudacao_com_literal() {
        assert_eq!(saudacao("Ana"), "Olá, Ana!");
    }

    #[test]
    fn saudacao_com_string() {
        let nome = String::from("Bia");
        let s = saudacao(&nome);
        assert_eq!(s, "Olá, Bia!");
        assert_eq!(nome, "Bia");
    }

    #[test]
    fn gritar_com_literal() {
        assert_eq!(gritar("oi"), "OI!");
    }

    #[test]
    fn total_sem_consumir_o_vetor() {
        let palavras = vec![String::from("olá"), String::from("mundo")];
        assert_eq!(total_de_letras(&palavras), 8);
        assert_eq!(palavras.len(), 2);
    }
}

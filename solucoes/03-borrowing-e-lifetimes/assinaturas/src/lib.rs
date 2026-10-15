/// Devolve a string mais longa (a primeira, em caso de empate).
pub fn longest<'a>(x: &'a str, y: &'a str) -> &'a str {
    if x.len() >= y.len() { x } else { y }
}

/// Devolve a primeira palavra (separada por espaço).
pub fn first_word(s: &str) -> &str {
    s.split(' ').next().unwrap_or("")
}

/// Devolve sempre o primeiro argumento. O segundo só é registrado em log.
pub fn keep_first<'a>(x: &'a str, y: &str) -> &'a str {
    println!("descartando {y}");
    x
}

/// Versão do programa.
pub fn version() -> &'static str {
    "1.0.3"
}

/// Mensagem de saudação conforme a hora.
pub fn greeting(hour: u32) -> String {
    let period = if hour < 12 { "bom dia" } else { "boa tarde" };
    format!("{period}! são {hour}h")
}

/// Divide em duas partes no primeiro `sep`. Sem `sep`, a segunda parte é vazia.
pub fn split_pair(s: &str, sep: char) -> (&str, &str) {
    match s.find(sep) {
        Some(i) => (&s[..i], &s[i + sep.len_utf8()..]),
        None => (s, ""),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_longest() {
        assert_eq!(longest("abc", "de"), "abc");
        assert_eq!(longest("ab", "cde"), "cde");
    }

    #[test]
    fn test_first_word() {
        assert_eq!(first_word("olá mundo"), "olá");
    }

    #[test]
    fn keep_first_nao_depende_do_segundo() {
        let x = String::from("fica");
        let r;
        {
            let y = String::from("morre");
            r = keep_first(&x, &y);
        }
        assert_eq!(r, "fica");
    }

    #[test]
    fn test_version() {
        assert_eq!(version(), "1.0.3");
    }

    #[test]
    fn test_greeting() {
        assert_eq!(greeting(9), "bom dia! são 9h");
        assert_eq!(greeting(15), "boa tarde! são 15h");
    }

    #[test]
    fn test_split_pair() {
        assert_eq!(split_pair("chave=valor", '='), ("chave", "valor"));
        assert_eq!(split_pair("sozinho", '='), ("sozinho", ""));
    }
}

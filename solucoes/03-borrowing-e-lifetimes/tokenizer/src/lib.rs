/// Quebra um texto em tokens: sequências de letras, dígitos e `_`, ou um
/// único caractere de pontuação. Espaços são ignorados.
///
/// O `Tokenizer` só guarda uma visão do texto que ainda falta ler. Ele não
/// copia nada.
pub struct Tokenizer<'a> {
    rest: &'a str,
}

fn is_word_char(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

impl<'a> Tokenizer<'a> {
    pub fn new(src: &'a str) -> Self {
        Tokenizer { rest: src }
    }

    /// O retorno é `&'a str`: o token aponta para o texto original e vale
    /// enquanto o TEXTO valer, e não só enquanto este empréstimo de `self`.
    pub fn next_token(&mut self) -> Option<&'a str> {
        let s = self.rest.trim_start();
        // `?` em `Option`: se não houver próximo caractere, devolve `None`
        // na hora. É o mesmo que um `match` com `None => return None`.
        // O operador `?` é explicado no Módulo 05.
        let first = s.chars().next()?;
        let end = if is_word_char(first) {
            s.find(|c: char| !is_word_char(c)).unwrap_or(s.len())
        } else {
            first.len_utf8()
        };
        let (token, rest) = s.split_at(end);
        self.rest = rest;
        Some(token)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn all_tokens(src: &str) -> Vec<&str> {
        let mut t = Tokenizer::new(src);
        let mut out = Vec::new();
        // Com `-> Option<&str>` (preso a `&mut self`), este laço dá E0499.
        while let Some(tok) = t.next_token() {
            out.push(tok);
        }
        out
    }

    #[test]
    fn separa_palavras_e_pontuacao() {
        assert_eq!(all_tokens("let x=42;"), ["let", "x", "=", "42", ";"]);
    }

    #[test]
    fn ignora_espacos_e_aceita_unicode() {
        assert_eq!(all_tokens("  ação  (já)  "), ["ação", "(", "já", ")"]);
        assert!(all_tokens("   ").is_empty());
    }

    #[test]
    fn token_sobrevive_ao_tokenizer() {
        fn first_token(src: &str) -> Option<&str> {
            let mut t = Tokenizer::new(src);
            t.next_token()
        }
        let src = String::from("primeiro segundo");
        assert_eq!(first_token(&src), Some("primeiro"));
    }
}

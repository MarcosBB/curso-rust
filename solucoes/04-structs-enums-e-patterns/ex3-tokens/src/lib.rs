#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Num(i64),
    Op(char),
    LParen,
    RParen,
    Ident(String),
}

pub fn classify(t: &Token) -> &'static str {
    match t {
        Token::Num(0) => "zero",
        Token::Num(n) if *n < 0 => "negativo",
        Token::Num(1..=9) => "dígito",
        Token::Num(_) => "número",
        Token::Op('+' | '-') => "aditivo",
        Token::Op('*' | '/' | '%') => "multiplicativo",
        Token::Op(_) => "operador desconhecido",
        Token::LParen | Token::RParen => "parêntese",
        Token::Ident(name) if name.starts_with('_') => "privado",
        Token::Ident(_) => "identificador",
    }
}

pub fn describe(t: &Token) -> String {
    match t {
        Token::Num(n @ 1..=9) => format!("o dígito {n}"),
        Token::Num(n) => format!("o número {n}"),
        Token::Op(op @ ('+' | '-' | '*' | '/' | '%')) => format!("o operador {op}"),
        Token::Op(other) => format!("o símbolo {other:?}"),
        Token::LParen => String::from("abre parêntese"),
        Token::RParen => String::from("fecha parêntese"),
        Token::Ident(name) => format!("o nome {name}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn numbers() {
        assert_eq!(classify(&Token::Num(0)), "zero");
        assert_eq!(classify(&Token::Num(-3)), "negativo");
        assert_eq!(classify(&Token::Num(7)), "dígito");
        assert_eq!(classify(&Token::Num(42)), "número");
    }

    #[test]
    fn operators_and_parens() {
        assert_eq!(classify(&Token::Op('-')), "aditivo");
        assert_eq!(classify(&Token::Op('%')), "multiplicativo");
        assert_eq!(classify(&Token::Op('^')), "operador desconhecido");
        assert_eq!(classify(&Token::LParen), "parêntese");
        assert_eq!(classify(&Token::RParen), "parêntese");
    }

    #[test]
    fn identifiers() {
        assert_eq!(classify(&Token::Ident(String::from("_tmp"))), "privado");
        assert_eq!(classify(&Token::Ident(String::from("x"))), "identificador");
    }

    #[test]
    fn descriptions() {
        assert_eq!(describe(&Token::Num(5)), "o dígito 5");
        assert_eq!(describe(&Token::Num(50)), "o número 50");
        assert_eq!(describe(&Token::Op('*')), "o operador *");
        assert_eq!(describe(&Token::Op('^')), "o símbolo '^'");
        assert_eq!(describe(&Token::Ident(String::from("y"))), "o nome y");
    }
}

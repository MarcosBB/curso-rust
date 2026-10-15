//! Cada função foi reescrita sem `match`, `if let` nem laço explícito.

use std::num::ParseIntError;

/// 1. O número, ou 0 se não for número.
pub fn parse_or_zero(s: &str) -> i32 {
    s.parse().unwrap_or_default()
}

/// 2. O dobro do número.
pub fn parse_double(s: &str) -> Result<i32, ParseIntError> {
    s.parse::<i32>().map(|n| n * 2)
}

/// 3. O número, com uma mensagem de erro em português.
pub fn parse_with_message(s: &str) -> Result<i32, String> {
    s.parse().map_err(|e| format!("'{s}' não é um número: {e}"))
}

/// 4. A primeira palavra do texto.
pub fn first_word(s: &str) -> Result<&str, String> {
    s.split_whitespace()
        .next()
        .ok_or_else(|| "texto vazio".to_string())
}

/// 5. O número, se for um.
pub fn parse_optional(s: &str) -> Option<i32> {
    s.parse().ok()
}

/// 6. Um número estritamente positivo.
pub fn parse_positive(s: &str) -> Result<u32, String> {
    s.parse::<u32>().map_err(|e| e.to_string()).and_then(|n| {
        if n > 0 {
            Ok(n)
        } else {
            Err("zero não é positivo".to_string())
        }
    })
}

/// 7. Todos os números separados por vírgula, ou o primeiro erro.
pub fn parse_all(s: &str) -> Result<Vec<i32>, ParseIntError> {
    s.split(',').map(|item| item.trim().parse()).collect()
}

/// 8. Um campo opcional que, se presente, precisa ser um número.
pub fn parse_field(field: Option<&str>) -> Result<Option<i32>, ParseIntError> {
    field.map(|s| s.parse()).transpose()
}

/// 9. Maior de idade, se a idade for conhecida.
pub fn is_adult(age: Option<u32>) -> bool {
    age.is_some_and(|a| a >= 18)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn t1_parse_or_zero() {
        assert_eq!(parse_or_zero("12"), 12);
        assert_eq!(parse_or_zero("x"), 0);
    }

    #[test]
    fn t2_parse_double() {
        assert_eq!(parse_double("21"), Ok(42));
        assert!(parse_double("x").is_err());
    }

    #[test]
    fn t3_parse_with_message() {
        assert_eq!(parse_with_message("7"), Ok(7));
        assert_eq!(
            parse_with_message("sete"),
            Err("'sete' não é um número: invalid digit found in string".to_string())
        );
    }

    #[test]
    fn t4_first_word() {
        assert_eq!(first_word("  olá mundo"), Ok("olá"));
        assert_eq!(first_word("   "), Err("texto vazio".to_string()));
    }

    #[test]
    fn t5_parse_optional() {
        assert_eq!(parse_optional("5"), Some(5));
        assert_eq!(parse_optional("cinco"), None);
    }

    #[test]
    fn t6_parse_positive() {
        assert_eq!(parse_positive("3"), Ok(3));
        assert_eq!(parse_positive("0"), Err("zero não é positivo".to_string()));
        assert_eq!(
            parse_positive("-3"),
            Err("invalid digit found in string".to_string())
        );
    }

    #[test]
    fn t7_parse_all() {
        assert_eq!(parse_all("1, 2,3"), Ok(vec![1, 2, 3]));
        assert!(parse_all("1,x,3").is_err());
    }

    #[test]
    fn t8_parse_field() {
        assert_eq!(parse_field(None), Ok(None));
        assert_eq!(parse_field(Some("8")), Ok(Some(8)));
        assert!(parse_field(Some("oito")).is_err());
    }

    #[test]
    fn t9_is_adult() {
        assert!(is_adult(Some(30)));
        assert!(!is_adult(Some(10)));
        assert!(!is_adult(None));
    }
}

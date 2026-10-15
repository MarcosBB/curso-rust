pub fn fizzbuzz(n: u32) -> String {
    match (n % 3, n % 5) {
        (0, 0) => String::from("FizzBuzz"),
        (0, _) => String::from("Fizz"),
        (_, 0) => String::from("Buzz"),
        _ => n.to_string(),
    }
}

pub fn sequencia(ate: u32) -> Vec<String> {
    let mut saida = Vec::new();
    for n in 1..=ate {
        saida.push(fizzbuzz(n));
    }
    saida
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn numeros_comuns() {
        assert_eq!(fizzbuzz(1), "1");
        assert_eq!(fizzbuzz(7), "7");
    }

    #[test]
    fn multiplos() {
        assert_eq!(fizzbuzz(3), "Fizz");
        assert_eq!(fizzbuzz(10), "Buzz");
        assert_eq!(fizzbuzz(15), "FizzBuzz");
        assert_eq!(fizzbuzz(0), "FizzBuzz");
    }

    #[test]
    fn sequencia_ate_cinco() {
        assert_eq!(sequencia(5), vec!["1", "2", "Fizz", "4", "Buzz"]);
    }

    #[test]
    fn sequencia_vazia() {
        assert!(sequencia(0).is_empty());
    }
}

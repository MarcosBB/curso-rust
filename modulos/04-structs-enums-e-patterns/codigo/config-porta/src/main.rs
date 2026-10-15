//! Lição 07: `if let`, `while let`, `let...else`, let chains,
//! `if let` guards e `matches!`.

use std::collections::HashMap;

#[derive(Debug)]
enum Cmd {
    Set(String),
    Reset,
}

/// `let...else`: extrai o valor ou sai cedo.
fn parse_age(s: &str) -> Option<u32> {
    let Ok(n) = s.trim().parse::<u32>() else {
        return None;
    };
    if n > 150 {
        return None;
    }
    Some(n)
}

/// Let chain (edition 2024): vários testes encadeados com `&&`.
fn port_from(cfg: &HashMap<&str, &str>) -> u16 {
    if let Some(p) = cfg.get("port")
        && let Ok(port) = p.parse::<u16>()
        && port >= 1024
    {
        port
    } else {
        8080
    }
}

/// `if let` guard (Rust 1.95): o braço só vale se o segundo casamento der certo.
fn run(cmd: &Cmd) -> String {
    match cmd {
        Cmd::Set(v)
            if let Ok(n) = v.parse::<i32>()
                && n > 0 =>
        {
            format!("valor = {n}")
        }
        Cmd::Set(v) => format!("valor inválido: {v:?}"),
        Cmd::Reset => String::from("zerado"),
    }
}

fn is_small_digit(x: Option<i32>) -> bool {
    matches!(x, Some(1..=9))
}

fn main() {
    let level = Some(3);
    if let Some(l) = level {
        println!("nível {l}");
    } else {
        println!("sem nível");
    }

    let mut stack = vec![1, 2, 3];
    while let Some(top) = stack.pop() {
        print!("{top} ");
    }
    println!();

    println!(
        "{:?} {:?} {:?}",
        parse_age(" 42 "),
        parse_age("abc"),
        parse_age("200")
    );

    let mut cfg = HashMap::new();
    cfg.insert("port", "3000");
    println!("{}", port_from(&cfg));
    cfg.insert("port", "80");
    println!("{}", port_from(&cfg));

    for c in [
        Cmd::Set(String::from("7")),
        Cmd::Set(String::from("-2")),
        Cmd::Reset,
    ] {
        println!("{c:?}: {}", run(&c));
    }

    println!("{} {}", is_small_digit(Some(5)), is_small_digit(Some(42)));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ages() {
        assert_eq!(parse_age("42"), Some(42));
        assert_eq!(parse_age("x"), None);
        assert_eq!(parse_age("151"), None);
    }

    #[test]
    fn ports() {
        let mut cfg = HashMap::new();
        assert_eq!(port_from(&cfg), 8080);
        cfg.insert("port", "abc");
        assert_eq!(port_from(&cfg), 8080);
        cfg.insert("port", "80");
        assert_eq!(port_from(&cfg), 8080);
        cfg.insert("port", "5432");
        assert_eq!(port_from(&cfg), 5432);
    }

    #[test]
    fn commands() {
        assert_eq!(run(&Cmd::Set(String::from("10"))), "valor = 10");
        assert_eq!(run(&Cmd::Set(String::from("0"))), "valor inválido: \"0\"");
        assert_eq!(run(&Cmd::Reset), "zerado");
    }

    #[test]
    fn small_digit() {
        assert!(is_small_digit(Some(9)));
        assert!(!is_small_digit(Some(0)));
        assert!(!is_small_digit(None));
    }
}

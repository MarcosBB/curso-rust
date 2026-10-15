#[derive(Debug, Clone, PartialEq)]
pub enum Command {
    Set { key: String, value: i64 },
    Get(String),
    Del(String),
    Quit,
}

pub fn parse_command(line: &str) -> Option<Command> {
    let parts: Vec<&str> = line.split_whitespace().collect();
    match parts.as_slice() {
        ["set", key, value] if let Ok(value) = value.parse::<i64>() => Some(Command::Set {
            key: key.to_string(),
            value,
        }),
        ["get", key] => Some(Command::Get(key.to_string())),
        ["del", key] => Some(Command::Del(key.to_string())),
        ["quit" | "exit"] => Some(Command::Quit),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn set() {
        assert_eq!(
            parse_command("set x 10"),
            Some(Command::Set {
                key: String::from("x"),
                value: 10
            })
        );
        assert_eq!(
            parse_command("  set   y   -3 "),
            Some(Command::Set {
                key: String::from("y"),
                value: -3
            })
        );
    }

    #[test]
    fn set_rejects_bad_numbers_and_arity() {
        assert_eq!(parse_command("set x dez"), None);
        assert_eq!(parse_command("set x"), None);
        assert_eq!(parse_command("set x 1 2"), None);
    }

    #[test]
    fn get_del_quit() {
        assert_eq!(
            parse_command("get x"),
            Some(Command::Get(String::from("x")))
        );
        assert_eq!(
            parse_command("del x"),
            Some(Command::Del(String::from("x")))
        );
        assert_eq!(parse_command("quit"), Some(Command::Quit));
        assert_eq!(parse_command("exit"), Some(Command::Quit));
    }

    #[test]
    fn garbage() {
        assert_eq!(parse_command(""), None);
        assert_eq!(parse_command("get"), None);
        assert_eq!(parse_command("SET x 1"), None);
        assert_eq!(parse_command("quit agora"), None);
    }
}

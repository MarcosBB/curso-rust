#[derive(Debug)]
pub struct User {
    pub name: String,
    pub email: Option<String>,
    pub age: Option<u32>,
}

/// `let...else` achata a pirâmide: cada caso "ruim" sai cedo,
/// e o caminho feliz fica sem indentação.
pub fn contact_line(user: &Option<User>) -> String {
    let Some(u) = user else {
        return String::from("ninguém");
    };
    let Some(email) = &u.email else {
        return format!("{} (sem e-mail)", u.name);
    };
    match u.age {
        Some(age) if age >= 18 => format!("{} <{email}>", u.name),
        Some(_) => format!("{} (menor de idade)", u.name),
        None => format!("{} <{email}> (idade desconhecida)", u.name),
    }
}

/// Sem `mut` no padrão: `age` sai como `&u32`, e a soma funciona direto.
pub fn next_age(entry: &(String, u32)) -> u32 {
    let (_, age) = entry;
    age + 1
}

/// Let chain: três testes encadeados num `if` só.
pub fn parse_entry(line: &str) -> Option<(String, u32)> {
    if let Some((name, age)) = line.split_once('=')
        && !name.trim().is_empty()
        && let Ok(age) = age.trim().parse::<u32>()
    {
        Some((name.trim().to_string(), age))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn user(email: Option<&str>, age: Option<u32>) -> Option<User> {
        Some(User {
            name: String::from("Ana"),
            email: email.map(String::from),
            age,
        })
    }

    #[test]
    fn contact_lines() {
        assert_eq!(contact_line(&None), "ninguém");
        assert_eq!(contact_line(&user(None, Some(30))), "Ana (sem e-mail)");
        assert_eq!(
            contact_line(&user(Some("a@x.com"), Some(30))),
            "Ana <a@x.com>"
        );
        assert_eq!(
            contact_line(&user(Some("a@x.com"), Some(12))),
            "Ana (menor de idade)"
        );
        assert_eq!(
            contact_line(&user(Some("a@x.com"), None)),
            "Ana <a@x.com> (idade desconhecida)"
        );
    }

    #[test]
    fn ages() {
        let entry = (String::from("Bia"), 41);
        assert_eq!(next_age(&entry), 42);
        assert_eq!(entry.1, 41);
    }

    #[test]
    fn entries() {
        assert_eq!(parse_entry("Bia = 41"), Some((String::from("Bia"), 41)));
        assert_eq!(parse_entry(" = 41"), None);
        assert_eq!(parse_entry("Bia = x"), None);
        assert_eq!(parse_entry("Bia 41"), None);
    }
}

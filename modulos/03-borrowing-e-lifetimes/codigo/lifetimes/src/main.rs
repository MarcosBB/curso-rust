//! Exemplos das lições 04 a 06: lifetimes em funções, elisão, `'_`,
//! structs que guardam referências e `'static`.

/// O retorno vale enquanto **os dois** argumentos valerem.
fn longest<'a>(x: &'a str, y: &'a str) -> &'a str {
    if x.len() >= y.len() { x } else { y }
}

/// Só `x` se relaciona com o retorno; `y` pode morrer antes.
fn first<'a>(x: &'a str, _y: &str) -> &'a str {
    x
}

/// Sem referência de onde emprestar: devolva um valor com posse.
fn greeting(name: &str) -> String {
    format!("olá, {name}")
}

/// Literais são `&'static str`: estão gravados no binário.
fn status(ok: bool) -> &'static str {
    if ok { "ok" } else { "falhou" }
}

struct Stack {
    items: Vec<i32>,
}

impl Stack {
    /// `'_` deixa visível que o iterador empresta de `self`.
    fn iter(&self) -> std::slice::Iter<'_, i32> {
        self.items.iter()
    }

    /// Regra 3 da elisão: o retorno empresta de `self`.
    fn top(&self) -> Option<&i32> {
        self.items.last()
    }
}

/// Uma visão temporária de um texto que pertence a outra pessoa.
struct Excerpt<'a> {
    part: &'a str,
}

impl<'a> Excerpt<'a> {
    /// Elidido: o retorno fica preso ao empréstimo de `self`.
    fn part(&self) -> &str {
        self.part
    }

    /// Explícito: o retorno fica preso ao texto original (`'a`).
    fn part_long(&self) -> &'a str {
        self.part
    }
}

/// Funciona porque `part_long` devolve `&'a str`, e não `&str` ligado a `e`.
fn first_sentence(text: &str) -> &str {
    let e = Excerpt {
        part: text.split('.').next().unwrap_or(""),
    };
    e.part_long()
}

fn main() {
    let string1 = String::from("uma string longa");
    {
        let string2 = String::from("xyz");
        let result = longest(string1.as_str(), string2.as_str());
        println!("a maior é: {result}");
    }

    let a = String::from("fica");
    let r;
    {
        let b = String::from("morre");
        r = first(&a, &b);
    }
    println!("{r}");

    println!("{} / {} / {}", greeting("Ana"), status(true), status(false));

    let s = Stack {
        items: vec![1, 2, 3],
    };
    let total: i32 = s.iter().sum();
    println!("{total} {:?}", s.top());

    let novel = String::from("Chamai-me Ismael. Alguns anos atrás...");
    let e = Excerpt { part: &novel[..16] };
    println!("{} | {}", e.part(), first_sentence(&novel));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn longest_picks_first_on_tie() {
        assert_eq!(longest("ab", "cd"), "ab");
        assert_eq!(longest("a", "cd"), "cd");
    }

    #[test]
    fn sentence_outlives_excerpt() {
        let text = String::from("Um. Dois.");
        let s = first_sentence(&text);
        assert_eq!(s, "Um");
    }
}

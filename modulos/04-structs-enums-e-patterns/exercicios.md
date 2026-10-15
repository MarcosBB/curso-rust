# Exercícios — Módulo 04

Soluções em [`solucoes/04-structs-enums-e-patterns`](../../solucoes/04-structs-enums-e-patterns/).

Todos os exercícios são bibliotecas com testes. Para cada um, crie um projeto com `cargo new --lib nome`, acrescente `rust-version = "1.96"` no `Cargo.toml`, cole o código no `src/lib.rs` e rode `cargo test`. O objetivo é sempre o mesmo: **todos os testes passando**, sem avisos no `cargo clippy`.

## 1. Faça compilar: a forma nova (fácil)

Alguém acrescentou a variante `Triangle` (definida pelos três lados) ao enum `Shape`, mas não atualizou as funções. O código não compila. Siga os erros do compilador e complete as três funções. Os testes já cobrem o triângulo.

Para a área, use a fórmula de Heron: com `p = (a + b + c) / 2`, a área é `√(p·(p−a)·(p−b)·(p−c))`.

```rust
use std::f64::consts::PI;

#[derive(Debug, Clone, PartialEq)]
pub enum Shape {
    Circle { r: f64 },
    Square { side: f64 },
    Rect { w: f64, h: f64 },
    Triangle { a: f64, b: f64, c: f64 },
}

pub fn area(s: &Shape) -> f64 {
    match s {
        Shape::Circle { r } => PI * r * r,
        Shape::Square { side } => side * side,
        Shape::Rect { w, h } => w * h,
    }
}

pub fn perimeter(s: &Shape) -> f64 {
    match s {
        Shape::Circle { r } => 2.0 * PI * r,
        Shape::Square { side } => 4.0 * side,
        Shape::Rect { w, h } => 2.0 * (w + h),
    }
}

pub fn name(s: &Shape) -> &'static str {
    match s {
        Shape::Circle { .. } => "círculo",
        Shape::Square { .. } => "quadrado",
        Shape::Rect { .. } => "retângulo",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-9
    }

    #[test]
    fn circle() {
        let s = Shape::Circle { r: 1.0 };
        assert!(close(area(&s), PI));
        assert!(close(perimeter(&s), 2.0 * PI));
        assert_eq!(name(&s), "círculo");
    }

    #[test]
    fn square_and_rect() {
        assert!(close(area(&Shape::Square { side: 3.0 }), 9.0));
        assert!(close(perimeter(&Shape::Rect { w: 2.0, h: 5.0 }), 14.0));
        assert_eq!(name(&Shape::Rect { w: 2.0, h: 5.0 }), "retângulo");
    }

    #[test]
    fn triangle() {
        let t = Shape::Triangle {
            a: 3.0,
            b: 4.0,
            c: 5.0,
        };
        assert!(close(area(&t), 6.0));
        assert!(close(perimeter(&t), 12.0));
        assert_eq!(name(&t), "triângulo");
    }
}
```

**Esperado:** três erros E0004 antes da correção; depois, `cargo test` com 3 testes passando.

<details>
<summary>Dica</summary>

O `help:` de cada E0004 mostra onde o braço novo entra. Não use `_ =>`: liste a variante, para que o compilador continue avisando quando surgir uma quinta forma. Raiz quadrada de um `f64`: `x.sqrt()`.

</details>

## 2. Newtype validado (fácil)

Implemente `Port`, uma porta de rede que só aceita valores a partir de 1024 (lição 08).

```rust
pub mod net {
    #[derive(Debug, Clone, Copy, PartialEq)]
    pub struct Port(u16);

    impl Port {
        /// Aceita só portas não privilegiadas (>= 1024).
        pub fn new(value: u16) -> Option<Port> {
            todo!()
        }

        pub fn get(self) -> u16 {
            todo!()
        }

        /// Faixa dinâmica/efêmera da IANA: 49152 a 65535.
        pub fn is_ephemeral(self) -> bool {
            todo!()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::net::Port;

    #[test]
    fn rejects_privileged_ports() {
        assert_eq!(Port::new(0), None);
        assert_eq!(Port::new(80), None);
        assert_eq!(Port::new(1023), None);
    }

    #[test]
    fn accepts_the_rest() {
        assert_eq!(Port::new(1024).map(Port::get), Some(1024));
        assert_eq!(Port::new(65535).map(Port::get), Some(65535));
    }

    #[test]
    fn ephemeral() {
        assert!(!Port::new(8080).unwrap().is_ephemeral());
        assert!(Port::new(49152).unwrap().is_ephemeral());
    }
}
```

**Esperado:** 3 testes passando.

Depois, responda: o que acontece se você escrever `let p = Port(80);` dentro do `mod tests`? E `Port::new(8080).unwrap().0`? Por que isso é bom?

<details>
<summary>Dica</summary>

`new` é um `if` que devolve `Some(Port(value))` ou `None`. Dentro do módulo `net`, você pode usar o construtor `Port(...)` e o campo `.0`. Fora dele, não.

</details>

## 3. Classificador de tokens (médio)

Implemente as duas funções usando **só `match`**, com literais, faixas, `|`, guards e `@` (lição 05). Os testes dizem o que cada caso deve devolver.

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Num(i64),
    Op(char),
    LParen,
    RParen,
    Ident(String),
}

pub fn classify(t: &Token) -> &'static str {
    todo!()
}

pub fn describe(t: &Token) -> String {
    todo!()
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
```

**Esperado:** 4 testes passando.

<details>
<summary>Dica</summary>

A ordem dos braços importa: `Token::Num(0)` precisa vir antes de um braço genérico de números. Para os negativos, um guard (`Token::Num(n) if *n < 0`). Em `describe`, `Token::Num(n @ 1..=9)` casa com a faixa e guarda o valor. `'^'` formatado com `{:?}` vira `'^'`, com aspas.

</details>

## 4. Modernize o código (médio)

O código abaixo foi escrito para a edition 2021. Crie o projeto com `edition = "2021"` no `Cargo.toml` e confirme que os testes passam.

```rust
#[derive(Debug)]
pub struct User {
    pub name: String,
    pub email: Option<String>,
    pub age: Option<u32>,
}

pub fn contact_line(user: &Option<User>) -> String {
    match user {
        Some(ref u) => match u.email {
            Some(ref e) => {
                if let Some(age) = u.age {
                    if age >= 18 {
                        format!("{} <{}>", u.name, e)
                    } else {
                        format!("{} (menor de idade)", u.name)
                    }
                } else {
                    format!("{} <{}> (idade desconhecida)", u.name, e)
                }
            }
            None => format!("{} (sem e-mail)", u.name),
        },
        None => String::from("ninguém"),
    }
}

pub fn next_age(entry: &(String, u32)) -> u32 {
    let (_, mut age) = entry;
    age += 1;
    age
}

pub fn parse_entry(line: &str) -> Option<(String, u32)> {
    match line.split_once('=') {
        Some((name, age)) => {
            let name = name.trim();
            if !name.is_empty() {
                match age.trim().parse::<u32>() {
                    Ok(age) => Some((name.to_string(), age)),
                    Err(_) => None,
                }
            } else {
                None
            }
        }
        None => None,
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
```

Agora:

1. Mude para `edition = "2024"` e rode `cargo build`. Quais erros aparecem, e por quê? (Lição 06.)
2. Volte para `edition = "2021"`, faça um commit (ou uma cópia do projeto) e rode `cargo fix --edition`. Depois mude para `edition = "2024"`. Compare o que o `cargo fix` mudou.
3. Reescreva as três funções à mão, no estilo moderno: sem `ref`, sem `mut` em padrão que empresta, com `let...else` e let chains no lugar das pirâmides de `if`/`match` (lição 07).

**Esperado:** os mesmos 3 testes passando, agora em edition 2024, sem avisos do clippy, e com `contact_line` e `parse_entry` sem nenhum nível de aninhamento a mais que o necessário.

<details>
<summary>Dica</summary>

Em `contact_line`, comece com `let Some(u) = user else { return ...; };` e faça o mesmo com o e-mail (`let Some(email) = &u.email else { ... };`). O que sobra é um `match` em `u.age` com um guard. Em `parse_entry`, uma let chain com três elos: `split_once`, o nome não vazio e o `parse`.

</details>

## 5. Mini-REPL com slice patterns (médio)

Implemente `parse_command`, que interpreta uma linha de um REPL de chave/valor: `set x 10`, `get x`, `del x`, `quit` (ou `exit`). Qualquer outra coisa devolve `None`, inclusive `set` com um valor que não é número.

A linha já foi quebrada em palavras para você. Use um `match` sobre `parts.as_slice()` com padrões de slice (`["get", key]`) e um `if let` guard para o número (lição 07).

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum Command {
    Set { key: String, value: i64 },
    Get(String),
    Del(String),
    Quit,
}

pub fn parse_command(line: &str) -> Option<Command> {
    let parts: Vec<&str> = line.split_whitespace().collect();
    todo!()
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
```

**Esperado:** 4 testes passando.

<details>
<summary>Dica</summary>

`parts.as_slice()` tem tipo `&[&str]`, e um padrão como `["set", key, value]` casa com slices de exatamente três elementos, o primeiro igual a `"set"`. O guard fica `if let Ok(value) = value.parse::<i64>()`. Dentro do braço, `key` é um `&&str`: `key.to_string()` resolve.

</details>

## 6. Máquina de estados de um pedido (médio/desafio)

Um pedido passa por carrinho → pago → enviado, e pode ser cancelado antes do envio. Cada estado guarda só os dados que fazem sentido nele (lição 08). Implemente os métodos. As transições consomem `self` e, quando não se aplicam, devolvem o pedido como estava.

```rust
#[derive(Debug, Clone, PartialEq)]
pub struct Item {
    pub name: String,
    pub price_cents: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Order {
    Cart { items: Vec<Item> },
    Paid { items: Vec<Item>, total_cents: u64 },
    Shipped { tracking: String },
    Cancelled { reason: String },
}

impl Order {
    /// Um carrinho vazio.
    pub fn new() -> Order {
        todo!()
    }

    /// Só dá para adicionar itens no carrinho. Devolve `false` nos outros estados.
    pub fn add_item(&mut self, item: Item) -> bool {
        todo!()
    }

    /// Carrinho com itens vira `Paid`, com o total calculado. Qualquer outro caso fica como está.
    pub fn pay(self) -> Order {
        todo!()
    }

    /// Só um pedido pago pode ser enviado.
    pub fn ship(self, tracking: &str) -> Order {
        todo!()
    }

    /// Dá para cancelar antes do envio, nunca depois.
    pub fn cancel(self, reason: &str) -> Order {
        todo!()
    }

    /// "carrinho vazio", "carrinho", "pago", "enviado" ou "cancelado".
    pub fn status(&self) -> &'static str {
        todo!()
    }

    /// O total, só no estado `Paid`.
    pub fn total_cents(&self) -> Option<u64> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn item(name: &str, price_cents: u64) -> Item {
        Item {
            name: name.to_string(),
            price_cents,
        }
    }

    #[test]
    fn happy_path() {
        let mut o = Order::new();
        assert_eq!(o.status(), "carrinho vazio");
        assert!(o.add_item(item("livro", 5000)));
        assert!(o.add_item(item("caneta", 350)));
        assert_eq!(o.status(), "carrinho");

        let o = o.pay();
        assert_eq!(o.status(), "pago");
        assert_eq!(o.total_cents(), Some(5350));

        let o = o.ship("BR123");
        assert_eq!(
            o,
            Order::Shipped {
                tracking: String::from("BR123")
            }
        );
    }

    #[test]
    fn empty_cart_cannot_be_paid() {
        let o = Order::new().pay();
        assert_eq!(o.status(), "carrinho vazio");
        assert_eq!(o.total_cents(), None);
    }

    #[test]
    fn cannot_ship_unpaid_order() {
        let mut o = Order::new();
        o.add_item(item("livro", 5000));
        let o = o.ship("BR123");
        assert_eq!(o.status(), "carrinho");
    }

    #[test]
    fn cannot_add_items_after_paying() {
        let mut o = Order::new();
        o.add_item(item("livro", 5000));
        let mut o = o.pay();
        assert!(!o.add_item(item("caneta", 350)));
        assert_eq!(o.total_cents(), Some(5000));
    }

    #[test]
    fn cancel_before_shipping_only() {
        let mut o = Order::new();
        o.add_item(item("livro", 5000));
        let cancelled = o.clone().pay().cancel("desisti");
        assert_eq!(cancelled.status(), "cancelado");

        let shipped = o.pay().ship("BR1").cancel("tarde demais");
        assert_eq!(shipped.status(), "enviado");
    }
}
```

**Esperado:** 5 testes passando e nenhum aviso do clippy.

<details>
<summary>Dica</summary>

O padrão `other => other` devolve o estado intacto. Em `pay`, um guard separa o carrinho vazio: `Order::Cart { items } if !items.is_empty() => ...`. Para somar os preços, um `for` com um acumulador resolve (com iteradores, `items.iter().map(|i| i.price_cents).sum()`, Módulo 07). Em `add_item`, `match self` com `self: &mut Order` dá acesso a `items` como `&mut Vec<Item>` (lição 06). O clippy vai pedir um `impl Default` por causa do `new()` sem argumentos: veja a solução, e as traits no Módulo 06.

</details>

**Para pensar:** nesta versão, `ship` num carrinho compila e só é ignorado em tempo de execução. Como seria possível fazer essa chamada nem compilar? (Resposta no Módulo 13: *typestate*.)

---

[Índice do módulo](README.md)

# Soluções — Módulo 04

Enunciados em [`modulos/04-structs-enums-e-patterns/exercicios.md`](../../modulos/04-structs-enums-e-patterns/exercicios.md).

Cada solução é um projeto Cargo (biblioteca) com os mesmos testes do enunciado. Para conferir:

```bash
cd ex1-formas
cargo test
```

## 1. Faça compilar: a forma nova

Projeto: [`ex1-formas/`](ex1-formas/src/lib.rs).

O compilador dá três E0004, um por função, todos com a mesma mensagem:

```text
error[E0004]: non-exhaustive patterns: `&Shape::Triangle { .. }` not covered
```

A correção é um braço `Shape::Triangle { a, b, c } => ...` em cada `match`. Na área, a fórmula de Heron:

```rust
Shape::Triangle { a, b, c } => {
    let p = (a + b + c) / 2.0;
    (p * (p - a) * (p - b) * (p - c)).sqrt()
}
```

O ponto do exercício é o processo: você acrescenta a variante, e o compilador lista **todos** os lugares que precisam mudar. Por isso o enunciado pede para não usar `_ =>`: com um curinga, a próxima variante passaria despercebida.

## 2. Newtype validado

Projeto: [`ex2-porta/`](ex2-porta/src/lib.rs).

```rust
pub fn new(value: u16) -> Option<Port> {
    if value >= 1024 {
        Some(Port(value))
    } else {
        None
    }
}
```

`get` devolve `self.0`, e `is_ephemeral` testa `self.0 >= 49152`. Como `Port` é `Copy`, os métodos podem receber `self` por valor sem custo nem perda.

**A pergunta.** O `mod tests` está fora do módulo `net`, então não enxerga o campo privado:

```text
error[E0423]: cannot initialize a tuple struct which contains private fields
  --> src/lib.rs:56:17
   |
56 |         let p = Port(80);
   |                 ^^^^
...
error[E0616]: field `0` of struct `Port` is private
  --> src/lib.rs:57:42
   |
57 |         let x = Port::new(8080).unwrap().0;
   |                                          ^ private field
```

É bom porque garante que **toda** `Port` passou por `new`. Uma função que recebe `Port` não precisa checar de novo se o valor é maior ou igual a 1024. O `help:` do E0423 sugere tornar o campo `pub`, e isso destruiria a garantia.

## 3. Classificador de tokens

Projeto: [`ex3-tokens/`](ex3-tokens/src/lib.rs).

```rust
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
```

Pontos a observar:

- **Ordem.** `Num(0)` vem antes do guard de negativos e da faixa; `Num(_)` fica por último, cobrindo o que sobrou (10 em diante).
- **`*n` no guard.** Como casamos `&Token`, o `n` é um `&i64` (lição 06). Na comparação `*n < 0`, desreferenciamos explicitamente.
- **Exaustividade.** Os braços com guard (`if *n < 0`, `if name.starts_with('_')`) não contam, por isso cada um tem um braço sem guard logo depois (`Num(_)`, `Ident(_)`).
- Em `describe`, `Token::Num(n @ 1..=9)` testa a faixa e guarda o valor, e `Token::Op(op @ ('+' | '-' | '*' | '/' | '%'))` faz o mesmo com alternativas. Os parênteses em volta das alternativas são obrigatórios.

## 4. Modernize o código

Projeto: [`ex4-modernize/`](ex4-modernize/src/lib.rs).

**Passo 1.** Com `edition = "2024"`, aparecem dois erros (o terceiro `ref`, em `match u.email { Some(ref e) ... }`, é válido: `u.email` não é uma referência, então o padrão é explícito):

```text
error: cannot explicitly borrow within an implicitly-borrowing pattern
  --> src/lib.rs:10:14
   |
10 |         Some(ref u) => match u.email {
   |              ^^^ explicit `ref` binding modifier not allowed when implicitly borrowing
...
error: cannot mutably bind by value within an implicitly-borrowing pattern
  --> src/lib.rs:29:13
   |
29 |     let (_, mut age) = entry;
   |             ^^^ `mut` binding modifier not allowed when implicitly borrowing
```

`user` é `&Option<User>`, então `Some(u)` já empresta, e o `ref` é redundante. `entry` é `&(String, u32)`, e na 2021 o `mut` fazia `age` sair como `u32` copiado, enquanto um binding sem `mut` sairia como `&u32`.

**Passo 2.** O `cargo fix --edition` faz duas mudanças mínimas:

```diff
-        Some(ref u) => match u.email {
+        Some(u) => match u.email {
...
-    let (_, mut age) = entry;
+    let &(_, mut age) = entry;
```

Compila e passa nos testes, mas não mexe na estrutura. Observe também que o `cargo fix` não altera a edition no `Cargo.toml`: isso é feito à mão.

**Passo 3.** A versão moderna:

```rust
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

pub fn next_age(entry: &(String, u32)) -> u32 {
    let (_, age) = entry;
    age + 1
}

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
```

- `contact_line` caiu de cinco níveis de aninhamento para um. Os casos "ruins" saem cedo com `let...else`, e os três casos de idade ficam lado a lado num `match`, o que facilita conferir que todos foram tratados.
- Em `next_age`, `age` é um `&u32`, e `age + 1` funciona porque a soma aceita referências a números. Outra forma igualmente boa: `entry.1 + 1`.
- Em `parse_entry`, a let chain troca dois `match` e um `if` aninhados por três elos com `&&`.

## 5. Mini-REPL com slice patterns

Projeto: [`ex5-comandos/`](ex5-comandos/src/lib.rs).

```rust
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
```

- Os padrões de slice verificam ao mesmo tempo o **tamanho** e o **conteúdo**: `["get", key]` só casa com exatamente duas palavras, a primeira sendo `"get"`. Por isso `"quit agora"` e `"set x 1 2"` caem no `_`.
- O `if let` guard (Rust 1.95) faz o `parse` e, se der certo, deixa `value: i64` disponível no braço. Ele "sombreia" o `value: &&str` do padrão. Se o `parse` falhar, o `match` segue e cai no `_`.
- Sem o `if let` guard, você precisaria casar `["set", key, value]` e fazer um segundo `match` (ou `let...else`) dentro do braço.

## 6. Máquina de estados de um pedido

Projeto: [`ex6-pedido/`](ex6-pedido/src/lib.rs).

Os trechos principais:

```rust
pub fn add_item(&mut self, item: Item) -> bool {
    match self {
        Order::Cart { items } => {
            items.push(item);
            true
        }
        _ => false,
    }
}

pub fn pay(self) -> Order {
    match self {
        Order::Cart { items } if !items.is_empty() => {
            let total_cents = items.iter().map(|i| i.price_cents).sum();
            Order::Paid { items, total_cents }
        }
        other => other,
    }
}

pub fn cancel(self, reason: &str) -> Order {
    match self {
        Order::Cart { .. } | Order::Paid { .. } => Order::Cancelled {
            reason: reason.to_string(),
        },
        other => other,
    }
}
```

- Em `add_item`, `self` é `&mut Order`. Pelo *match ergonomics*, `items` sai como `&mut Vec<Item>`, e o `push` altera o pedido no lugar.
- Em `pay`, o guard separa o carrinho vazio. Como `pay` recebe `self` por valor, o `items` do padrão é movido do carrinho para o `Paid`, sem cópia.
- `other => other` devolve intacto o estado em que a transição não se aplica. É isso que faz `ship` num carrinho e `cancel` num pedido enviado não terem efeito.
- O `impl Default for Order` existe porque o clippy (`new_without_default`) pede um `Default` quando há um `new()` sem argumentos. Implementar traits é assunto do Módulo 06.

**Para pensar.** Com *typestate*, `Cart`, `Paid` e `Shipped` seriam tipos diferentes. `ship` existiria só em `Paid`, e `cart.ship(...)` seria erro de compilação (método inexistente), em vez de uma chamada ignorada em tempo de execução. Veja o Módulo 13.

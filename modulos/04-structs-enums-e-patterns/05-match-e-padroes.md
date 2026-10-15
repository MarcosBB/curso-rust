# 05 — `match` e padrões

**Objetivos.** Ao final, você vai saber:

- por que o `match` é exaustivo e como usar isso para refatorar com segurança;
- a sintaxe de padrões: literais, faixas, `|`, `_`, `..`, desestruturação, slices, guards e `@`;
- a diferença entre padrões refutáveis e irrefutáveis;
- duas armadilhas clássicas: o nome solto que vira variável e o `match` que move o valor.

## Exaustividade: o compilador confere a lista

Você viu `match` no Módulo 01. A propriedade que importa agora é esta: um `match` precisa cobrir **todos** os valores possíveis. Acrescente uma variante `Triangle` ao `Shape` da lição 03 e compile:

```rust
// ❌ NÃO COMPILA
enum Shape {
    Circle { r: f64 },
    Rect { w: f64, h: f64 },
    Triangle { b: f64, h: f64 },
}

fn area(s: &Shape) -> f64 {
    match s {
        Shape::Circle { r } => std::f64::consts::PI * r * r,
        Shape::Rect { w, h } => w * h,
    }
}
```

```text
error[E0004]: non-exhaustive patterns: `&Shape::Triangle { .. }` not covered
  --> src/main.rs:8:11
   |
 8 |     match s {
   |           ^ pattern `&Shape::Triangle { .. }` not covered
   |
note: `Shape` defined here
  --> src/main.rs:1:6
   |
 1 | enum Shape {
   |      ^^^^^
...
 4 |     Triangle { b: f64, h: f64 },
   |     -------- not covered
   = note: the matched value is of type `&Shape`
help: ensure that all possible cases are being handled by adding a match arm with a wildcard pattern or an explicit pattern as shown
   |
10 ~         Shape::Rect { w, h } => w * h,
11 ~         &Shape::Triangle { .. } => todo!(),
   |
```

O compilador diz qual variante falta, onde ela foi declarada e sugere o braço. Num projeto grande, isso vira uma ferramenta de refatoração: acrescente a variante, compile, e a lista de erros é a lista de lugares que precisam de atenção. Em C++, um `switch` sem um `case` só gera aviso com `-Wswitch`, e o `match` do Python simplesmente não faz nada.

A exaustividade vale para qualquer tipo, inclusive inteiros com faixas:

```rust
// ❌ NÃO COMPILA
fn classify(n: i32) -> &'static str {
    match n {
        i32::MIN..0 => "negativo",
        0 => "zero",
        1..1000 => "pequeno",
    }
}
```

```text
error[E0004]: non-exhaustive patterns: `1000_i32..=i32::MAX` not covered
 --> src/main.rs:2:11
  |
2 |     match n {
  |           ^ pattern `1000_i32..=i32::MAX` not covered
```

O compilador calculou exatamente a faixa que ficou de fora.

> [!TIP]
> Evite `_ => ...` em enums que você controla. O curinga cobre as variantes futuras também, e aí o compilador não avisa quando você acrescentar uma. Prefira listar as variantes, mesmo que várias levem ao mesmo braço (`A | B => ...`).

## O catálogo de padrões

Os padrões são os mesmos em `match`, `let`, `if let`, parâmetros de função e `for`. Este exemplo reúne quase todos ([`codigo/formas`](codigo/formas/) tem mais):

```rust
#[derive(Debug)]
struct Point {
    x: i32,
    y: i32,
}

fn where_is(p: &Point) -> String {
    match p {
        Point { x: 0, y: 0 } => String::from("na origem"),          // literais nos campos
        Point { x: 0, y } => format!("no eixo y, em {y}"),          // literal + binding
        Point { x, y: 0 } => format!("no eixo x, em {x}"),
        Point { x, y } if x == y => format!("na diagonal, em {x}"), // guard
        Point { x, .. } => format!("em outro lugar, com x = {x}"),  // `..` ignora o resto
    }
}

fn age_group(age: u32) -> &'static str {
    match age {
        0 => "recém-nascido",
        1..=12 => "criança",        // faixa inclusiva
        13..=17 => "adolescente",
        18 | 19 => "recém-adulto",  // alternativas
        _ => "adulto",              // curinga
    }
}
```

```text
Point { x: 0, y: 0 } está na origem
Point { x: 0, y: 7 } está no eixo y, em 7
Point { x: 3, y: 3 } está na diagonal, em 3
Point { x: 5, y: 1 } está em outro lugar, com x = 5
recém-nascido adolescente recém-adulto
```

| Padrão | Exemplo | Casa com |
|---|---|---|
| literal | `0`, `'a'`, `"sim"`, `true` | aquele valor exato |
| binding | `x` | qualquer valor, e dá a ele o nome `x` |
| curinga | `_` | qualquer valor, sem criar variável |
| faixa | `1..=12`, `'a'..='z'`, `i32::MIN..0` | valores na faixa (`..` exclui o fim) |
| alternativa | `18 \| 19`, `'+' \| '-'` | qualquer uma das opções |
| struct | `Point { x, y: 0 }` | struct com aqueles campos |
| tupla | `(a, _, c)` | tupla de 3 elementos |
| variante | `Some(n)`, `Shape::Rect { w, h }` | aquela variante, desestruturada |
| resto | `Point { x, .. }`, `(first, ..)` | ignora os campos ou elementos restantes |
| slice | `[first, .., last]`, `[head, rest @ ..]` | slices e arrays por formato |
| guard | `x if x > 0` | o padrão, **e** a condição verdadeira |
| `@` | `n @ 1..=9` | o padrão, guardando o valor em `n` |

Os padrões se aninham à vontade: `Some(Shape::Rect { w, h: 0.0 })` é válido.

### Slices

Padrões de slice testam o formato de uma sequência. São ótimos para processar listas e argumentos:

```rust
fn describe(nums: &[i32]) -> String {
    match nums {
        [] => String::from("vazia"),
        [x] => format!("um elemento: {x}"),
        [first, .., last] if first == last => format!("começa e termina com {first}"),
        [first, rest @ ..] => format!("{first} e mais {} elementos", rest.len()),
    }
}
```

```text
vazia
um elemento: 4
começa e termina com 1
1 e mais 2 elementos
```

O `rest @ ..` dá o nome `rest` ao pedaço que sobrou, que é outro slice. Equivale ao `case [first, *rest]:` do Python.

### `@`: testar e guardar ao mesmo tempo

Num padrão como `'0'..='9'`, você sabe que casou, mas não tem uma variável com o valor. O `@` resolve:

```rust
fn digit(c: char) -> String {
    match c {
        d @ '0'..='9' => format!("dígito {d}"),
        op @ ('+' | '-' | '*' | '/') => format!("operador {op}"),
        _ => String::from("outro"),
    }
}
```

```text
dígito 7 | operador * | outro
```

### Guards

Um guard (`if condição` depois do padrão) acrescenta uma condição que padrões não expressam, como comparar dois campos (`x == y`). Num padrão com `|`, o guard vale para **todas** as alternativas: `4 | 5 | 6 if y` significa `(4 | 5 | 6) if y`.

Os guards têm um custo: o compilador não analisa a condição, então **braços com guard não contam para a exaustividade**:

```rust
// ❌ NÃO COMPILA
fn sign(n: i32) -> &'static str {
    match n {
        x if x < 0 => "negativo",
        x if x > 0 => "positivo",
        0 => "zero",
    }
}
```

```text
error[E0004]: non-exhaustive patterns: `i32::MIN..=-1_i32` and `1_i32..=i32::MAX` not covered
 --> src/main.rs:2:11
  |
2 |     match n {
  |           ^ patterns `i32::MIN..=-1_i32` and `1_i32..=i32::MAX` not covered
  |
  = note: the matched value is of type `i32`
  = note: match arms with guards don't count towards exhaustivity
```

Nós sabemos que os três braços cobrem tudo, mas o compilador não lê o `x < 0`. Use faixas (`i32::MIN..0`, `0`, `1..`) quando der, ou troque o último guard por um `_`.

## Desestruturação fora do `match`

Os mesmos padrões funcionam em `let` e em parâmetros, desde que não possam falhar:

```rust
let (a, (b, c)) = (1, (2.5, "três"));
let Point { x: px, y: py } = Point { x: 10, y: 20 };

fn print_coords(&(x, y): &(i32, i32)) {
    println!("({x}, {y})");
}
```

`Point { x: px, .. }` lê "pegue o campo `x` e chame de `px`". Se o nome da variável é igual ao do campo, basta `Point { x, .. }`.

Esquecer um campo numa desestruturação é erro, e as sugestões mostram as três saídas:

```text
error[E0027]: pattern does not mention field `y`
 --> src/main.rs:8:9
  |
8 |     let Point { x } = p;
  |         ^^^^^^^^^^^ missing field `y`
  |
help: include the missing field in the pattern
  |
8 |     let Point { x, y } = p;
  |                  +++
help: if you don't care about this missing field, you can explicitly ignore it
  |
8 |     let Point { x, y: _ } = p;
  |                  ++++++
help: or always ignore missing fields here
  |
8 |     let Point { x, .. } = p;
  |                  ++++
```

## Refutável ou irrefutável

Um padrão é **irrefutável** se casa com qualquer valor do tipo (`x`, `(a, b)`, `Point { x, y }`) e **refutável** se pode falhar (`Some(x)`, `0`, `[first, ..]`). O `let` só aceita os irrefutáveis, porque não há o que fazer se o padrão não casar:

```rust
// ❌ NÃO COMPILA
let opt: Option<i32> = Some(3);
let Some(x) = opt;
```

```text
error[E0005]: refutable pattern in local binding
 --> src/main.rs:3:9
  |
3 |     let Some(x) = opt;
  |         ^^^^^^^ pattern `None` not covered
  |
  = note: `let` bindings require an "irrefutable pattern", like a `struct` or an `enum` with only one variant
...
help: you might want to use `let...else` to handle the variant that isn't matched
  |
3 |     let Some(x) = opt else { todo!() };
  |                       ++++++++++++++++
```

A sugestão (`let...else`) é assunto da lição 07. O caso contrário também existe: `if let x = 5` gera o aviso `irrefutable_let_patterns`, porque o `if` nunca vai falhar.

## Python 3.10: muito parecido, com duas diferenças

O `match` do Python foi inspirado nas mesmas ideias, e a tradução é quase direta:

| Python | Rust |
|---|---|
| `case Point(x=0, y=y):` | `Point { x: 0, y } =>` |
| `case [first, *rest]:` | `[first, rest @ ..] =>` |
| `case 1 \| 2:` | `1 \| 2 =>` |
| `case x if x > 0:` | `x if x > 0 =>` |
| `case _:` | `_ =>` |

As diferenças: o `match` do Python **não é exaustivo** (sem um `case` que case, nada acontece) e **não é expressão** (não devolve valor). Em Rust, `let s = match ... { ... };` é comum.

## Armadilhas comuns

**Nome solto é variável nova, não comparação.** Este código parece comparar `n` com `max`:

```rust
// ⚠️ compila, mas está errado
let max = 10;
let n = 3;
match n {
    max => println!("é o máximo"),
    _ => println!("outro"),
}
```

```text
warning: unreachable pattern
 --> src/main.rs:6:9
  |
5 |         max => println!("é o máximo"),
  |         --- matches any value
6 |         _ => println!("outro"),
  |         ^ no value can reach this
  |
note: there is a binding of the same name; if you meant to pattern match against the value of that binding, that is a feature of constants that is not available for `let` bindings
```

O `max` do padrão é uma variável **nova**, que casa com qualquer valor. O programa imprime "é o máximo" para `n = 3`. O Python tem exatamente a mesma armadilha: `case MAX:` também captura em vez de comparar. A correção em Rust é usar uma `const` (`const MAX: i32 = 10;`): o nome de uma constante, num padrão, é comparado e não capturado. A outra saída é um guard (`x if x == max`). Em Python, um nome qualificado (`case Limits.MAX:`).

**`match` por valor move o conteúdo.**

```rust
// ❌ NÃO COMPILA
let opt = Some(String::from("oi"));
match opt {
    Some(s) => println!("{s}"),
    None => println!("nada"),
}
println!("{opt:?}");
```

```text
error[E0382]: borrow of partially moved value: `opt`
 --> src/main.rs:7:16
  |
4 |         Some(s) => println!("{s}"),
  |              - value partially moved here
...
7 |     println!("{opt:?}");
  |                ^^^ value borrowed here after partial move
  |
  = note: partial move occurs because value has type `String`, which does not implement the `Copy` trait
help: borrow this binding in the pattern to avoid moving the value
  |
4 |         Some(ref s) => println!("{s}"),
  |              +++
```

O padrão `Some(s)` move a `String` para `s`. A sugestão do compilador (`ref s`) funciona, mas o jeito idiomático é casar com uma referência: `match &opt { Some(s) => ... }`. Como isso funciona, e por que o `ref` virou erro em alguns lugares na edition 2024, é o assunto da [próxima lição](06-padroes-e-referencias.md).

**Ordem dos braços.** Os braços são testados de cima para baixo. Um braço genérico antes de um específico deixa o específico inalcançável, e o compilador avisa com `unreachable pattern`.

## Resumo

- `match` é exaustivo: o E0004 diz exatamente o que falta. Use isso para refatorar.
- Padrões: literais, `_`, faixas (`a..=b`, `a..b`), `|`, desestruturação de structs, enums, tuplas e slices, `..`, guards e `@`.
- Braços com guard não contam para a exaustividade.
- `let` e parâmetros exigem padrões irrefutáveis. Para os refutáveis, `match`, `if let` ou `let...else`.
- Nome solto no padrão cria variável. Para comparar com um valor nomeado, use `const` ou guard.
- `match` por valor move o conteúdo. Case com `&valor` para só emprestar.

## Exercícios

Os exercícios 1 e 3 em [`exercicios.md`](exercicios.md) praticam exaustividade, faixas, `|`, guards e `@`.

## Leitura complementar

- [The Rust Programming Language, 6.2 The `match` Control Flow Construct](https://doc.rust-lang.org/stable/book/ch06-02-match.html)
- [The Rust Programming Language, 19.2 Refutability](https://doc.rust-lang.org/stable/book/ch19-02-refutability.html)
- [The Rust Programming Language, 19.3 Pattern Syntax](https://doc.rust-lang.org/stable/book/ch19-03-pattern-syntax.html)
- [The Rust Reference: Patterns](https://doc.rust-lang.org/reference/patterns.html)
- [PEP 636: Structural Pattern Matching (tutorial)](https://peps.python.org/pep-0636/)

---

[← Lição anterior](04-option.md) · [Índice do módulo](README.md) · [Próxima lição →](06-padroes-e-referencias.md)

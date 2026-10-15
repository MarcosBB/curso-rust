# 03 — Expressões, blocos e funções

**Objetivos.** Ao final, você vai saber:

- a diferença entre instrução e expressão, e por que um bloco tem valor;
- o efeito do `;` no fim de uma linha (o erro número 1 de quem chega ao Rust);
- escrever funções com retorno implícito e com `return`;
- o que são os tipos `()` e `!`.

## Quase tudo é expressão

Uma **expressão** produz um valor: `5`, `x + 1`, `f(x)`. Uma **instrução** (*statement*) executa algo e não produz valor. Em Rust, quase tudo é expressão, inclusive coisas que em C++ e Python são instruções, como blocos, `if` e `match`. As exceções principais são o `let` e as declarações de itens (`fn`, `struct`...).

Um bloco `{ ... }` é uma expressão. O valor dele é o da **última expressão**, escrita sem `;`:

```rust
let x = 3;
let y = {
    let dobro = x * 2;
    dobro + 1          // sem `;`: é o valor do bloco
};
println!("{y}");       // 7
```

Isso é útil para calcular um valor com variáveis auxiliares que somem no `}` em vez de poluírem o resto da função.

O `let`, por outro lado, não tem valor. Não dá para escrever `x = y = 6` como em C:

```rust
// ❌ NÃO COMPILA
fn main() {
    let x = (let y = 6);
}
```

```text
error: expected expression, found `let` statement
 --> src/main.rs:2:14
  |
2 |     let x = (let y = 6);
  |              ^^^
  |
  = note: only supported directly in conditions of `if` and `while` expressions
```

## O `;` transforma a expressão em instrução

Este é o ponto que mais confunde no começo. Colocar `;` depois de uma expressão **descarta o valor dela**. A linha vira uma instrução, e um bloco que termina com uma instrução vale `()`:

```text
{ let a = 3; a + 1 }      →  valor do bloco: 4      (tipo i32)
{ let a = 3; a + 1; }     →  valor do bloco: ()     (tipo ())
```

`()` se chama *unit*: é um tipo com um único valor, também escrito `()`. Ele faz o papel do `void` do C++ e do `None` que uma função Python sem `return` devolve, com uma diferença: em Rust, `()` é um valor de verdade, que pode ser guardado numa variável (`let u: () = ();`) ou num `Vec`.

Por isso o compilador reclama quando o tipo esperado era outro:

```rust
// ❌ NÃO COMPILA
fn main() {
    let x = 3;
    let y: i32 = {
        x + 1;
    };
    println!("{y}");
}
```

```text
error[E0308]: mismatched types
 --> src/main.rs:3:18
  |
3 |       let y: i32 = {
  |  __________________^
4 | |         x + 1;
  | |              - help: remove this semicolon to return this value
5 | |     };
  | |_____^ expected `i32`, found `()`
```

O `help:` aponta exatamente o `;` que sobra.

## Funções

```rust
fn soma(a: i32, b: i32) -> i32 {
    a + b
}
```

- Os **tipos dos parâmetros são obrigatórios**, e o tipo de retorno também, se não for `()`. Rust infere tipos dentro da função, mas não na assinatura. É de propósito: a assinatura é o contrato, e o compilador não deixa que uma mudança no corpo altere esse contrato sem você perceber.
- O corpo é um bloco, então a função devolve o valor da **última expressão**. É o jeito idiomático de retornar.
- `return` existe e serve para sair **antes** do fim.

```rust
fn primeiro_multiplo(de: u32, a_partir_de: u32) -> u32 {
    if a_partir_de % de == 0 {
        return a_partir_de; // saída antecipada
    }
    a_partir_de + (de - a_partir_de % de) // valor da função
}
```

Uma função sem `->` devolve `()`. A ordem das funções no arquivo não importa: você pode chamar uma função definida mais abaixo, sem declaração prévia como em C++.

O mesmo `;` do bloco vale para a função. Este é o erro que você mais vai ver nas primeiras semanas:

```rust
// ❌ NÃO COMPILA
fn mais_um(x: i32) -> i32 {
    x + 1;
}

fn main() {
    println!("{}", mais_um(1));
}
```

```text
error[E0308]: mismatched types
 --> src/main.rs:1:23
  |
1 | fn mais_um(x: i32) -> i32 {
  |    -------            ^^^ expected `i32`, found `()`
  |    |
  |    implicitly returns `()` as its body has no tail or `return` expression
2 |     x + 1;
  |          - help: remove this semicolon to return this value
```

Leia o rótulo secundário: o corpo "não tem expressão final" (*no tail expression*), então devolve `()`. A correção é tirar o `;`.

> [!TIP]
> Se o compilador disser `expected X, found ()`, procure um `;` sobrando no fim de um bloco ou de uma função.

### Sem argumento padrão e sem sobrecarga

Rust não tem argumentos com valor padrão, argumentos nomeados (*kwargs*) nem sobrecarga de funções. Cada função tem um nome e uma lista fixa de parâmetros:

```rust
// ❌ NÃO COMPILA
fn saudar(nome: &str, saudacao: &str) -> String {
    format!("{saudacao}, {nome}!")
}

fn main() {
    println!("{}", saudar("Ana"));
}
```

```text
error[E0061]: this function takes 2 arguments but 1 argument was supplied
 --> src/main.rs:6:20
  |
6 |     println!("{}", saudar("Ana"));
  |                    ^^^^^^------- argument #2 of type `&str` is missing
  |
note: function defined here
 --> src/main.rs:1:4
  |
1 | fn saudar(nome: &str, saudacao: &str) -> String {
  |    ^^^^^^             --------------
help: provide the argument
  |
6 |     println!("{}", saudar("Ana", /* &str */));
  |                                ++++++++++++
```

Os parâmetros de texto aqui são `&str`. Por enquanto, guarde a receita: **um parâmetro que só lê um texto tem tipo `&str`**, e você pode passar um literal (`"Ana"`). O porquê fica para o [Módulo 02](../02-ownership/README.md).

As alternativas idiomáticas (dois nomes de função, um parâmetro `Option`, o padrão *builder*, traits) aparecem ao longo do curso. A vantagem do jeito do Rust é que uma chamada nunca é ambígua: lendo `saudar("Ana", "Oi")`, você sabe exatamente qual função roda.

## O tipo `!`: expressões que nunca terminam

Algumas expressões nunca produzem um valor, porque desviam o fluxo: `panic!(...)`, `return`, `break`, `continue`, um `loop` sem `break` e `std::process::exit`. O tipo delas é `!`, chamado *never type*.

Como `!` nunca acontece, ele se encaixa onde qualquer tipo é esperado. Por isso este `if` compila, mesmo com um braço que não devolve `i32`:

```rust
fn falhar(msg: &str) -> ! {
    eprintln!("erro: {msg}");
    std::process::exit(1)
}

fn dividir(a: i32, b: i32) -> i32 {
    if b == 0 { falhar("divisão por zero") } else { a / b }
}

fn main() {
    println!("{}", dividir(10, 3)); // 3
    println!("{}", dividir(1, 0));  // imprime "erro: divisão por zero" no stderr e sai
}
```

O `-> !` documenta (e o compilador verifica) que `falhar` nunca retorna. É o equivalente do `[[noreturn]]` do C++ e do `NoReturn` do Python, com a diferença de que o Python só usa a anotação para o verificador de tipos.

Você vai usar `!` quase sempre sem escrever o nome dele: um braço de `match` que dá `panic!` ou faz `continue` funciona pelo mesmo motivo.

> [!NOTE]
> Fora do retorno de função, escrever `!` como tipo ainda não é estável. A edition 2024 também mudou um detalhe do never type (o tipo que o compilador escolhe quando não há outra pista passou de `()` para `!`), que só aparece em código avançado.

## Armadilhas comuns

- **`;` na última linha de uma função ou bloco** (E0308 com `found ()`): o `help: remove this semicolon` é a correção certa.
- **Esquecer o `else`** num `if` que deve produzir valor: sem `else`, o `if` vale `()` quando a condição é falsa, e o tipo não fecha.
- **Esperar argumento padrão ou sobrecarga** (E0061): passe todos os argumentos ou crie outra função com outro nome.
- **Usar `return` na última linha.** Funciona, mas não é idiomático: o Clippy avisa com `needless_return`.

## Resumo

- Quase tudo é expressão. `let` e as declarações de itens são instruções.
- Um bloco vale a última expressão. Com `;` no fim, o bloco vale `()`.
- Funções exigem os tipos dos parâmetros e do retorno, e devolvem a última expressão. `return` é para sair antes.
- Não há argumento padrão, argumento nomeado nem sobrecarga.
- `()` é o "nada útil", e é um valor de verdade. `!` é o tipo de quem nunca termina, e se encaixa em qualquer lugar.

## Exercícios

Veja os exercícios 1 a 3 em [`exercicios.md`](exercicios.md).

## Leitura complementar

- [The Rust Programming Language, 3.3 Functions](https://doc.rust-lang.org/stable/book/ch03-03-how-functions-work.html)
- [Comprehensive Rust: Blocks and Scopes](https://google.github.io/comprehensive-rust/control-flow-basics/blocks-and-scopes.html)
- [Rust Reference: Block expressions](https://doc.rust-lang.org/reference/expressions/block-expr.html)
- [Rust Reference: Never type](https://doc.rust-lang.org/reference/types/never.html)

---

[← Lição anterior](02-tipos-primitivos-e-conversoes.md) · [Índice do módulo](README.md) · [Próxima lição →](04-controle-de-fluxo.md)

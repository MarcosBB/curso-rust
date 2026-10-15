# 05 — Lendo erros do compilador

**Objetivos.** Ao final, você vai saber:

- ler cada parte de uma mensagem de erro do Rust;
- usar `rustc --explain` para entender um código de erro;
- a ordem certa de atacar vários erros;
- diferenciar erros, avisos e lints que bloqueiam o build.

## Por que isso importa

Em Rust, você vai ver muitos erros de compilação, principalmente no começo. Isso é esperado: o compilador está fazendo o trabalho que, em outras linguagens, fica para os testes, para o code review ou para a produção.

A boa notícia é que as mensagens de erro do Rust estão entre as melhores que existem. Elas costumam dizer **o que** está errado, **onde**, **por quê** e **como corrigir**. Quem aprende a lê-las com calma avança muito mais rápido do que quem só olha a primeira linha e sai mudando o código.

A partir daqui, todo conceito difícil do curso vem acompanhado de um erro real do compilador.

## Anatomia de uma mensagem

Este código tenta alterar uma variável. Em Rust, variáveis são imutáveis por padrão (veremos o porquê no Módulo 01):

```rust
// ❌ NÃO COMPILA
fn main() {
    let total = 10;
    total = total + 1;
    println!("{total}");
}
```

```text
error[E0384]: cannot assign twice to immutable variable `total`
 --> src/main.rs:3:5
  |
2 |     let total = 10;
  |         ----- first assignment to `total`
3 |     total = total + 1;
  |     ^^^^^^^^^^^^^^^^^ cannot assign twice to immutable variable
  |
help: consider making this binding mutable
  |
2 |     let mut total = 10;
  |         +++

For more information about this error, try `rustc --explain E0384`.
```

Parte por parte:

| Parte | No exemplo | O que significa |
|---|---|---|
| severidade e código | `error[E0384]` | é um erro (não um aviso), e tem um código que você pode pesquisar |
| resumo | `cannot assign twice to immutable variable` | o problema em uma frase |
| local | `--> src/main.rs:3:5` | arquivo, linha e coluna |
| rótulo primário (`^^^`) | linha 3 | **onde** o problema se manifesta |
| rótulo secundário (`---`) | linha 2, `first assignment` | **a causa** ou o contexto. Muitas vezes é a parte mais importante. |
| `help:` | `let mut total` | uma sugestão de correção, com `+++` marcando o que acrescentar |
| `--explain` | `rustc --explain E0384` | onde achar uma explicação longa |

> [!TIP]
> Leia os rótulos secundários. Eles contam a história do erro: "o valor foi criado aqui", "foi movido aqui" e "foi usado aqui". Nos módulos de *ownership* e *borrowing*, a resposta quase sempre está neles.

Também aparecem com frequência as linhas `= note:`, que trazem contexto extra, como o tipo envolvido ou a regra que foi violada.

## `rustc --explain`

Todo código de erro tem uma explicação detalhada, com um exemplo errado e um corrigido. Ela funciona offline:

```bash
rustc --explain E0384
```

```text
An immutable variable was reassigned.

Erroneous code example:

    fn main() {
        let x = 3;
        x = 5; // error, reassignment of immutable variable
    }

By default, variables in Rust are immutable. To fix this error, add the keyword
`mut` after the keyword `let` when declaring the variable. For example:
...
```

O mesmo texto está no [índice de códigos de erro](https://doc.rust-lang.org/error_codes/error-index.html).

## Vários erros de uma vez

Quando há mais de um erro, o compilador lista todos. Veja este código com dois problemas:

```rust
// ❌ NÃO COMPILA
fn dobro(x: i32) -> i32 {
    x * 2;
}

fn main() {
    println!("{}", dobro(4));
    println!("{}", resultado);
}
```

```text
error[E0425]: cannot find value `resultado` in this scope
 --> src/main.rs:7:20
  |
7 |     println!("{}", resultado);
  |                    ^^^^^^^^^ not found in this scope

error[E0308]: mismatched types
 --> src/main.rs:1:21
  |
1 | fn dobro(x: i32) -> i32 {
  |    -----            ^^^ expected `i32`, found `()`
  |    |
  |    implicitly returns `()` as its body has no tail or `return` expression
2 |     x * 2;
  |          - help: remove this semicolon to return this value

Some errors have detailed explanations: E0308, E0425.
For more information about an error, try `rustc --explain E0308`.
```

Duas observações:

1. **A ordem da saída não é a ordem do arquivo.** O compilador trabalha em etapas: primeiro resolve nomes (E0425), depois verifica tipos (E0308). Os erros saem na ordem das etapas.
2. O E0308 é um clássico: o `;` no fim de `x * 2;` transforma a expressão em um comando, e a função passa a não devolver nada (`()`). O `help:` aponta exatamente o ponto-e-vírgula. Veremos o porquê no Módulo 01.

**Estratégia:** corrija um erro de cada vez, começando pelos que parecem mais "básicos" (nomes, sintaxe), e recompile. Um erro costuma causar outros em cascata. Quando o primeiro é corrigido, vários dos seguintes desaparecem.

> [!TIP]
> Use `cargo check` nesse ciclo de corrigir e recompilar. Ele é bem mais rápido que `cargo build`.

## Erros, avisos e lints

| Tipo | Exemplo | Bloqueia o build? |
|---|---|---|
| erro (com código `E`) | `E0384`, `E0308` | sim |
| aviso do compilador | `unused_variables`, `unused_mut`, `dead_code` | não |
| lint *deny-by-default* | `arithmetic_overflow` (compilador), `clippy::approx_constant` (clippy) | **sim**, mesmo sem código `E` |

Não ignore os avisos. Uma `unused_variables`, por exemplo, muitas vezes indica um bug real: um valor calculado que você esqueceu de usar.

## O `help:` nem sempre é a melhor saída

O compilador sugere a correção **mais local** possível, e ela nem sempre é a melhor para o seu programa. O exemplo clássico, que veremos no Módulo 02, é:

```text
help: consider cloning the value if the performance cost is acceptable
```

Copiar o valor (`.clone()`) faz o erro sumir, mas às vezes o certo é mudar o desenho do código. Entenda **por que** o erro aconteceu antes de aplicar a sugestão.

Às vezes as notas citam arquivos da biblioteca padrão (algo como `.../library/core/src/...`). Normalmente dá para ignorar essas linhas e focar no seu código.

## Resumo

- Uma mensagem de erro tem código, resumo, local, rótulo primário (onde), rótulos secundários (por quê), `note` e `help`.
- `rustc --explain EXXXX` explica qualquer código de erro, offline.
- A ordem dos erros segue as etapas do compilador. Corrija um de cada vez e recompile com `cargo check`.
- Alguns lints bloqueiam o build mesmo sem código `E`. Não ignore avisos.
- Entenda o erro antes de aceitar o `help:`.

## Exercícios

Veja os exercícios 5 e 6 em [`exercicios.md`](exercicios.md).

## Leitura complementar

- [Índice de códigos de erro do Rust](https://doc.rust-lang.org/error_codes/error-index.html)
- [The Rust Programming Language, 1.2 Hello, World!](https://doc.rust-lang.org/stable/book/ch01-02-hello-world.html)
- [rustc book: Lints](https://doc.rust-lang.org/rustc/lints/index.html)

---

[← Lição anterior](04-ferramentas-do-dia-a-dia.md) · [Índice do módulo](README.md)

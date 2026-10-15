# Módulo 01 — Tour da sintaxe (para quem já programa)

Um passeio rápido pela sintaxe do Rust, para quem já programa em outra linguagem. O que é igual em todo lugar (laços, funções, operadores) passa depressa. O tempo vai para o que é diferente: quase tudo é expressão, o `;` muda o tipo, variáveis são imutáveis por padrão, não há conversão implícita entre números, overflow tem comportamento definido e o `match` precisa cobrir todos os casos.

O módulo termina com um **kit mínimo** (lições 07 a 10): `Vec`, `HashMap`, closures, módulos e testes, só no nível de receita. É o necessário para fazer os exercícios e o Projeto 1. O aprofundamento vem depois, nos Módulos 07 (coleções, iteradores e closures), 08 (módulos e crates) e 09 (testes).

**Pré-requisitos:** [Módulo 00](../00-introducao/README.md).

**Objetivos.** Ao final do módulo, você vai saber:

- declarar variáveis, entender a imutabilidade por padrão e usar *shadowing*;
- escolher tipos numéricos, converter entre eles de forma explícita e lidar com overflow;
- usar blocos, `if`, `match` e `loop` como expressões, e evitar o `;` que vira `()`;
- trabalhar com tuplas, arrays, slices e as macros de formatação;
- usar `Vec`, `HashMap` e closures no dia a dia;
- dividir um pacote em `lib.rs` + `main.rs` e escrever testes com `cargo test`.

## Lições

| # | Lição | Assunto |
|---|---|---|
| 01 | [Variáveis e mutabilidade](01-variaveis-e-mutabilidade.md) | `let`, `mut`, *shadowing*, escopo, `const` |
| 02 | [Tipos primitivos e conversões](02-tipos-primitivos-e-conversoes.md) | inteiros, floats, `char`, inferência, overflow, `as`/`From`/`TryFrom`, `parse` |
| 03 | [Expressões, blocos e funções](03-expressoes-blocos-e-funcoes.md) | bloco com valor, o `;`, retorno implícito, `()` e `!` |
| 04 | [Controle de fluxo](04-controle-de-fluxo.md) | `if` como expressão, `match` exaustivo, `loop`/`while`/`for`, rótulos |
| 05 | [Tuplas, arrays e slices](05-tuplas-arrays-e-slices.md) | tuplas, `[T; N]`, índice `usize`, verificação de limites, `&[T]` |
| 06 | [Macros e formatação](06-macros-e-formatacao.md) | `println!`, `{}` x `{:?}`, especificadores, `dbg!`, `todo!` |
| 07 | [Kit: `Vec` e `HashMap`](07-kit-vec-e-hashmap.md) | criar, inserir, percorrer, `get` e a prévia de `Option`, contagem com `entry` |
| 08 | [Kit: closures](08-kit-closures.md) | sintaxe, captura, `map`/`filter`/`sort_by_key`/`retain` |
| 09 | [Kit: módulos, `lib.rs` e `main.rs`](09-kit-modulos-lib-e-main.md) | `mod`, `pub`, `use`, duas crates num pacote |
| 10 | [Kit: testes](10-kit-testes.md) | `#[cfg(test)]`, `#[test]`, `assert_eq!`, `cargo test` |

## Prévias

Três assuntos aparecem neste módulo antes da hora, só no nível de "como usar":

| Prévia | Aparece em | Onde vem o detalhe |
|---|---|---|
| `&str` como tipo de um texto e de um parâmetro de texto | lições 01, 03 e 07 | [Módulo 02](../02-ownership/README.md) (`String` x `&str`) |
| `&` e `&mut` (empréstimos), em slices, `for x in &v` e `get` | lições 05, 07 e 08 | [Módulo 03](../03-borrowing-e-lifetimes/README.md) |
| `Option` (`Some`/`None`), em `checked_*` e `get` | lições 02 e 07 | [Módulo 04](../04-structs-enums-e-patterns/README.md) |

## Exercícios

[`exercicios.md`](exercicios.md): 6 exercícios, do "faça compilar" ao contador de palavras que junta o kit inteiro. Soluções em [`solucoes/01-tour-da-sintaxe`](../../solucoes/01-tour-da-sintaxe/).

## Código

| Projeto | O que mostra | Lições |
|---|---|---|
| [`codigo/overflow`](codigo/overflow/) | overflow em debug x release e os métodos `checked_`/`wrapping_`/`saturating_`/`overflowing_` | 02 |
| [`codigo/boletim-escolar`](codigo/boletim-escolar/) | `Vec`, `HashMap`, closures, `lib.rs` + `main.rs` + módulo em arquivo, testes | 07 a 10 |

Para rodar:

```bash
cd codigo/overflow
cargo run
cargo run --release

cd ../boletim-escolar
cargo run
cargo test
```

## Referências principais

- [The Rust Programming Language, cap. 3](https://doc.rust-lang.org/stable/book/ch03-00-common-programming-concepts.html) (conceitos comuns)
- [The Rust Programming Language, caps. 7, 8.1, 8.3, 11.1 e 13.1](https://doc.rust-lang.org/stable/book/) (as bases do kit)
- [Comprehensive Rust: Types and Values](https://google.github.io/comprehensive-rust/types-and-values.html) e [Control Flow Basics](https://google.github.io/comprehensive-rust/control-flow-basics.html)
- [Microsoft RustTraining](https://microsoft.github.io/RustTraining/), trilhas Python e C/C++ (capítulos 3 a 5)

---

[← Módulo anterior](../00-introducao/README.md) · [Índice do curso](../../README.md) · [Próximo módulo →](../02-ownership/README.md)

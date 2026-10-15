# Módulo 04 — Structs, enums e pattern matching

Até aqui você usou os tipos prontos da linguagem. Este módulo mostra como criar os seus: structs para agrupar dados, enums para representar "um entre vários casos, cada um com seus dados", e o *pattern matching* (casamento de padrões) para desmontar tudo isso com segurança. Juntos, eles são a principal ferramenta de modelagem do Rust, e o motivo de muitos bugs comuns em outras linguagens simplesmente não compilarem.

**Pré-requisitos:** Módulos [01](../01-tour-da-sintaxe/README.md), [02](../02-ownership/README.md) e [03](../03-borrowing-e-lifetimes/README.md).

**Objetivos.** Ao final do módulo, você vai saber:

- definir structs e enums com dados, e dar a eles métodos e funções associadas;
- usar `Option<T>` no lugar de `null` e o kit básico de métodos dele;
- usar `match` e toda a sintaxe de padrões, aproveitando a verificação de exaustividade;
- casar padrões com referências no estilo da edition 2024;
- escolher entre `if let`, `let...else`, *let chains*, `if let` guards e `matches!`;
- modelar dados de forma que estados inválidos não possam ser construídos.

## Lições

| # | Lição | Assunto |
|---|---|---|
| 01 | [Structs](01-structs.md) | structs nomeadas, tuple structs, unit structs, sintaxe de atualização, `derive` |
| 02 | [`impl`: métodos e funções associadas](02-impl-metodos-e-funcoes-associadas.md) | `&self`, `&mut self`, `self`, `new`, getters |
| 03 | [Enums algébricos](03-enums-algebricos.md) | variantes com dados, comparação com `std::variant` e Python, layout em memória |
| 04 | [`Option<T>`: adeus, `null`](04-option.md) | `Some`/`None`, métodos essenciais, otimização de nicho |
| 05 | [`match` e padrões](05-match-e-padroes.md) | exaustividade, desestruturação, faixas, slices, guards, `@`, refutabilidade |
| 06 | [Padrões e referências](06-padroes-e-referencias.md) | *match ergonomics*, regras da edition 2024 para `ref`, `mut` e `&` |
| 07 | [`if let`, `let...else` e *let chains*](07-if-let-let-else-e-let-chains.md) | `while let`, let chains (1.88), `if let` guards (1.95), `matches!` |
| 08 | [Estados inválidos irrepresentáveis](08-estados-invalidos-irrepresentaveis.md) | enum de estados, enums no lugar de `bool`, newtype validado |

## Exercícios

[`exercicios.md`](exercicios.md): 6 exercícios. Soluções em [`solucoes/04-structs-enums-e-patterns`](../../solucoes/04-structs-enums-e-patterns/).

## Código

Os exemplos completos estão em [`codigo/`](codigo/):

| Projeto | Lições | Conteúdo |
|---|---|---|
| [`retangulo`](codigo/retangulo/) | 01, 02 | struct `Rect` com métodos e funções associadas |
| [`formas`](codigo/formas/) | 03, 05 | enums `Shape` e `Message` com `match` |
| [`tamanhos`](codigo/tamanhos/) | 03, 04 | `size_of` de enums e de `Option` (otimização de nicho) |
| [`config-porta`](codigo/config-porta/) | 07 | `let...else`, let chains, `if let` guards e `matches!` |
| [`conexao`](codigo/conexao/) | 08 | máquina de estados com enum e newtype `Port` validado |

Para rodar:

```bash
cd codigo/formas
cargo run
cargo test
```

## Referências principais

- [The Rust Programming Language, cap. 5 (structs)](https://doc.rust-lang.org/stable/book/ch05-00-structs.html)
- [The Rust Programming Language, cap. 6 (enums e `match`)](https://doc.rust-lang.org/stable/book/ch06-00-enums.html)
- [The Rust Programming Language, cap. 19 (padrões)](https://doc.rust-lang.org/stable/book/ch19-00-patterns.html)
- [The Rust Reference: Patterns](https://doc.rust-lang.org/reference/patterns.html)
- [Edition Guide: Rust 2024](https://doc.rust-lang.org/edition-guide/rust-2024/index.html) (let chains e regras de padrões)
- [Effective Rust, Item 1](https://effective-rust.com/use-types.html)

---

[← Módulo anterior](../03-borrowing-e-lifetimes/README.md) · [Índice do curso](../../README.md) · [Próximo módulo →](../05-tratamento-de-erros/README.md)

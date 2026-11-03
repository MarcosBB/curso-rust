# Módulo 14 — Macros

> Última revisão: 2026-11-03 (syn 3.0.9, quote 1.0.47, proc-macro2 1.0.107)

Desde o Módulo 01 você usa `println!`, `vec!` e `#[derive(Debug)]`. Este módulo mostra como escrever as suas: macros declarativas com `macro_rules!` (padrões, repetição, recursão e higiene) e macros procedurais com `syn` e `quote` (derive, atributo e *function-like*), com erros que apontam para o lugar certo e testes. No fim, a pergunta mais importante: quando **não** usar uma macro.

**Pré-requisitos:** Módulos 04 (*pattern matching*), 06 (traits e `derive`), 08 (visibilidade, workspaces e dependências) e 09 (testes de integração). A lição 07 cita `#[tokio::main]` e `#[async_trait]`, do Módulo 12.

**Objetivos.** Ao final do módulo, você vai saber:

- explicar o que uma macro faz, quando roda e como ver o código que ela gera (`cargo expand`);
- escrever `macro_rules!` com várias regras, repetições, recursão e mensagens de erro próprias;
- exportar macros com `#[macro_export]` e `$crate`, entendendo a higiene;
- escrever um `#[derive]` com `syn` e `quote`, com atributos auxiliares e genéricos;
- escrever macros de atributo e *function-like*, com erros bons e testes;
- decidir entre macro, função, genérico e `const fn`.

## Lições

| # | Lição | Assunto |
|---|---|---|
| 01 | [O que são macros](01-o-que-sao-macros.md) | tokens, os quatro tipos, `cargo expand`, `#define` do C++ e decorators do Python |
| 02 | [`macro_rules!`: regras e fragmentos](02-macro-rules.md) | regras, fragmentos, `expr` na edition 2024, avaliação dupla, `compile_error!` |
| 03 | [Repetições e recursão](03-repeticoes-e-recursao.md) | `$(...),*`, vírgula final, gerar itens, recursão, *tt muncher* |
| 04 | [Higiene, escopo e `$crate`](04-higiene-e-escopo.md) | higiene *mixed site*, ordem textual, `pub(crate) use`, `#[macro_export]`, `$crate`, `$vis` |
| 05 | [Macros procedurais: um `derive`](05-macros-procedurais-e-derive.md) | crate `proc-macro`, `syn`, `quote`, atributos auxiliares, genéricos, workspace |
| 06 | [Erros e testes de macros procedurais](06-erros-e-testes.md) | `syn::Error`, spans, `quote_spanned!`, testes com `parse_quote!`, `trybuild` |
| 07 | [Macros de atributo e *function-like*](07-atributos-e-function-like.md) | `#[cronometrar]`, `Span::mixed_site()`, `cor!`, macros do ecossistema |
| 08 | [Quando *não* usar macros](08-quando-nao-usar-macros.md) | custos, alternativas (função, genérico, `const fn`, closure), checklist |

## Exercícios

[`exercicios.md`](exercicios.md): 5 exercícios. Soluções em [`solucoes/14-macros`](../../solucoes/14-macros/).

## Código

Os exemplos completos estão em [`codigo/`](codigo/):

- [`macros-declarativas`](codigo/macros-declarativas/): as macros das lições 02 a 04 (`quadrado!`, `cor!`, `hashmap!`, `soma!`, `contar!`, `matriz!`, `comandos!`, `unidades!`, `registrar!`), com testes;
- [`macros-procedurais`](codigo/macros-procedurais/): um mini-workspace com a crate `proc-macro` `minhas-macros` (`#[derive(LinhaCsv)]`, `#[cronometrar]`, `cor!`), a crate `linha-csv` (a trait e o reexport do derive) e o binário `exemplo` (lições 05 a 07).

Para rodar:

```bash
cd codigo/macros-declarativas
cargo run
cargo test

cd ../macros-procedurais
cargo run -p exemplo
cargo test            # testes das três crates
```

Para ver o código gerado, instale o `cargo expand` (`cargo install cargo-expand`) e rode `cargo expand` dentro de `codigo/macros-procedurais/exemplo`.

## Referências principais

- [The Rust Programming Language, cap. 20.5: Macros](https://doc.rust-lang.org/stable/book/ch20-05-macros.html) (usa `syn` 2; o código de derive é o mesmo)
- [The Rust Reference: Macros by example](https://doc.rust-lang.org/reference/macros-by-example.html) e [Procedural macros](https://doc.rust-lang.org/reference/procedural-macros.html)
- [The Little Book of Rust Macros](https://lukaswirth.dev/tlborm/) (a parte de macros declarativas está completa; a de procedurais, incompleta)
- Documentação de [`syn`](https://docs.rs/syn/3/syn/), [`quote`](https://docs.rs/quote) e [`proc_macro`](https://doc.rust-lang.org/proc_macro/index.html)
- [proc-macro-workshop](https://github.com/dtolnay/proc-macro-workshop), de David Tolnay: o próximo passo, com cinco macros para implementar
- [cargo-expand](https://github.com/dtolnay/cargo-expand)

---

[← Módulo anterior](../13-tipos-e-traits-avancados/README.md) · [Índice do curso](../../README.md) · [Próximo módulo →](../15-unsafe-e-ffi/README.md)

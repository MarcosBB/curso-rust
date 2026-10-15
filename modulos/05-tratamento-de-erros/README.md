# Módulo 05 — Tratamento de erros

Rust não tem exceções. Ele separa bugs, que encerram o programa com `panic!`, de falhas esperadas, que são valores comuns do tipo `Result` e aparecem na assinatura de cada função. Este módulo mostra como usar as duas coisas, como propagar erros com `?`, como criar tipos de erro que informam bem, e as duas crates que o ecossistema usa para isso: `thiserror` e `anyhow`. É a base do Projeto 1, que vem logo a seguir.

**Pré-requisitos:** Módulos 01 a 04 (em especial enums, `Option`, `match`, `if let` e `let ... else`, do [Módulo 04](../04-structs-enums-e-patterns/README.md)).

**Objetivos.** Ao final do módulo, você vai saber:

- decidir entre `panic!`, `Option` e `Result` para cada tipo de falha;
- propagar erros com `?` e entender as conversões que ele faz;
- transformar `Option` e `Result` com combinadores, sem `match` aninhado;
- criar um tipo de erro próprio, com mensagem e cadeia de causas;
- usar `thiserror` em bibliotecas e `anyhow` em aplicações;
- escrever um `main` que reporta erros direito e devolve o código de saída certo.

## Lições

| # | Lição | Assunto |
|---|---|---|
| 01 | [`panic!` e erros irrecuperáveis](01-panic-e-erros-irrecuperaveis.md) | bug x falha esperada, unwind e abort, backtrace, `assert!`, quando panicar |
| 02 | [`Result` e o operador `?`](02-result-e-o-operador-interrogacao.md) | `Result`, `#[must_use]`, `?` e `From`, `?` em `Option`, `main` com `Result` e `ExitCode` |
| 03 | [`unwrap`, `expect` e combinadores](03-unwrap-expect-e-combinadores.md) | `map`, `map_err`, `and_then`, `ok_or`, `unwrap_or_else`, `collect` em `Result` |
| 04 | [Tipos de erro próprios](04-tipos-de-erro-proprios.md) | enum de erro, `Display` + `Error` + `From` (receita), `source()`, `Box<dyn Error>` |
| 05 | [`thiserror` e `anyhow`](05-thiserror-e-anyhow.md) | biblioteca x aplicação, `#[error]`/`#[from]`/`#[source]`, `context`, `bail!`, `{:#}` |

## Exercícios

[`exercicios.md`](exercicios.md): 6 exercícios. Soluções em [`solucoes/05-tratamento-de-erros`](../../solucoes/05-tratamento-de-erros/).

## Código

Os exemplos completos estão em [`codigo/`](codigo/):

- [`erro-manual`](codigo/erro-manual/): o tipo de erro da lição 04, escrito à mão, com a cadeia de causas;
- [`config-thiserror-anyhow`](codigo/config-thiserror-anyhow/): biblioteca com `thiserror` e aplicação com `anyhow` (lição 05).

Para rodar:

```bash
cd codigo/erro-manual
cargo run -- exemplos/porta-invalida.txt
cargo test
```

## Referências principais

- [The Rust Programming Language, cap. 9: Error Handling](https://doc.rust-lang.org/stable/book/ch09-00-error-handling.html)
- [Documentação de `std::error`](https://doc.rust-lang.org/std/error/index.html) e [de `std::result`](https://doc.rust-lang.org/std/result/index.html)
- [Rust by Example: Error handling](https://doc.rust-lang.org/rust-by-example/error.html)
- [Documentação do `thiserror`](https://docs.rs/thiserror/latest/thiserror/) e [do `anyhow`](https://docs.rs/anyhow/latest/anyhow/)
- [Effective Rust, Items 3 e 4](https://effective-rust.com/transform.html)

---

[← Módulo anterior](../04-structs-enums-e-patterns/README.md) · [Índice do curso](../../README.md) · [Próximo: Projeto 1 →](../../projetos/01-minigrep/README.md)

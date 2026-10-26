# Módulo 09 — Testes e qualidade

O Módulo 01 deu o kit mínimo de testes. Este módulo mostra o resto: como o `cargo test` funciona por dentro, testes unitários e de integração a fundo, `assert_matches!`, os lints do compilador e do clippy configurados para o projeto inteiro, uma CI no GitHub Actions, *property testing* com `proptest` e benchmarks com `criterion`. É o que transforma "compila e parece funcionar" em "está verificado, e continua verificado a cada commit".

**Pré-requisitos:** Módulos 01 a 08 (em especial a [lição 10 do Módulo 01](../01-tour-da-sintaxe/10-kit-testes.md), o [Módulo 05](../05-tratamento-de-erros/README.md) e o [Projeto 1](../../projetos/01-minigrep/README.md)).

**Objetivos.** Ao final do módulo, você vai saber:

- filtrar, ignorar e organizar testes, e ler a saída do `cargo test`;
- testar *panics*, erros (`Result` e `assert_matches!`) e o executável de verdade (`CARGO_BIN_EXE`);
- configurar lints com `#[expect]`, `[lints]`, `clippy.toml` e `CARGO_BUILD_WARNINGS`;
- montar uma CI com fmt, clippy, testes, documentação e MSRV;
- escrever propriedades com `proptest` e medir com `criterion`.

## Lições

| # | Lição | Assunto |
|---|---|---|
| 01 | [Anatomia do `cargo test`](01-anatomia-do-cargo-test.md) | um executável por crate, ordem, paralelismo, filtros, `#[ignore]`, saída, doctests |
| 02 | [Testes unitários a fundo](02-testes-unitarios-a-fundo.md) | `#[cfg(test)]`, `#[should_panic(expected)]`, testes com `Result`, fixtures e `Drop`, isolamento |
| 03 | [`assert_matches!` e as outras asserções](03-assert-matches-e-asserts.md) | `assert_matches!` (1.96), a importação certa, `debug_assert!`, floats |
| 04 | [Testes de integração](04-testes-de-integracao.md) | `tests/`, `CARGO_BIN_EXE` + `Command`, `tests/common/mod.rs`, `tempfile` |
| 05 | [Lints: `allow`, `expect`, `deny`](05-lints-allow-expect-deny.md) | níveis, `reason`, `#[expect]`, a tabela `[lints]` e `priority` |
| 06 | [Clippy e rustfmt a fundo](06-clippy-e-rustfmt-a-fundo.md) | `pedantic`, `restriction`, `clippy.toml`, `CARGO_BUILD_WARNINGS=deny`, `fmt --check` |
| 07 | [CI com GitHub Actions](07-ci-com-github-actions.md) | workflow com fmt, clippy, test, doc, cache, `--locked` e job de MSRV |
| 08 | [Property testing com `proptest`](08-property-testing-com-proptest.md) | propriedades, estratégias, *shrinking*, `proptest-regressions/` |
| 09 | [Benchmarks com `criterion`](09-benchmarks-com-criterion.md) | `harness = false`, `std::hint::black_box`, intervalo de confiança, ruído |
| 10 | [Outras ferramentas de teste](10-outras-ferramentas-de-teste.md) | `rstest`, `insta`, *mocks* com traits, nextest, cobertura, fuzzing, pirâmide |

## Exercícios

[`exercicios.md`](exercicios.md): 6 exercícios. Soluções em [`solucoes/09-testes-e-qualidade`](../../solucoes/09-testes-e-qualidade/).

## Código

Os exemplos completos estão em [`codigo/`](codigo/):

- [`idade`](codigo/idade/): testes unitários, `should_panic`, testes com `Result`, `assert_matches!` e um teste ignorado (lições 01 a 03);
- [`contador`](codigo/contador/): um `wc` com testes de integração da API e do executável, `tests/common/mod.rs` e `tempfile` (lição 04);
- [`qualidade`](codigo/qualidade/): `[lints]` com `pedantic`, `unwrap_used` e `clippy.toml` (lições 05 e 06);
- [`linha-csv`](codigo/linha-csv/): codificação de uma linha CSV, com propriedades, benchmarks e um [workflow de CI](codigo/linha-csv/.github/workflows/ci.yml) de exemplo (lições 07 a 09);
- [`outras-ferramentas`](codigo/outras-ferramentas/): `rstest`, `insta` e um *mock* feito com uma trait (lição 10).

Para rodar:

```bash
cd codigo/idade
cargo test
cargo test -- --include-ignored

cd ../linha-csv
cargo bench -- --warm-up-time 1 --measurement-time 2
```

## Referências principais

- [The Rust Programming Language, cap. 11: Writing Automated Tests](https://doc.rust-lang.org/stable/book/ch11-00-testing.html)
- [The Cargo Book: `cargo test`](https://doc.rust-lang.org/cargo/commands/cargo-test.html) e [Continuous Integration](https://doc.rust-lang.org/cargo/guide/continuous-integration.html)
- [Clippy](https://doc.rust-lang.org/clippy/)
- [Proptest Book](https://proptest-rs.github.io/proptest/) e [documentação do `criterion`](https://docs.rs/criterion/latest/criterion/)
- [Effective Rust, Items 29 a 32](https://effective-rust.com/clippy.html)

Os próximos projetos põem tudo isso em prática: o Projeto 2 (`csvlite`) tem testes de integração, uma propriedade de ida e volta, um benchmark e CI; o [Projeto 3](../../projetos/03-tarefas/README.md) (`tarefas`) usa `[lints]`, `rstest`, `insta` e uma trait como *mock* do armazenamento.

---

[← Módulo anterior](../08-modulos-e-crates/README.md) · [Índice do curso](../../README.md) · [Próximo: Projeto 2 →](../../projetos/02-csvlite/README.md)

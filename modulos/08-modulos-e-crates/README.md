# Módulo 08 — Módulos, crates e organização de projetos

O Módulo 01 deu o kit mínimo: `mod`, `pub`, `use` e a divisão `lib.rs` + `main.rs`. Este módulo vai além: como o compilador monta a árvore de módulos, as visibilidades finas, *re-exports*, os alvos de um pacote, dependências e versionamento semântico, *features*, *workspaces*, o `rust-version`, documentação com *doctests* e, por fim, publicação e `cargo install`. É o que você precisa para organizar um projeto de verdade, como o Projeto 2.

**Pré-requisitos:** Módulos 01 a 07 (em especial a [lição 09 do Módulo 01](../01-tour-da-sintaxe/09-kit-modulos-lib-e-main.md) e os tipos de erro do [Módulo 05](../05-tratamento-de-erros/README.md)).

**Objetivos.** Ao final do módulo, você vai saber:

- prever onde o compilador procura cada módulo e organizar os arquivos;
- escolher a visibilidade certa (`pub`, `pub(crate)`, `pub(super)`) e montar uma API com `pub use`;
- usar os alvos de um pacote: vários binários, exemplos e testes;
- ler e escrever requisitos de versão, e entender o `Cargo.lock`;
- criar *features* aditivas e *workspaces* com dependências e lints centralizados;
- declarar e verificar o MSRV com `rust-version`;
- documentar uma biblioteca com exemplos que são testados;
- preparar uma crate para publicação e instalar ferramentas com `cargo install`.

## Lições

| # | Lição | Assunto |
|---|---|---|
| 01 | [A árvore de módulos](01-a-arvore-de-modulos.md) | raiz da crate, onde o compilador procura cada `mod`, `rede.rs` x `rede/mod.rs`, `mod` não é `#include` |
| 02 | [Visibilidade](02-visibilidade.md) | privacidade por módulo, `pub(crate)`, `pub(super)`, `pub(in ...)`, campos, `private_interfaces` |
| 03 | [Caminhos, `use` e re-exports](03-caminhos-use-e-reexports.md) | `crate`/`self`/`super`, `use` local ao módulo, `as`, globs, `pub use` como fachada |
| 04 | [Pacotes, crates e alvos](04-pacotes-crates-e-alvos.md) | `src/bin/`, `examples/`, `tests/`, `default-run`, a crate como unidade de compilação |
| 05 | [Dependências e SemVer](05-dependencias-e-semver.md) | requisitos de versão, `path`/`git`, `dev-dependencies`, `Cargo.lock`, `cargo tree`, duas versões da mesma crate |
| 06 | [Features](06-features.md) | `[features]`, dependências opcionais e `dep:`, aditividade, `#[cfg(feature)]` |
| 07 | [Workspaces](07-workspaces.md) | manifesto virtual, `resolver = "3"`, herança de metadados, dependências e lints |
| 08 | [`rust-version` e o resolver 3](08-rust-version-e-resolver-3.md) | MSRV, o piso da edition 2024, resolução ciente de MSRV, mudanças de manifesto da 2024 |
| 09 | [Documentação e doctests](09-documentacao-e-doctests.md) | `///` e `//!`, seções, links, doctests, `?`, `should_panic`, `compile_fail`, doctests combinados |
| 10 | [Publicação e `cargo install`](10-publicacao-e-cargo-install.md) | metadados, `cargo publish --dry-run`, `yank`, `cargo install`, subcomandos `cargo-xyz` |

## Exercícios

[`exercicios.md`](exercicios.md): 6 exercícios. Soluções em [`solucoes/08-modulos-e-crates`](../../solucoes/08-modulos-e-crates/).

## Código

Os exemplos completos estão em [`codigo/`](codigo/):

- [`chat`](codigo/chat/): uma biblioteca com árvore de módulos em vários arquivos e uma fachada (lições 01 a 03);
- [`palavras`](codigo/palavras/): um pacote com biblioteca, dois executáveis, exemplo e teste de integração (lição 04);
- [`duas-versoes`](codigo/duas-versoes/): duas versões da mesma crate no grafo, e a correção com re-export (lição 05);
- [`vendas`](codigo/vendas/): features e dependência opcional (lição 06);
- [`loja`](codigo/loja/): um workspace com uma biblioteca e uma CLI (lição 07);
- [`temperatura`](codigo/temperatura/): documentação e doctests (lição 09);
- [`cargo-ola`](codigo/cargo-ola/): um subcomando do Cargo (lição 10).

Para rodar:

```bash
cd codigo/loja
cargo run -p loja-cli
cargo test
```

## Referências principais

- [The Rust Programming Language, cap. 7: Packages, Crates, and Modules](https://doc.rust-lang.org/stable/book/ch07-00-managing-growing-projects-with-packages-crates-and-modules.html)
- [The Rust Programming Language, cap. 14: More about Cargo and Crates.io](https://doc.rust-lang.org/stable/book/ch14-00-more-about-cargo.html)
- [The Cargo Book](https://doc.rust-lang.org/cargo/)
- [The rustdoc book](https://doc.rust-lang.org/rustdoc/)
- [Effective Rust, cap. 4: Dependencies](https://effective-rust.com/semver.html)

---

[← Módulo anterior](../07-colecoes-iteradores-e-closures/README.md) · [Índice do curso](../../README.md) · [Próximo módulo →](../09-testes-e-qualidade/README.md)

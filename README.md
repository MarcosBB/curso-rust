# Curso de Rust para quem já programa

Um curso de Rust em português, escrito em Markdown, para quem **já sabe programar** e quer aprender Rust sem rever o básico de programação.

O foco é o que Rust tem de diferente: *ownership*, *borrowing* e *lifetimes*, o sistema de tipos e traits, o tratamento de erros sem exceções, a concorrência sem *data races* e o ecossistema (Cargo e crates). Os conceitos são comparados com **C++** e **Python**: C++ mostra o que Rust garante na compilação, e Python mostra o que Rust obriga a deixar explícito.

> [!NOTE]
> Curso em construção. Os módulos entram aos poucos.

## Para quem é

- Você já programa com conforto em pelo menos uma linguagem (C++, Python, Java, JS/TS, Go, C#...).
- Você sabe usar terminal e Git.
- Você **não** precisa saber C++ nem Python, porque as comparações são um apoio, não um pré-requisito.

## Como usar

1. **Instale o Rust** com o [rustup](https://rustup.rs/). O curso usa a **edition 2024** e exige **Rust 1.96 ou mais novo**. Se você já tem o Rust instalado, atualize:
   ```bash
   rustup update
   rustc --version
   ```
2. **Siga os módulos em ordem.** Cada módulo tem um `README.md` com objetivos e a lista de lições.
3. **Rode o código.** Os exemplos de cada módulo estão em `codigo/` como projetos Cargo independentes:
   ```bash
   cd modulos/00-introducao/codigo/ola
   cargo run
   ```
4. **Faça os exercícios** (`exercicios.md` de cada módulo) antes de olhar as soluções em [`solucoes/`](solucoes/).
5. **Faça os projetos** em [`projetos/`](projetos/). Eles juntam o que foi visto em vários módulos.

## Conteúdo

| # | Módulo | Assunto |
|---|---|---|
| 00 | [Introdução e ferramentas](modulos/00-introducao/README.md) | rustup, Cargo, clippy, rustfmt, como ler erros do compilador |
| 01 | [Tour da sintaxe](modulos/01-tour-da-sintaxe/README.md) | expressões, tipos, funções, e o kit mínimo: `Vec`, `HashMap`, closures, módulos, testes |
| 02 | [Ownership](modulos/02-ownership/README.md) | move, `Copy` x `Clone`, RAII, `String` x `&str`, UTF-8 |
| 03 | [Borrowing e lifetimes](modulos/03-borrowing-e-lifetimes/README.md) | `&T` e `&mut T`, borrow checker, slices, anotações de lifetime |
| 04 | [Structs, enums e pattern matching](modulos/04-structs-enums-e-patterns/README.md) | tipos algébricos, `Option`, `match`, let chains |
| 05 | [Tratamento de erros](modulos/05-tratamento-de-erros/README.md) | `Result`, `?`, erros customizados, `thiserror` e `anyhow` |
| 06 | [Traits e generics](modulos/06-traits-e-generics/README.md) | trait bounds, `impl Trait`, `dyn Trait`, traits da std |
| 07 | [Coleções, iteradores e closures](modulos/07-colecoes-iteradores-e-closures/README.md) | coleções da std, `Fn`/`FnMut`/`FnOnce`, iteradores |
| 08 | [Módulos, crates e organização](modulos/08-modulos-e-crates/README.md) | visibilidade, workspaces, features, SemVer, documentação |
| 09 | [Testes e qualidade](modulos/09-testes-e-qualidade/README.md) | testes unitários, de integração e doctests, clippy, CI |
| 10 | Smart pointers | `Box`, `Rc`, `RefCell`, `Weak`, `Cow`, `Deref` e `Drop` |
| 11 | Concorrência | threads, `Send`/`Sync`, `Arc`, `Mutex`, canais, `rayon` |
| 12 | Programação assíncrona | `async`/`await`, tokio, `Pin`, cancelamento |
| 13 | Tipos e traits avançados | variância, HRTB, typestate, `PhantomData`, idiomas |
| 14 | Macros | `macro_rules!` e macros procedurais |
| 15 | Unsafe e FFI | raw pointers, invariantes, FFI com C, Miri |
| 16 | Ecossistema na prática | `clap`, `serde`, `reqwest`, `axum`, `sqlx`, `tracing` |
| 17 | Frameworks e bibliotecas | panorama do ecossistema, área por área |
| 18 | Performance e próximos passos | profiling, perfis do Cargo, wasm, `no_std` |

### Projetos

| # | Projeto | Depois do módulo |
|---|---|---|
| 1 | [`minigrep`](projetos/01-minigrep/README.md): busca em arquivos pela linha de comando | 05 |
| 2 | [`csvlite`](projetos/02-csvlite/README.md): parser de CSV com aspas e a CLI `csvq` | 09 |
| 3 | [`tarefas`](projetos/03-tarefas/README.md): gerenciador de tarefas em CLI com persistência em JSON | 09 |
| 4 | Processador paralelo de arquivos | 11 |
| 5 | Chat TCP assíncrono | 12 |
| 6 | API REST com `axum` e `sqlx` (projeto final) | 16 |

Os links da tabela entram à medida que cada módulo fica pronto.

## Fontes

O curso se baseia principalmente no [Rust Book](https://doc.rust-lang.org/stable/book/) e na documentação oficial, complementados por cursos gratuitos como o [Comprehensive Rust](https://google.github.io/comprehensive-rust/).

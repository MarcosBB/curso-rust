# Módulo 00 — Introdução e ferramentas

Antes de escrever Rust, você precisa do ambiente pronto e de saber conversar com o compilador. Este módulo é curto e prático: por que Rust existe, como instalar, o Cargo, as ferramentas do dia a dia e como ler mensagens de erro, que é a habilidade que mais acelera o aprendizado.

**Pré-requisitos:** saber programar em alguma linguagem e usar o terminal.

**Objetivos.** Ao final do módulo, você vai saber:

- explicar que problema Rust resolve e o que ele cobra em troca;
- instalar e atualizar o Rust, e entender toolchains e editions;
- criar, compilar, rodar e configurar projetos com o Cargo;
- usar `rustfmt`, `clippy` e `rust-analyzer`;
- ler uma mensagem de erro do compilador e agir sobre ela.

## Lições

| # | Lição | Assunto |
|---|---|---|
| 01 | [Por que Rust](01-por-que-rust.md) | controle x segurança, o que Rust garante e o que não garante |
| 02 | [Instalação e toolchains](02-instalacao-e-toolchains.md) | `rustup`, canais, perfis, editions |
| 03 | [Cargo essencial](03-cargo-essencial.md) | `cargo new/check/run/build`, `Cargo.toml`, `Cargo.lock`, debug x release |
| 04 | [Ferramentas do dia a dia](04-ferramentas-do-dia-a-dia.md) | `rustfmt`, `clippy`, `cargo fix`, `rust-analyzer`, documentação |
| 05 | [Lendo erros do compilador](05-lendo-erros-do-compilador.md) | anatomia de um erro, `rustc --explain`, estratégia de correção |

## Exercícios

[`exercicios.md`](exercicios.md): 6 exercícios. Soluções em [`solucoes/00-introducao`](../../solucoes/00-introducao/).

## Código

O projeto de exemplo está em [`codigo/ola`](codigo/ola/). Para rodar:

```bash
cd codigo/ola
cargo run
```

## Referências principais

- [The Rust Programming Language, cap. 1](https://doc.rust-lang.org/stable/book/ch01-00-getting-started.html)
- [The rustup book](https://rust-lang.github.io/rustup/)
- [The Cargo Book](https://doc.rust-lang.org/cargo/)
- [Clippy](https://doc.rust-lang.org/clippy/)

---

[Índice do curso](../../README.md) · [Próximo módulo →](../01-tour-da-sintaxe/README.md)

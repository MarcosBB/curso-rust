# 08 — `rust-version` e o resolver 3

**Objetivos.** Ao final, você vai saber:

- o que o `rust-version` declara e o que acontece com uma toolchain mais velha;
- como o resolver 3 escolhe versões de dependências compatíveis com o seu MSRV;
- verificar o MSRV de verdade;
- as mudanças de manifesto que a edition 2024 trouxe.

## O que é o MSRV

Todo `Cargo.toml` do curso tem `rust-version = "1.96"`. É o *MSRV* (*minimum supported Rust version*): a versão mais antiga do compilador com que o pacote promete compilar. O valor é um número puro, sem operadores (`"1.96"` ou `"1.96.1"`).

Com uma toolchain mais velha que o MSRV, o Cargo para antes de compilar, com uma mensagem clara (aqui, com um MSRV exagerado de propósito):

```toml
rust-version = "1.120"
```

```text
error: rustc 1.99.0 is not supported by the following package:
  msrv@0.1.0 requires rustc 1.120
```

Isso é muito melhor que a alternativa: um erro obscuro de "método não encontrado" no meio da compilação de uma dependência. `--ignore-rust-version` força a tentativa, se você souber o que está fazendo.

A edition também tem um piso. A 2024 exige Rust 1.85, então um MSRV menor é recusado já na leitura do manifesto:

```text
error: failed to parse manifest at `…/Cargo.toml`

Caused by:
  rust-version 1.70 is incompatible with the version (1.85.0) required by the specified edition (2024)
```

O curso usa 1.96 porque usa *let chains* (1.88), `if let` guards (1.95) e `assert_matches!` (1.96).

## O resolver 3: dependências que respeitam o MSRV

Um MSRV não adianta se uma dependência exigir um compilador mais novo. Desde o Rust 1.84, o **resolver 3** (o padrão da edition 2024) leva o `rust-version` em conta: quando há escolha, ele prefere a versão mais nova de cada dependência **compatível com o seu MSRV**. É por isso que toda resolução diz:

```text
     Locking 6 packages to highest Rust 1.96 compatible versions
```

O `cargo add` faz o mesmo. Num pacote com `rust-version = "1.85"`:

```text
$ cargo add --dev proptest
    Updating crates.io index
warning: ignoring proptest@1.12.0 (which requires rustc 1.88) to maintain msrv's rust-version of 1.85
      Adding proptest v1.11.0 to dev-dependencies
```

Ele escolheu a 1.11.0, a mais nova que ainda compila com 1.85, e avisou. Duas observações:

- **É uma preferência, não uma garantia.** Se nenhuma versão compatível satisfizer o requisito, o resolver escolhe uma incompatível mesmo assim, e o erro aparece depois, na compilação.
- **Para testar com o mais novo de tudo** (útil numa CI), desligue a preferência:

```bash
CARGO_RESOLVER_INCOMPATIBLE_RUST_VERSIONS=allow cargo update
```

```text
     Locking 26 packages to highest compatible versions
    Updating bit-set v0.8.0 -> v0.11.1
    Updating bit-vec v0.8.0 -> v0.10.1
...
```

Repare que a linha mudou: "highest compatible versions", sem "Rust 1.85".

## Verificando o MSRV de verdade

Declarar não é verificar. O `rust-version` não impede você de usar, sem querer, uma função da std que só existe numa versão mais nova. Duas defesas:

**O clippy** conhece o MSRV e avisa (`incompatible_msrv`). Com `rust-version = "1.85"`:

```rust
let mut pilha = vec![1, 2, 3, 4];
let par = pilha.pop_if(|x| *x % 2 == 0);
```

```text
warning: current MSRV (Minimum Supported Rust Version) is `1.85.0` but this item is stable since `1.86.0`
 --> src/main.rs:3:21
  |
3 |     let par = pilha.pop_if(|x| *x % 2 == 0);
  |                     ^^^^^^^^^^^^^^^^^^^^^^^
  |
...
  = note: `#[warn(clippy::incompatible_msrv)]` on by default
```

**Compilar com a toolchain mínima**, que é a prova definitiva. Com o `rustup` (Módulo 00):

```bash
rustup toolchain install 1.96
cargo +1.96 check
```

```text
    Checking msrv v0.1.0 (…)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.06s
```

O Módulo 09 coloca isso numa CI.

Subir o MSRV de uma biblioteca publicada obriga quem usa a atualizar o compilador. Muitas crates tratam isso como mudança de `MINOR`, e anunciam. Escolha um MSRV que você consiga manter.

## Mudanças de manifesto da edition 2024

Além do resolver 3 como padrão, a 2024 limpou nomes antigos do `Cargo.toml`. As chaves com `_` que eram aceitas como sinônimo agora são erro:

```text
Caused by:
  `dev_dependencies` is unsupported as of the 2024 edition; instead use `dev-dependencies`
  (in the `msrv` package)
```

```text
Caused by:
  `default_features` is unsupported as of the 2024 edition; instead use `default-features`
  (in the `serde_json` dependency)
```

Se você copiar um `Cargo.toml` de material antigo, troque `_` por `-` nessas chaves.

## Armadilhas comuns

- **`rust-version` abaixo de 1.85 com edition 2024**: recusado no parse.
- **Achar que o `rust-version` verifica o código.** Ele só barra toolchains antigas e orienta o resolver. Verifique com `cargo +1.96 check` e com o clippy.
- **Workspace virtual sem `resolver = "3"`** (lição 07): sem ele, o resolver não respeita o MSRV.
- **Chaves com `_` copiadas de material antigo**: erro na 2024.

## Resumo

- `rust-version` declara o MSRV; toolchains mais velhas param com erro claro.
- A edition 2024 implica MSRV mínimo de 1.85.
- O resolver 3 prefere versões de dependências compatíveis com o MSRV; `cargo add` também.
- Verifique de verdade: clippy (`incompatible_msrv`) e `cargo +1.96 check`.
- Na 2024, `dev_dependencies` e `default_features` são erro: use hífen.

## Exercícios

Os projetos de todos os exercícios em [`exercicios.md`](exercicios.md) declaram `rust-version = "1.96"`. Rode `cargo +1.96 check` neles.

## Leitura complementar

- [The Cargo Book: The `rust-version` field](https://doc.rust-lang.org/cargo/reference/rust-version.html)
- [The Cargo Book: Resolver versions](https://doc.rust-lang.org/cargo/reference/resolver.html#resolver-versions)
- [Edition Guide: Cargo Rust-version aware resolver](https://doc.rust-lang.org/edition-guide/rust-2024/cargo-resolver.html) e [table and key name consistency](https://doc.rust-lang.org/edition-guide/rust-2024/cargo-table-key-names.html)

---

[← Lição anterior](07-workspaces.md) · [Índice do módulo](README.md) · [Próxima lição →](09-documentacao-e-doctests.md)

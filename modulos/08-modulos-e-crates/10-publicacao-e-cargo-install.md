# 10 — Publicação e `cargo install`

**Objetivos.** Ao final, você vai saber:

- os metadados que o crates.io exige e como conferir o pacote com `cargo publish --dry-run`;
- por que uma versão publicada é para sempre, e o que o `yank` faz;
- instalar ferramentas com `cargo install`;
- criar um subcomando do Cargo.

> [!WARNING]
> Esta lição **não** publica nada. Publicar no crates.io é permanente: o nome e a versão ficam reservados para sempre. Tudo aqui usa `--dry-run`.

## Metadados

Para o crates.io, o `[package]` precisa de pelo menos `description` e `license` (um identificador SPDX):

```toml
[package]
name = "temperatura"
version = "0.1.0"
edition = "2024"
rust-version = "1.96"
description = "Conversões de temperatura: exemplo de documentação e doctests do curso de Rust"
license = "MIT OR Apache-2.0"
repository = "https://github.com/exemplo/temperatura"   # recomendado
readme = "README.md"                                     # vai para a página do crates.io
keywords = ["temperatura", "conversao"]
categories = ["science"]
```

`"MIT OR Apache-2.0"` é a combinação mais comum no ecossistema (a do próprio Rust). Sem os metadados, o Cargo avisa e o servidor recusa:

```text
warning: manifest has no description, license, license-file, documentation, homepage or repository
  |
  = note: see https://doc.rust-lang.org/cargo/reference/manifest.html#package-metadata for more info
```

## `cargo publish --dry-run`

O `--dry-run` faz tudo menos o envio: monta o arquivo `.crate`, descompacta numa pasta à parte e **compila a partir dele**. É isso que pega um arquivo esquecido, como um `include_str!` de algo que ficou fora do pacote.

```bash
cargo package --list      # o que vai no pacote
cargo publish --dry-run   # empacota, compila e para antes de enviar
```

```text
$ cargo package --list
Cargo.lock
Cargo.toml
Cargo.toml.orig
src/lib.rs

$ cargo publish --dry-run
    Updating crates.io index
warning: manifest has no documentation, homepage or repository
...
   Packaging temperatura v0.1.0 (…)
    Packaged 4 files, 4.8KiB (2.2KiB compressed)
   Verifying temperatura v0.1.0 (…)
   Compiling temperatura v0.1.0 (…/target/package/temperatura-0.1.0)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.76s
   Uploading temperatura v0.1.0 (…)
warning: aborting upload due to dry run
```

O `Cargo.toml.orig` é o seu manifesto original. O `Cargo.toml` empacotado é uma versão normalizada (por exemplo, com as heranças de workspace já resolvidas). As chaves `include` e `exclude` controlam o que entra.

Uma crate interna, que nunca deve ir para o crates.io, leva `publish = false`. É o caso dos projetos do curso:

```text
error: `temperatura` cannot be published.
`package.publish` must be set to `true` or a non-empty list in Cargo.toml to publish.
```

Para publicar de verdade, você criaria um token no crates.io, faria `cargo login` e rodaria `cargo publish`. Num workspace, `cargo publish --workspace` (Rust 1.90) publica todos os membros na ordem das dependências.

## Uma versão é para sempre

O crates.io não deixa apagar nem substituir uma versão publicada. É isso que torna o `Cargo.lock` confiável: `1.0.1` é sempre o mesmo código.

Se uma versão tiver um bug grave, existe o `yank`:

```bash
cargo yank --vers 1.0.1          # novos lockfiles não escolhem mais a 1.0.1
cargo yank --vers 1.0.1 --undo   # desfaz
```

O `yank` não apaga nada: quem já tem a `1.0.1` no `Cargo.lock` continua compilando. E se você publicou uma senha ou um token por engano, `yank` não resolve: **revogue o segredo**, porque o código continua baixável.

Toda crate publicada ganha uma página no [docs.rs](https://docs.rs/), gerada a partir dos `///` e `//!` (lição 09), e o README vira a página no crates.io.

| | Python | C++ | Rust |
|---|---|---|---|
| repositório | PyPI | Conan Center, vcpkg | crates.io |
| documentação | Read the Docs (opcional) | — | docs.rs, para toda crate |
| teste antes de enviar | `twine check` | — | `cargo publish --dry-run` |
| remover versão | *yank* | depende | `cargo yank` (não apaga) |

## `cargo install`

`cargo install` baixa uma crate com binário, compila em **release** e põe o executável em `~/.cargo/bin` (que o `rustup` já colocou no `PATH`). É o `pipx install` do Rust:

```bash
cargo install ripgrep            # do crates.io
cargo install --locked ripgrep   # usando o Cargo.lock publicado
cargo install --path .           # o projeto atual
cargo install --list             # o que está instalado
cargo uninstall ripgrep
```

Dois detalhes:

- **Sem `--locked`, o Cargo resolve as dependências de novo**, com as versões mais novas compatíveis, e o resultado pode ser diferente do que o autor testou. Para ferramentas, prefira `--locked`.
- **Só serve para binários.** Bibliotecas não se "instalam": você as declara no `Cargo.toml`. Também não é a forma de distribuir para quem não tem Rust; para isso existem binários pré-compilados.

## Subcomandos do Cargo

Qualquer executável chamado `cargo-xyz` no `PATH` vira o comando `cargo xyz`. É assim que funcionam ferramentas como `cargo-nextest`, `cargo-llvm-cov`, `cargo-hack` e `cargo-semver-checks` (Módulos 09 e 17).

O projeto [`codigo/cargo-ola`](codigo/cargo-ola/src/main.rs) é um subcomando mínimo:

```rust
fn main() {
    // Chamado como `cargo ola x y`, o Cargo executa `cargo-ola ola x y`:
    // o primeiro argumento depois do programa é o nome do subcomando.
    let args: Vec<String> = std::env::args().skip(1).collect();
    let resto = match args.first().map(String::as_str) {
        Some("ola") => &args[1..],
        _ => &args[..],
    };
    println!("olá do subcomando!");
    println!("argumentos: {args:?}");
    println!("seus argumentos: {resto:?}");
}
```

Instalando numa pasta temporária, para não mexer no seu `~/.cargo/bin`:

```bash
cargo install --path . --root /tmp/instalado
PATH="/tmp/instalado/bin:$PATH" cargo ola --verbose x
```

```text
  Installing /tmp/instalado/bin/cargo-ola
   Installed package `cargo-ola v0.1.0 (…/cargo-ola)` (executable `cargo-ola`)
warning: be sure to add `/tmp/instalado/bin` to your PATH to be able to run the installed binaries
olá do subcomando!
argumentos: ["ola", "--verbose", "x"]
seus argumentos: ["--verbose", "x"]
```

Repare no primeiro argumento: o Cargo repassa o nome do subcomando (`ola`). Um subcomando real descarta esse argumento antes de interpretar o resto. Para limpar: `cargo uninstall cargo-ola --root /tmp/instalado`.

## Armadilhas comuns

- **Publicar para testar.** Não existe desfazer. Use `--dry-run`.
- **`yank` para esconder um segredo vazado.** Revogue o segredo.
- **`cargo install` sem `--locked`** e estranhar um erro que o autor não tem.
- **Esperar que `cargo install` instale bibliotecas.** Só binários.
- **Esquecer o `publish = false`** numa crate interna de um workspace que tem outras crates publicadas.

## Resumo

- crates.io exige `description` e `license`; `repository`, `readme`, `keywords` e `categories` são recomendados.
- `cargo publish --dry-run` empacota e compila a partir do pacote; `cargo package --list` mostra o conteúdo; `publish = false` protege crates internas.
- Versões publicadas são imutáveis; `cargo yank` só impede novos lockfiles de escolhê-las.
- `cargo install` compila em release e instala binários em `~/.cargo/bin`; use `--locked`.
- Um executável `cargo-xyz` no `PATH` vira `cargo xyz`.

## Exercícios

Num dos projetos dos exercícios em [`exercicios.md`](exercicios.md), acrescente os metadados, tire o `publish = false` (se houver) e rode `cargo publish --dry-run`. Não rode sem o `--dry-run`.

## Leitura complementar

- [The Rust Programming Language, 14.2 Publishing a Crate to Crates.io](https://doc.rust-lang.org/stable/book/ch14-02-publishing-to-crates-io.html)
- [The Rust Programming Language, 14.4 Installing Binaries](https://doc.rust-lang.org/stable/book/ch14-04-installing-binaries.html) e [14.5 Extending Cargo](https://doc.rust-lang.org/stable/book/ch14-05-extending-cargo.html)
- [The Cargo Book: Publishing on crates.io](https://doc.rust-lang.org/cargo/reference/publishing.html)
- [The Cargo Book: `cargo install`](https://doc.rust-lang.org/cargo/commands/cargo-install.html)

---

[← Lição anterior](09-documentacao-e-doctests.md) · [Índice do módulo](README.md)

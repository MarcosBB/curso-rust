# 03 — Cargo essencial

**Objetivos.** Ao final, você vai saber:

- criar, compilar e rodar um projeto com o Cargo;
- o que são `Cargo.toml`, `Cargo.lock` e a pasta `target/`;
- a diferença entre os perfis debug e release;
- adicionar uma dependência.

## O que é o Cargo

O Cargo é o gerenciador de projetos do Rust. Ele faz sozinho o que em outras linguagens exige várias ferramentas:

| Tarefa | Python | C++ | Rust |
|---|---|---|---|
| criar projeto | `uv init` | (à mão) | `cargo new` |
| dependências | `pip`/`uv add` | vcpkg, Conan | `cargo add` |
| manifesto | `pyproject.toml` | `CMakeLists.txt` | `Cargo.toml` |
| versões travadas | `uv.lock` | (depende) | `Cargo.lock` |
| ambiente isolado | `venv` | — | não precisa |
| build | — | CMake + make/ninja | `cargo build` |
| testes | `pytest` | GoogleTest, Catch2 | `cargo test` |
| documentação | Sphinx | Doxygen | `cargo doc` |

Na prática, você quase nunca chama o compilador (`rustc`) diretamente. O Cargo chama por você.

## Criando um projeto

```bash
cargo new ola
```

```text
    Creating binary (application) `ola` package
note: see more `Cargo.toml` keys and their definitions at https://doc.rust-lang.org/cargo/reference/manifest.html
```

O que foi criado:

```text
ola/
├── .git/          # repositório git novo (exceto se você já estiver dentro de um)
├── .gitignore     # já ignora target/
├── Cargo.toml     # o manifesto
└── src/
    └── main.rs    # ponto de entrada
```

O `src/main.rs` já vem com o clássico:

```rust
fn main() {
    println!("Hello, world!");
}
```

Variações úteis:

- `cargo new --lib nome` cria uma **biblioteca** (`src/lib.rs` em vez de `main.rs`). Veremos no Módulo 01.
- `cargo init` faz o mesmo numa pasta que já existe.

## O `Cargo.toml`

```toml
[package]
name = "ola"
version = "0.1.0"
edition = "2024"

[dependencies]
```

- `[package]`: os metadados do projeto. A `edition` você já conhece da lição anterior.
- `[dependencies]`: as bibliotecas (*crates*) de que o projeto depende.

Nos projetos deste curso, acrescente também a versão mínima do Rust:

```toml
[package]
name = "ola"
version = "0.1.0"
edition = "2024"
rust-version = "1.96"
```

## Compilar, checar e rodar

```bash
cargo run
```

```text
   Compiling ola v0.1.0 (/caminho/para/ola)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.14s
     Running `target/debug/ola`
Hello, world!
```

Os comandos do dia a dia:

| Comando | O que faz |
|---|---|
| `cargo check` | só verifica se compila, sem gerar executável. **É o mais rápido**: use enquanto escreve. |
| `cargo build` | compila e gera o executável em `target/debug/` |
| `cargo run` | compila (se precisar) e executa |
| `cargo test` | compila e roda os testes (Módulo 01) |
| `cargo build --release` | compila com otimizações, em `target/release/` |

> [!TIP]
> Para passar argumentos ao **seu programa** (e não ao Cargo), use `--`: `cargo run -- arquivo.txt -v`.

## Debug x release

O Cargo tem dois perfis de compilação principais:

- **`dev`** (padrão de `build`, `run` e `test`): compila rápido, sem otimizações, com informações de debug e **verificações extras**.
- **`release`** (`--release`): compila mais devagar, mas o executável é muito mais rápido.

Depois de um `cargo build` e de um `cargo build --release`:

```text
target/
├── debug/
│   └── ola        # executável de desenvolvimento
└── release/
    └── ola        # executável otimizado
```

A diferença não é só de velocidade. No perfil `dev`, um **overflow de inteiro** encerra o programa com erro (*panic*). No `release`, essa verificação é desligada por desempenho, e o valor "dá a volta". Veremos isso no Módulo 01.

> [!WARNING]
> Nunca meça desempenho com `cargo run` sem `--release`. O código de debug pode ser **muitas vezes** mais lento que o de release, e a comparação não vale nada.

## Dependências

Para adicionar uma crate do [crates.io](https://crates.io/) (o repositório oficial, como o PyPI):

```bash
cargo add rand
```

Isso acrescenta uma linha no `Cargo.toml`:

```toml
[dependencies]
rand = "0.10.3"
```

Apesar de parecer uma versão exata, `"0.10.3"` quer dizer "0.10.3 ou qualquer versão compatível mais nova" (versionamento semântico, assunto do Módulo 08).

No próximo build, o Cargo baixa e compila a crate e as dependências dela. Não existe `venv`: cada projeto tem as próprias dependências, e não há instalação global para atrapalhar.

### O `Cargo.lock`

No primeiro build, o Cargo cria o `Cargo.lock` com as **versões exatas** de todas as dependências, inclusive as indiretas. Ele garante que todo mundo compila com as mesmas versões.

- Não edite à mão. Para atualizar as versões, use `cargo update`.
- Em **aplicações** (executáveis), coloque o `Cargo.lock` no git.

## Documentação local

```bash
cargo doc --open
```

Isso gera e abre no navegador a documentação do seu projeto **e de todas as dependências**, no mesmo formato do [docs.rs](https://docs.rs/). Funciona offline e mostra exatamente as versões que você está usando.

## Resumo

- O Cargo cria projetos, gerencia dependências, compila, testa e gera documentação.
- `Cargo.toml` é o manifesto. `Cargo.lock` trava as versões exatas.
- Use `cargo check` enquanto escreve e `cargo run` para executar.
- Perfil `dev` para desenvolver, `--release` para medir e distribuir. Eles se comportam diferente em overflow.
- `cargo add` adiciona dependências. Não existe `venv`.

## Exercícios

Veja o exercício 2 em [`exercicios.md`](exercicios.md).

## Leitura complementar

- [The Rust Programming Language, 1.3 Hello, Cargo!](https://doc.rust-lang.org/stable/book/ch01-03-hello-cargo.html)
- [The Cargo Book](https://doc.rust-lang.org/cargo/)
- [The Cargo Book: Profiles](https://doc.rust-lang.org/cargo/reference/profiles.html)

---

[← Lição anterior](02-instalacao-e-toolchains.md) · [Índice do módulo](README.md) · [Próxima lição →](04-ferramentas-do-dia-a-dia.md)

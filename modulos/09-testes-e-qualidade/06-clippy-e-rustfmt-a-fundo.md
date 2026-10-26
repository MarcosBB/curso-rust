# 06 — Clippy e rustfmt a fundo

**Objetivos.** Ao final, você vai saber:

- escolher os grupos e lints do clippy para o seu projeto (`pedantic`, `restriction`);
- configurar o clippy com `clippy.toml` e com o `rust-version`;
- transformar avisos em erro com `CARGO_BUILD_WARNINGS=deny`, sem invalidar o cache;
- usar o `rustfmt` como verificação.

A [lição 04 do Módulo 00](../00-introducao/04-ferramentas-do-dia-a-dia.md) apresentou o `cargo clippy`, as categorias e o `cargo fmt`. Aqui é a configuração para um projeto de verdade, usando o [`codigo/qualidade`](codigo/qualidade/).

## `--all-targets`: o clippy também nos testes

`cargo clippy` sozinho verifica a biblioteca e os binários. Testes, exemplos e benchmarks ficam de fora. Para incluir tudo:

```bash
cargo clippy --all-targets
```

Sem isso, um `mod tests` cheio de problemas passa limpo. Daqui em diante, todo comando de clippy do curso usa `--all-targets`. (O `cargo clippy --fix` já implica `--all-targets`.)

## `pedantic`: ligue o grupo e desligue o que incomoda

O grupo `pedantic` é opinativo de propósito, e tem falsos positivos. O jeito usual é ligar o grupo inteiro e desligar, um a um, os lints que não fazem sentido para você (com a `priority` da lição 05):

```toml
[lints.clippy]
pedantic = { level = "warn", priority = -1 }
must_use_candidate = "allow"
```

Alguns lints pedânticos ensinam bons hábitos. Um exemplo: função pública que devolve `Result` sem uma seção `# Errors` na documentação (o formato do [Módulo 08](../08-modulos-e-crates/README.md)):

```text
warning: docs for function returning `Result` missing `# Errors` section
 --> src/lib.rs:7:1
  |
7 | pub fn ler_notas(texto: &str) -> Result<Vec<u32>, ParseIntError> {
  | ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
  |
  = help: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.99.0/index.html#missing_errors_doc
  = note: `-W clippy::missing-errors-doc` implied by `-W clippy::pedantic`
  = help: to override `-W clippy::pedantic` add `#[allow(clippy::missing_errors_doc)]`
```

Outro, o `cast_possible_truncation`, avisa de todo `as` que pode cortar bits. Quando a conversão é segura por um motivo que o clippy não enxerga, silencie no lugar, com o motivo:

```rust
#[expect(
    clippy::cast_possible_truncation,
    reason = "a média de valores u32 sempre cabe em u32"
)]
Some((soma / notas.len() as u64) as u32)
```

Se um dia o código mudar para `u32::try_from(...)`, o `expect` deixa de se cumprir e avisa (lição 05).

## `restriction`: escolha a dedo

O grupo `restriction` é uma coleção de proibições, várias contraditórias entre si (há lints que exigem e que proíbem a mesma coisa). **Nunca** ligue o grupo inteiro. Escolha os que fazem sentido. O mais comum é `unwrap_used`:

```toml
[lints.clippy]
unwrap_used = "warn"
```

```text
warning: used `unwrap()` on a `Result` value
 --> src/main.rs:3:17
  |
3 |     let notas = qualidade::ler_notas(&texto).unwrap();
  |                 ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
  |
  = note: if this value is an `Err`, it will panic
  = help: consider using `expect()` to provide a better panic message
  = help: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.99.0/index.html#unwrap_used
  = note: requested on the command line with `-W clippy::unwrap-used`
```

Só que, nos testes, `unwrap()` é o idioma: se falhar, o teste falha, que é o que você quer. Para liberar só lá, existe o `clippy.toml`.

## `clippy.toml`

Um arquivo `clippy.toml` (ou `.clippy.toml`) na raiz do pacote configura **o comportamento** de alguns lints (quais lints ligar continua no `Cargo.toml`):

```toml
# clippy.toml
allow-unwrap-in-tests = true
```

Sem ele, o `cargo clippy --all-targets` aponta o `unwrap` dentro do `#[test]`:

```text
warning: used `unwrap()` on a `Result` value
  --> src/lib.rs:35:21
warning: `qualidade` (lib test) generated 1 warning
```

Com ele, só o `unwrap` do código de produção é apontado. Outras opções úteis: `disallowed-methods` (proíbe métodos específicos, como `std::env::set_var`) e `too-many-arguments-threshold`. O próprio livro do clippy avisa que as opções do arquivo não são estáveis: confira a [lista](https://doc.rust-lang.org/clippy/lint_configuration.html) quando atualizar.

### O clippy lê o `rust-version`

O `clippy.toml` aceita `msrv`, mas você não precisa dele: o clippy usa o `rust-version` do `Cargo.toml`. Com `rust-version = "1.85"` e um método estabilizado na 1.94:

```text
warning: current MSRV (Minimum Supported Rust Version) is `1.85.0` but this item is stable since `1.94.0`
 --> src/main.rs:7:18
  |
7 |     for par in v.array_windows::<2>() {
  |                  ^^^^^^^^^^^^^^^^^^^^
  |
  = help: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.99.0/index.html#incompatible_msrv
  = note: `#[warn(clippy::incompatible_msrv)]` on by default
```

É uma rede de segurança, não uma garantia (ele não conhece todo item). A garantia é compilar na versão mínima, como faz o job de MSRV da [lição 07](07-ci-com-github-actions.md).

## Avisos que quebram o build: `CARGO_BUILD_WARNINGS`

Na sua máquina, um aviso é um lembrete. Na CI, um aviso que ninguém lê vira cem avisos em um mês. O que se quer na CI é: **qualquer aviso falha**. Desde o Cargo 1.97, existe uma configuração para isso:

```bash
CARGO_BUILD_WARNINGS=deny cargo clippy --all-targets
```

```text
warning: used `unwrap()` on a `Result` value
 --> src/main.rs:3:17
...
error: `qualidade` (bin "qualidade") generated 1 warning (1 duplicate)
error: warnings are denied by `build.warnings` configuration
```

O comando sai com o código 101. A mesma configuração pode ficar num arquivo `.cargo/config.toml` (`[build]` com `warnings = "deny"`) e vale para `cargo build`, `test`, `doc` (um link quebrado na documentação vira erro) e `clippy`.

O jeito antigo, que você vai encontrar em muitos projetos e tutoriais, é `cargo clippy -- -D warnings` (ou `RUSTFLAGS="-D warnings"`):

```text
error: used `unwrap()` on a `Result` value
...
  = note: `-D clippy::unwrap-used` implied by `-D warnings`
```

O resultado parece o mesmo, mas há uma diferença: `-D warnings` muda as flags do compilador, e mudar as flags **invalida o cache de build**. Alternar entre `cargo build` e `cargo clippy -- -D warnings` recompila tudo. O `CARGO_BUILD_WARNINGS` deixa a compilação igual e só decide, no fim, se os avisos viram erro.

> [!WARNING]
> No Cargo 1.96 (o mínimo do curso), a variável é **ignorada em silêncio**: o mesmo comando mostra o aviso e sai com código 0. Na CI, use a toolchain stable para o clippy, e não conte com a variável no job da versão mínima.

```text
$ CARGO_BUILD_WARNINGS=deny cargo +1.96 clippy --all-targets
warning: used `unwrap()` on a `Result` value
...
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.86s
```

## `rustfmt` como verificação

`cargo fmt --check` não muda nada: mostra o diff e sai com código 1 se algum arquivo estiver fora do padrão.

```text
$ cargo fmt --check
Diff in src/main.rs:1:
-fn main(){let x=1;
-println!("{x}");}
+fn main() {
+    let x = 1;
+    println!("{x}");
+}
```

O `rustfmt` aceita um `rustfmt.toml` (`max_width`, `style_edition` e algumas outras opções estáveis). A recomendação é **não configurar nada**: o valor do estilo único está em todo projeto Rust ter a mesma cara. Na edition 2024 o estilo 2024 já é o padrão (ele muda detalhes como a ordenação dos `use`, que passa a comparar números como números: `u8` antes de `u16`).

## Comparação

| | C++ | Python | Rust |
|---|---|---|---|
| linter | clang-tidy | ruff, pylint, mypy | clippy (mais os lints do `rustc`) |
| configuração | `.clang-tidy` | `pyproject.toml` | `[lints]` no `Cargo.toml` + `clippy.toml` |
| aviso vira erro | `-Werror` | padrão (o linter sai com erro) | `CARGO_BUILD_WARNINGS=deny` |
| formatador | clang-format (com estilos) | black, `ruff format` | rustfmt (um estilo para todos) |

## Armadilhas comuns

- **Esquecer `--all-targets`.** O código de teste fica sem verificação.
- **Ligar `clippy::restriction` inteiro.** Lints contraditórios; escolha a dedo.
- **`-D warnings` no dia a dia.** Invalida o cache. Prefira `CARGO_BUILD_WARNINGS=deny`.
- **Contar com `CARGO_BUILD_WARNINGS` no Cargo 1.96.** É ignorada.

## Resumo

- `cargo clippy --all-targets` inclui testes, exemplos e benchmarks.
- `pedantic` com `priority = -1` e exceções; de `restriction`, só lints escolhidos (como `unwrap_used`).
- `clippy.toml` ajusta lints (`allow-unwrap-in-tests = true`); o clippy usa o `rust-version` como MSRV.
- `CARGO_BUILD_WARNINGS=deny` (Cargo 1.97+) faz qualquer aviso falhar sem invalidar o cache; `-D warnings` é o jeito antigo.
- `cargo fmt --check` verifica o estilo; não configure o `rustfmt` sem motivo.

## Exercícios

Veja o exercício 3 em [`exercicios.md`](exercicios.md).

## Leitura complementar

- [Clippy: Usage](https://doc.rust-lang.org/clippy/usage.html) e [Configuration](https://doc.rust-lang.org/clippy/configuration.html)
- [Lista de lints do clippy](https://rust-lang.github.io/rust-clippy/)
- [The Cargo Book: `build.warnings`](https://doc.rust-lang.org/cargo/reference/config.html#buildwarnings)
- [Effective Rust, Item 29: Listen to Clippy](https://effective-rust.com/clippy.html)
- [rustfmt: configuração](https://rust-lang.github.io/rustfmt/)

---

[← Lição anterior](05-lints-allow-expect-deny.md) · [Índice do módulo](README.md) · [Próxima lição →](07-ci-com-github-actions.md)

# 04 — Ferramentas do dia a dia

**Objetivos.** Ao final, você vai saber:

- formatar código com `rustfmt`;
- usar o `clippy` para achar problemas e código pouco idiomático;
- aplicar correções automáticas com `cargo fix`;
- aproveitar o `rust-analyzer`, a documentação offline e o Playground.

## `cargo fmt`: formatação

O Rust tem um estilo oficial, e o `rustfmt` o aplica. Não há discussão sobre onde vai a chave ou quantos espaços usar: rode o formatador e pronto.

```bash
cargo fmt            # formata todo o projeto
cargo fmt --check    # só verifica (útil em CI), mostra o diff e falha se algo estiver fora do padrão
```

É o equivalente ao `black`/`ruff format` do Python e ao `clang-format` do C++, com a diferença de que **todo mundo usa o mesmo estilo**. Código Rust de projetos diferentes tem a mesma cara.

> [!TIP]
> Configure o editor para formatar ao salvar. Você nunca mais vai pensar nisso.

## `cargo clippy`: o linter

O `clippy` conhece centenas de padrões de código problemático ou pouco idiomático. Para quem está aprendendo, ele funciona como um revisor que ensina o "jeito Rust" de escrever.

Veja este programa, que compila normalmente com `cargo build`:

```rust
fn main() {
    let mut x = 3.1415;
    let v = vec![1, 2, 3];
    for i in 0..v.len() {
        println!("{}", v[i]);
    }
    if v.len() == 0 {
        println!("vazio");
    }
    println!("{}", x * 2.0);
}
```

O `cargo clippy` aponta cinco problemas (saída encurtada):

```text
warning: variable does not need to be mutable
 --> src/main.rs:2:9
  |
2 |     let mut x = 3.1415;
  |         ----^
  |         |
  |         help: remove this `mut`

error: approximate value of `f{32, 64}::consts::PI` found
 --> src/main.rs:2:17
  |
2 |     let mut x = 3.1415;
  |                 ^^^^^^
  |
  = help: consider using the constant directly
  = note: `#[deny(clippy::approx_constant)]` on by default

warning: the loop variable `i` is only used to index `v`
 --> src/main.rs:4:14
  |
4 |     for i in 0..v.len() {
  |              ^^^^^^^^^^
...
help: consider using an iterator
  |
4 -     for i in 0..v.len() {
4 +     for <item> in &v {

warning: length comparison to zero
 --> src/main.rs:7:8
  |
7 |     if v.len() == 0 {
  |        ^^^^^^^^^^^^ help: using `is_empty` is clearer and more explicit: `v.is_empty()`

warning: useless use of `vec!`
 --> src/main.rs:3:13
  |
3 |     let v = vec![1, 2, 3];
  |             ^^^^^^^^^^^^^ help: you can use an array directly: `[1, 2, 3]`

error: could not compile `demo` (bin "demo") due to 1 previous error; 4 warnings emitted
```

Repare em três coisas:

1. O primeiro aviso vem do **próprio compilador** (`unused_mut`). O `clippy` mostra os avisos do compilador junto com os dele.
2. Um dos problemas é **erro**, não aviso: escrever `3.1415` em vez de usar a constante `PI` é considerado bug. As categorias do clippy têm severidades diferentes:

   | Categoria | O que pega | Padrão |
   |---|---|---|
   | `correctness` | código quase certamente errado | **erro** |
   | `suspicious`, `style`, `complexity`, `perf` | código suspeito, não idiomático, complicado ou lento | aviso |
   | `pedantic`, `restriction`, `nursery` | regras opcionais e mais rígidas | desligado |

3. Quase todo aviso vem com `help:`, que diz **como** corrigir. O `for i in 0..v.len()` com `v[i]` é o laço típico de C/C++/Python antigo, e o Rust prefere iterar direto sobre a coleção.

> [!NOTE]
> Rode o `cargo clippy` com frequência durante o curso. Os avisos dele costumam antecipar assuntos dos próximos módulos.

## `cargo fix` e `cargo clippy --fix`

Muitas sugestões podem ser aplicadas automaticamente:

```bash
cargo fix             # aplica as sugestões do compilador
cargo clippy --fix    # aplica as sugestões do clippy
```

Por segurança, os dois se recusam a rodar se houver mudanças não commitadas no git:

```text
error: the working directory of this package has uncommitted changes, and `cargo fix` can potentially perform destructive changes; if you'd like to suppress this error pass `--allow-dirty`, or commit the changes to these files:
...
```

Faça commit antes, ou passe `--allow-dirty` se souber o que está fazendo.

## `rust-analyzer`: o editor que entende Rust

O `rust-analyzer` dá ao editor:

- **erros enquanto você digita**, sem precisar compilar;
- **tipos inferidos** mostrados no código (*inlay hints*). Isso ajuda muito a aprender, porque mostra o que o compilador deduziu;
- autocompletar, ir para a definição, renomear com segurança;
- botões "Run" e "Debug" acima de `main` e dos testes.

Na lição anterior vimos como instalar. Se os tipos não aparecerem ao lado das variáveis, o `rust-analyzer` não está ativo.

## Documentação offline

```bash
rustup doc --std       # biblioteca padrão
rustup doc --book      # o Rust Book
cargo doc --open       # seu projeto + dependências
```

A documentação da biblioteca padrão é excelente e cheia de exemplos. Vale criar o hábito de consultar. Cada página tem um link `Source` que leva ao código-fonte.

## Rust Playground

O [Rust Playground](https://play.rust-lang.org/) roda Rust no navegador. Ele serve para testar algo rápido, compartilhar um trecho com um link e alternar entre stable, beta e nightly. Também mostra o assembly gerado, e pode rodar o clippy e o rustfmt.

Rust não tem um REPL oficial como o do Python. Na prática, o Playground e testes pequenos cumprem esse papel.

## Resumo

- `cargo fmt` aplica o estilo oficial. Use `--check` em CI.
- `cargo clippy` aponta bugs e código não idiomático. A categoria `correctness` dá erro.
- `cargo fix` e `cargo clippy --fix` aplicam correções e exigem o git limpo (ou `--allow-dirty`).
- O `rust-analyzer` mostra erros e tipos inferidos no editor.
- `rustup doc` e `cargo doc --open` funcionam offline. O Playground substitui o REPL.

## Exercícios

Veja os exercícios 3 e 4 em [`exercicios.md`](exercicios.md).

## Leitura complementar

- [The Rust Programming Language, Apêndice D: ferramentas](https://doc.rust-lang.org/stable/book/appendix-04-useful-development-tools.html)
- [Clippy: documentação](https://doc.rust-lang.org/clippy/) e [lista de lints](https://rust-lang.github.io/rust-clippy/)
- [rustfmt](https://github.com/rust-lang/rustfmt)

---

[← Lição anterior](03-cargo-essencial.md) · [Índice do módulo](README.md) · [Próxima lição →](05-lendo-erros-do-compilador.md)

# 09 — Documentação e doctests

**Objetivos.** Ao final, você vai saber:

- documentar itens com `///` e a crate ou o módulo com `//!`, nas seções convencionais;
- criar links entre itens que o `rustdoc` verifica;
- escrever *doctests*: exemplos que o `cargo test` compila e roda;
- usar `?`, linhas ocultas, `should_panic`, `no_run` e `compile_fail` nos exemplos.

## `///` e `//!`

Comentários de documentação são Markdown:

- `///` documenta o item **seguinte** (função, struct, campo, variante, módulo);
- `//!` documenta o item **que o contém**: no topo do `lib.rs`, a crate inteira; no topo de um arquivo de módulo, o módulo.

```bash
cargo doc --open      # gera e abre a documentação (com as dependências)
cargo doc --no-deps   # só a sua crate
```

A primeira linha vira o resumo nas listas, então faça dela uma frase completa. O projeto [`codigo/temperatura`](codigo/temperatura/src/lib.rs) é o exemplo desta lição.

### Seções convencionais

A comunidade usa sempre os mesmos títulos:

| Seção | Quando |
|---|---|
| `# Examples` | quase sempre: o exemplo vale mais que a descrição |
| `# Errors` | funções que devolvem `Result`: quando e qual erro |
| `# Panics` | funções que podem entrar em *panic*: em que condição |
| `# Safety` | funções `unsafe`: o que quem chama precisa garantir (Módulo 15) |

```rust
/// Lê uma temperatura em Celsius, aceitando espaços e um `°C` opcional no fim.
///
/// Veja também [`para_fahrenheit`].
///
/// # Errors
///
/// Devolve o [`ParseFloatError`] do `parse` se o que sobrar não for um número.
///
/// # Examples
///
/// ```
/// use temperatura::ler;
///
/// assert_eq!(ler(" 21.5 °C")?, 21.5);
/// assert!(ler("quente").is_err());
/// # Ok::<(), std::num::ParseFloatError>(())
/// ```
pub fn ler(texto: &str) -> Result<f64, ParseFloatError> {
```

Documente o que a assinatura **não** diz: pré-condições, o que significa cada erro, unidades, casos de borda. "Recebe um `&str` e devolve um `Result`" é ruído.

O clippy cobra as seções no grupo pedante. Sem o `# Errors` acima:

```text
warning: docs for function returning `Result` missing `# Errors` section
  --> src/lib.rs:41:1
   |
41 | pub fn ler(texto: &str) -> Result<f64, ParseFloatError> {
   | ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
   |
   = help: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.99.0/index.html#missing_errors_doc
   = note: `-W clippy::missing-errors-doc` implied by `-W clippy::pedantic`
```

E o lint `missing_docs` cobra a documentação de todo item público. O `temperatura` liga no `Cargo.toml`:

```toml
[lints.rust]
missing_docs = "warn"
```

```text
warning: missing documentation for a function
  --> src/lib.rs:47:1
   |
47 | pub fn sem_doc() {}
   | ^^^^^^^^^^^^^^^^
```

### Links entre itens

Entre colchetes e crases, o nome de um item vira link: ``[`para_fahrenheit`]``, ``[`Kelvin::new`]``, ``[`ParseFloatError`]`` (qualquer nome em escopo no arquivo). Também dá para escrever `[o texto](crate::modulo::Item)`. O `rustdoc` resolve os nomes como o compilador resolve um `use`, e avisa se não achar:

```text
warning: unresolved link to `para_kelvin`
   --> src/lib.rs:164:19
    |
164 | /// Veja também [`para_kelvin`].
    |                   ^^^^^^^^^^^ no item named `para_kelvin` in scope
    |
    = help: to escape `[` and `]` characters, add '\' before them like `\[` or `\]`
    = note: `#[warn(rustdoc::broken_intra_doc_links)]` on by default
```

A armadilha clássica é uma linha de uso de CLI num `//!`. Os colchetes de `[ARQUIVO]` viram uma tentativa de link:

```text
warning: unresolved link to `ARQUIVO`
 --> src/main.rs:1:22
  |
1 | //! uso: sobin [-v] [ARQUIVO]
  |                      ^^^^^^^ no item named `ARQUIVO` in scope
```

Coloque linhas assim num bloco ` ```text `, ou escape com `\[`.

## Doctests

Todo bloco de código na documentação, sem linguagem ou marcado ` ```rust `, é um **teste**. O `cargo test` extrai cada um, transforma num programa e roda. Um exemplo que deixou de compilar ou de passar quebra o build de testes, então a documentação não envelhece calada.

Para cada bloco, o `rustdoc`:

1. acrescenta um `extern crate temperatura;` (o exemplo usa a crate como uma dependência externa);
2. embrulha o código num `fn main() { ... }`, se ele não tiver um;
3. compila e executa. Um *panic*, como um `assert_eq!` que falha, reprova o teste.

Por ser uma crate externa, o exemplo só enxerga a **API pública**. Um doctest numa função privada não compila:

```text
---- src/lib.rs - arredondar (line 141) stdout ----
error[E0603]: function `arredondar` is private
   --> src/lib.rs:143:25
    |
143 | assert_eq!(temperatura::arredondar(21.46), 21.5);
    |                         ^^^^^^^^^^ private function
```

A saída do `cargo test` no `temperatura` tem um bloco `Doc-tests`, com o nome de cada exemplo e a linha onde ele começa:

```text
   Doc-tests temperatura

running 8 tests
test src/lib.rs - ler_arquivo (line 83) - compile ... ok
test src/lib.rs - Kelvin::para_celsius (line 119) ... ok
test src/lib.rs - ler (line 38) ... ok
test src/lib.rs - media (line 64) ... ok
test src/lib.rs - Kelvin::new (line 107) ... ok
test src/lib.rs - (line 6) ... ok
test src/lib.rs - para_fahrenheit (line 20) ... ok
test src/lib.rs - media (line 58) - should panic ... ok

test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


running 1 test
test src/lib.rs - Kelvin (line 98) - compile fail ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s

all doctests ran in 0.51s; merged doctests compilation took 0.16s
```

Para quem vem de Python, é o `doctest` (exemplos na docstring que viram teste), com uma diferença: em Rust ele roda sempre, junto com os outros testes. O Doxygen do C++ gera documentação, mas não executa os exemplos.

> [!NOTE]
> Doctests só existem para **bibliotecas**. Num pacote só com `main.rs`, os exemplos da documentação não rodam: um `assert_eq!(1, 2)` ali passa despercebido. Mais um motivo para pôr a lógica no `lib.rs`.

### Linhas ocultas e o `?`

Uma linha que começa com `# ` compila, mas não aparece na documentação. Serve para preparar o exemplo sem poluir o que o leitor vê.

O uso mais comum é o `?`. O `fn main()` gerado devolve `()`, e o `?` não funciona nele:

```text
---- src/lib.rs - ler (line 38) stdout ----
error[E0277]: the `?` operator can only be used in a function that returns `Result` or `Option` (or another type that implements `FromResidual`)
  --> src/lib.rs:42:27
   |
39 | fn main() { #[allow(non_snake_case)] fn _doctest_main_src_lib_rs_38_0() {
   |                                      ---------------------------------- this function should return `Result` or `Option` to accept `?`
...
42 | assert_eq!(ler(" 21.5 °C")?, 21.5);
   |                           ^ cannot use the `?` operator in a function that returns `()`
```

A mensagem mostra o `main` gerado. A solução: terminar o exemplo com uma expressão `Ok`, oculta. Quando o bloco termina num `Result`, o `rustdoc` faz o `main` devolvê-lo. O tipo do erro precisa ser escrito, porque não há nada de onde inferir:

```rust
/// assert_eq!(ler(" 21.5 °C")?, 21.5);
/// # Ok::<(), std::num::ParseFloatError>(())
```

O exemplo na documentação fica limpo, com `?` em vez de `unwrap`, que é o que você quer que o leitor copie.

### Atributos do bloco

| Bloco | O que o teste faz |
|---|---|
| ` ``` ` | compila e roda; passa se não houver *panic* |
| ` ```should_panic ` | compila e roda; passa **só** se houver *panic* |
| ` ```no_run ` | só compila (para exemplos que acessam rede, arquivos, ou demoram) |
| ` ```compile_fail ` | passa **só** se **não** compilar |
| ` ```ignore ` | não faz nada (quase nunca é o que você quer) |
| ` ```text ` | não é Rust; nem compila |

`should_panic` documenta a seção `# Panics`:

```rust
/// # Panics
///
/// Entra em *panic* se `leituras` estiver vazia: média de nada é um bug de
/// quem chamou, não uma condição esperada.
///
/// ```should_panic
/// temperatura::media(&[]);
/// ```
```

`compile_fail` documenta o que o sistema de tipos impede. O `Kelvin` é um newtype com campo privado ([M04-08](../04-structs-enums-e-patterns/08-estados-invalidos-irrepresentaveis.md)), e o exemplo prova que não dá para criá-lo sem validar:

```rust
/// O campo é privado, então a única forma de criar um `Kelvin` é por
/// [`Kelvin::new`], que valida. Por isso isto não compila:
///
/// ```compile_fail
/// let k = temperatura::Kelvin(-10.0);
/// ```
pub struct Kelvin(f64);
```

> [!WARNING]
> `compile_fail` passa com **qualquer** erro de compilação, inclusive um erro de digitação. Um exemplo com `temperatura::Kelvn::new(5.0)` (falta o `i`) também passa. Material antigo usa ` ```compile_fail,E0603 `, mas na versão estável o código do erro não é conferido. Mantenha esses exemplos curtos e confira a mensagem à mão de vez em quando.

### Doctests combinados (edition 2024)

Antes da 2024, cada doctest virava um executável separado, e documentação com muitos exemplos deixava o `cargo test` lento. Na edition 2024, os doctests compatíveis são **compilados juntos** num único binário (cada um ainda roda num processo próprio). É a linha `merged doctests compilation took ...` no fim da saída.

Ficam de fora sozinhos os que não podem ser combinados: `compile_fail` (o segundo bloco `running 1 test` acima), os que têm atributos de crate (`#![...]`) e alguns casos com macros. Para forçar um exemplo a ficar separado, marque ` ```standalone_crate `.

Um efeito colateral: quando um doctest combinado falha, a linha do *panic* na mensagem pode estar errada.

```text
---- src/lib.rs - ebulicao (line 150) stdout ----
Test executable failed (exit status: 101).
stderr:
thread 'main' (167726) panicked at src/lib.rs:5:1:
assertion `left == right` failed
  left: 99.97
 right: 100.0
```

O exemplo começa na linha 150, mas a mensagem diz `5:1`. Confie no **nome** do teste (`ebulicao (line 150)`), não na linha do *panic*.

Para rodar só os doctests: `cargo test --doc`. O Módulo 09 mostra onde eles entram na organização dos testes.

## Armadilhas comuns

- **Doctest de item privado** (E0603): o exemplo é uma crate externa. Documente a API pública.
- **`?` sem o `Ok` final** (E0277): termine com `# Ok::<(), TipoDoErro>(())`.
- **Colchetes no texto** (`[ARQUIVO]`): viram link quebrado. Use um bloco `text`.
- **`compile_fail` que passa pelo motivo errado**: o código do erro não é conferido.
- **Doctests num pacote só binário**: não rodam.
- **`ignore` para "resolver" um exemplo quebrado**: prefira `no_run`, ou conserte o exemplo.

## Resumo

- `///` documenta o item seguinte; `//!`, o que contém. Seções `# Examples`, `# Errors`, `# Panics`, `# Safety`.
- Links ``[`Item`]`` são verificados; `missing_docs` e o clippy pedante cobram documentação.
- Cada bloco de código vira um teste que usa só a API pública, rodado pelo `cargo test`.
- Linhas com `# ` ficam ocultas; `# Ok::<(), E>(())` habilita o `?`.
- `should_panic`, `no_run` e `compile_fail` documentam *panics*, efeitos externos e o que os tipos impedem.
- Na 2024, os doctests são compilados juntos; a linha do *panic* pode enganar.

## Exercícios

Veja o exercício 6 em [`exercicios.md`](exercicios.md).

## Leitura complementar

- [The rustdoc book: Documentation tests](https://doc.rust-lang.org/rustdoc/write-documentation/documentation-tests.html)
- [The rustdoc book: Linking to items by name](https://doc.rust-lang.org/rustdoc/write-documentation/linking-to-items-by-name.html)
- [Edition Guide: Rustdoc combined tests](https://doc.rust-lang.org/edition-guide/rust-2024/rustdoc-doctests.html)
- [Rust API Guidelines: Documentation](https://rust-lang.github.io/api-guidelines/documentation.html)
- [Effective Rust, Item 27: Document public interfaces](https://effective-rust.com/documentation.html)

---

[← Lição anterior](08-rust-version-e-resolver-3.md) · [Índice do módulo](README.md) · [Próxima lição →](10-publicacao-e-cargo-install.md)

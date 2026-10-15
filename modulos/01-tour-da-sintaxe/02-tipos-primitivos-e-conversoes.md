# 02 — Tipos primitivos e conversões

**Objetivos.** Ao final, você vai saber:

- quais são os tipos numéricos, `bool` e `char`, e qual tipo o compilador escolhe quando você não diz;
- o que acontece num overflow de inteiro e como escolher o comportamento;
- por que não existe conversão implícita entre números, e como converter com `as`, `From` e `TryFrom`;
- converter texto em número com `parse`.

## Os tipos escalares

| Categoria | Tipos | Observação |
|---|---|---|
| inteiros com sinal | `i8`, `i16`, `i32`, `i64`, `i128` | o número é a quantidade de bits |
| inteiros sem sinal | `u8`, `u16`, `u32`, `u64`, `u128` | `u8` é o "byte" |
| tamanho de ponteiro | `isize`, `usize` | `usize` é o tipo dos **índices e tamanhos** (`len()` devolve `usize`) |
| ponto flutuante | `f32`, `f64` | IEEE 754, como `float`/`double` |
| lógico | `bool` | `true` ou `false` |
| caractere | `char` | um valor Unicode, **4 bytes** |

Os literais aceitam `_` como separador e prefixos de base:

```rust
let milhao = 1_000_000;
let hexa = 0xff;       // 255
let binario = 0b1010;  // 10
let byte = b'A';       // u8: 65
let pequeno = 57u8;    // sufixo define o tipo
let f = 3.0_f32;
```

O `char` merece atenção de quem vem de C++: ele não é um byte, e sim um *Unicode scalar value* (um "caractere" Unicode qualquer). `'ç'` e `'🦀'` são `char` válidos, e todo `char` ocupa 4 bytes. Texto (`&str`/`String`) é outra história, guardado em UTF-8, e fica para o [Módulo 02](../02-ownership/README.md).

## Inferência não é tipagem dinâmica

Você raramente escreve o tipo de uma variável local:

```rust
let a = 10;        // i32
let b = 2.5;       // f64
let c: u64 = 10;   // anotado
let d = 10u8;      // pelo sufixo
```

O compilador descobre o tipo **na compilação**, olhando como o valor é usado. O código gerado é idêntico ao da versão anotada: cada variável continua tendo um tipo fixo, só que você não precisou escrevê-lo.

Quando nada restringe o tipo, um inteiro vira `i32` e um float vira `f64`. Enquanto o tipo ainda não foi decidido, as mensagens de erro mostram `{integer}` ou `{float}`. Se a mesma variável depois for passada para uma função que pede `u8`, ela será `u8` desde o começo: a inferência olha a função inteira, e não só a linha do `let`.

## Overflow: definido e barulhento

Um `u8` vai de 0 a 255. O que acontece com `255 + 1`?

Se o compilador consegue calcular o valor na compilação, o build quebra:

```rust
// ❌ NÃO COMPILA
fn main() {
    let x: u8 = 255 + 1;
    println!("{x}");
}
```

```text
error: this arithmetic operation will overflow
 --> src/main.rs:2:17
  |
2 |     let x: u8 = 255 + 1;
  |                 ^^^^^^^ attempt to compute `u8::MAX + 1_u8`, which would overflow
  |
  = note: `#[deny(arithmetic_overflow)]` on by default
```

Esse é um lint *deny-by-default*, como os que você viu no [Módulo 00](../00-introducao/README.md): não tem código `E`, mas bloqueia o build.

Quando o valor só aparece em tempo de execução, o comportamento depende do perfil de compilação:

```rust
// 💥 PANIC EM RUNTIME (no perfil dev)
fn incrementa(x: u8) -> u8 {
    x + 1
}

fn main() {
    let limite: u8 = "255".parse().unwrap();
    println!("{}", incrementa(limite));
}
```

```text
$ cargo run
thread 'main' (183327) panicked at src/main.rs:2:5:
attempt to add with overflow
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

$ cargo run --release
0
```

- **Perfil `dev`** (`cargo run`, `cargo test`): o programa para com *panic* (um erro irrecuperável que encerra a thread).
- **Perfil `release`**: a verificação é desligada por desempenho, e o valor dá a volta em complemento de dois (255 + 1 = 0).

Quem decide é a opção `overflow-checks` do perfil, ligada por padrão só no `dev`. Nos dois casos o comportamento é **definido**. Compare:

- **C/C++:** overflow de inteiro *sem sinal* dá a volta em silêncio. Com sinal (`int`), é **comportamento indefinido**: o compilador pode assumir que nunca acontece e otimizar em cima disso.
- **Python:** `int` não tem limite (`2**64 + 1` funciona), então não existe overflow, mas você paga em desempenho e memória.

Quando o overflow é uma possibilidade real, não dependa do perfil: diga o que quer. Todo tipo inteiro tem quatro famílias de métodos:

```rust
let x: u8 = 250;
println!("{:?}", x.checked_add(10));     // None: não coube
println!("{:?}", x.checked_add(5));      // Some(255)
println!("{}", x.wrapping_add(10));      // 4: dá a volta
println!("{}", x.saturating_add(10));    // 255: para no limite
println!("{:?}", x.overflowing_add(10)); // (4, true): resultado + "estourou?"
```

Existem as mesmas variantes para `sub`, `mul`, `div`, `pow` e outras. O `checked_*` devolve um `Option`: `Some(valor)` quando deu certo e `None` quando não deu. Você vai ver `Option` com frequência a partir da [lição 07](07-kit-vec-e-hashmap.md). O tipo em si é assunto do [Módulo 04](../04-structs-enums-e-patterns/README.md).

Código completo, para rodar nos dois perfis: [`codigo/overflow`](codigo/overflow/).

## Sem conversão implícita entre números

Em Rust, `i32 + i64` não compila. Nenhum número vira outro tipo sem você pedir, nem mesmo de um tipo menor para um maior:

```rust
// ❌ NÃO COMPILA
fn main() {
    let a: i32 = 10;
    let b: i64 = 20;
    let c = a + b;
    println!("{c}");
}
```

```text
error[E0308]: mismatched types
 --> src/main.rs:4:17
  |
4 |     let c = a + b;
  |                 ^ expected `i32`, found `i64`

error[E0277]: cannot add `i64` to `i32`
   --> src/main.rs:4:15
    |
  4 |     let c = a + b;
    |               ^ no implementation for `i32 + i64`
    |
    = help: the trait `Add<i64>` is not implemented for `i32`
help: the following other types implement trait `Add<Rhs>`
...
```

Saem dois erros para o mesmo problema. O segundo continua com uma lista longa de trechos da biblioteca padrão, que dá para ignorar (veja a [lição 05 do Módulo 00](../00-introducao/05-lendo-erros-do-compilador.md)). O recado é um só: os dois lados da soma precisam ter o mesmo tipo.

Em C++, `int x = 300; unsigned char y = x;` compila e guarda 44 (no máximo com um aviso de `-Wconversion`). Rust prefere obrigar você a escrever a conversão, e há três jeitos.

### `From`: a conversão que nunca perde nada

```rust
let a: i32 = 10;
let b: i64 = 20;
let c = i64::from(a) + b; // 30
```

`i64::from(...)` só existe quando a conversão é sempre segura (todo `i32` cabe num `i64`). O par dela é o `.into()`, que faz a mesma coisa com o tipo de destino inferido:

```rust
let a: i32 = 10;
let b: i64 = a.into();
```

Se a conversão puder perder dados, ela simplesmente não existe:

```rust
// ❌ NÃO COMPILA
fn main() {
    let grande: i32 = 300;
    let pequeno: u8 = grande.into();
    println!("{pequeno}");
}
```

```text
error[E0277]: the trait bound `u8: From<i32>` is not satisfied
    --> src/main.rs:3:30
     |
   3 |     let pequeno: u8 = grande.into();
     |                              ^^^^ the trait `From<i32>` is not implemented for `u8`
...
     = note: required for `i32` to implement `Into<u8>`
```

### `TryFrom`: a conversão que pode falhar

```rust
println!("{:?}", u8::try_from(300_i32)); // Err(TryFromIntError(PosOverflow))
println!("{:?}", u8::try_from(200_i32)); // Ok(200)
```

O resultado é um `Result`: `Ok(valor)` ou `Err(erro)`. Tratar erros é o assunto do [Módulo 05](../05-tratamento-de-erros/README.md). Por enquanto, basta saber que a falha vem no valor de retorno, sem exceção.

`From`, `Into` e `TryFrom` são *traits* (conjuntos de comportamentos que um tipo implementa), estudadas no Módulo 06.

### `as`: a conversão que nunca falha

```rust
println!("{}", 300_i32 as u8);     // 44: trunca os bits
println!("{}", -1_i32 as u32);     // 4294967295: reinterpreta o sinal
println!("{}", 3.99_f64 as i32);   // 3: trunca em direção ao zero
println!("{}", -3.99_f64 as i32);  // -3
println!("{}", 1e20_f64 as i32);   // 2147483647: satura no máximo
println!("{}", f64::NAN as i32);   // 0
println!("{}", 'A' as u32);        // 65
println!("{}", 97_u8 as char);     // a
```

O `as` sempre produz um valor, e por isso pode perder dados em silêncio. A diferença para o C++ é que, em Rust, a conversão fica **escrita** no código, e dá para procurar por ela.

A regra prática:

| Situação | Use |
|---|---|
| sempre cabe (`i32` → `i64`, `u8` → `u32`) | `i64::from(x)` ou `x.into()` |
| pode não caber, e você quer saber | `u8::try_from(x)` |
| você **sabe** que cabe, ou quer truncar de propósito; float ↔ inteiro | `x as u8` |

## Texto para número: `parse`

```rust
let n: i32 = "42".parse().unwrap();
let m = "7".parse::<u64>().unwrap();
```

O `parse` converte para qualquer tipo numérico, então ele precisa saber **qual**. Você informa pelo tipo da variável ou pelo *turbofish* `::<u64>` (o nome vem do formato `::<>`, que lembra um peixe). Sem nenhum dos dois:

```rust
// ❌ NÃO COMPILA
fn main() {
    let n = "42".parse().unwrap();
    println!("{n}");
}
```

```text
error[E0284]: type annotations needed
    --> src/main.rs:2:9
     |
   2 |     let n = "42".parse().unwrap();
     |         ^        ----- type must be known at this point
     |
     = note: cannot satisfy `<_ as FromStr>::Err == _`
     = note: the type must also implement `FromStr`
...
help: consider giving `n` an explicit type
     |
   2 |     let n: /* Type */ = "42".parse().unwrap();
     |          ++++++++++++
```

O `parse` devolve um `Result`, porque o texto pode não ser um número. O `.unwrap()` extrai o valor e dá *panic* se for erro. É aceitável em exemplos e exercícios. No Módulo 05, você vai aprender a tratar o erro direito.

## Armadilhas comuns

- **Misturar tipos numéricos numa conta** (E0308 + E0277): converta um dos lados, de preferência com `from`.
- **Usar `as` para "resolver" o erro de tipo sem pensar.** `as` nunca falha, então um valor fora da faixa vira lixo silencioso. Se o valor pode não caber, use `try_from`.
- **Confiar no comportamento de overflow do release.** O *wrap* do release não é um recurso, e o mesmo código dá *panic* nos testes. Se o *wrap* é intencional, escreva `wrapping_add`.
- **`parse` sem tipo** (E0284): anote a variável ou use o *turbofish*.

## Resumo

- Inteiros (`i8`…`i128`, `u8`…`u128`, `isize`/`usize`), floats (`f32`, `f64`), `bool` e `char` (Unicode, 4 bytes).
- Sem restrição, o inteiro vira `i32` e o float vira `f64`. A inferência acontece na compilação.
- Overflow é definido: *panic* em debug e volta em release. Para escolher, use `checked_*`, `wrapping_*`, `saturating_*` ou `overflowing_*`.
- Não há conversão implícita entre números. Use `From`/`into` (sem perda), `TryFrom` (pode falhar) ou `as` (sempre "funciona", pode perder dados).
- `parse` precisa do tipo de destino: `let n: i32 = s.parse().unwrap();`.

## Exercícios

Veja os exercícios 1 e 4 em [`exercicios.md`](exercicios.md).

## Leitura complementar

- [The Rust Programming Language, 3.2 Data Types](https://doc.rust-lang.org/stable/book/ch03-02-data-types.html)
- [Comprehensive Rust: Arithmetic](https://google.github.io/comprehensive-rust/types-and-values/arithmetic.html) e [Type Inference](https://google.github.io/comprehensive-rust/types-and-values/inference.html)
- [Rust Reference: casts com `as`](https://doc.rust-lang.org/reference/expressions/operator-expr.html#type-cast-expressions)
- [std: módulo `convert`](https://doc.rust-lang.org/std/convert/index.html)
- [The Cargo Book: `overflow-checks`](https://doc.rust-lang.org/cargo/reference/profiles.html#overflow-checks)

---

[← Lição anterior](01-variaveis-e-mutabilidade.md) · [Índice do módulo](README.md) · [Próxima lição →](03-expressoes-blocos-e-funcoes.md)

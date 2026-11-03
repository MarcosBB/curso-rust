# 05 — Chamando C a partir de Rust

**Objetivos.** Ao final, você vai saber:

- declarar funções de C com `unsafe extern "C"` e marcar as que podem ser `safe`;
- mapear tipos entre as linguagens: `c_int`, `#[repr(C)]`, `CStr`, `CString` e literais `c"..."`;
- compilar um arquivo `.c` junto com o crate usando um `build.rs` e a crate `cc`;
- embrulhar as chamadas numa API segura, incluindo memória alocada pelo C e *callbacks*;
- o que o `bindgen` automatiza.

## FFI em uma frase

*FFI* (*foreign function interface*) é o mecanismo para chamar código de outra linguagem. Para Rust, essa outra linguagem é quase sempre C, ou algo que fala a ABI de C (C++ com `extern "C"`, Zig, Fortran, bibliotecas do sistema). A *ABI* (*application binary interface*) é o contrato de baixo nível: em que registradores vão os argumentos, como as structs ficam na memória, quem limpa a pilha. Rust não tem ABI estável (Módulo 08), mas fala a de C quando você pede com `extern "C"`.

## Declarando funções de C

O Rust já liga a libc em todo programa, então dá para começar com ela:

```rust
use std::ffi::{CStr, c_char};

// Funções da libc (e da libm), que o Rust já liga em todo programa.
unsafe extern "C" {
    // `sqrt` aceita qualquer `f64` e não mexe em memória: pode ser `safe`.
    safe fn sqrt(x: f64) -> f64;
    // `strlen` lê através do ponteiro: fica `unsafe` (o padrão).
    fn strlen(s: *const c_char) -> usize;
}

fn main() {
    println!("sqrt(2) = {}", sqrt(2.0));

    let texto: &CStr = c"unsafe"; // literal de string C, com '\0' no fim
    // SAFETY: `texto` é uma string C válida e terminada em '\0'.
    let n = unsafe { strlen(texto.as_ptr()) };
    println!("strlen = {n}");
}
```

```text
sqrt(2) = 1.4142135623730951
strlen = 6
```

Três pontos da edition 2024 aparecem aqui:

- **O bloco é `unsafe extern`.** O compilador não tem como conferir se a assinatura que você escreveu bate com a função de C. Se não bater, é UB (veja mais abaixo). O `unsafe` deixa claro que essa promessa é sua. Sem ele, a 2024 dá erro:

```text
error: extern blocks must be unsafe
 --> src/main.rs:1:1
  |
1 | / extern "C" {
2 | |     fn abs(x: i32) -> i32;
3 | | }
  | | ^ help: needs `unsafe` before the extern keyword: `unsafe`
  | |_|
  |
```

- **Cada item é `unsafe` por padrão.** Chamar `strlen` exige um bloco `unsafe`, porque um ponteiro inválido faz a função ler memória qualquer.
- **`safe fn` marca o que não tem pré-condição.** `sqrt` aceita qualquer `f64`, então pode ser chamada sem `unsafe`. Mas `safe` é uma promessa, e não uma verificação. O Rust Book usa `safe fn abs(input: i32) -> i32` como exemplo, e vale desconfiar dele: em C, `abs(INT_MIN)` é UB, porque o resultado não cabe num `int`. Uma declaração honesta deixaria `abs` como `unsafe`, e o wrapper excluiria `i32::MIN` (o exercício 1 faz isso).

## Tipos na fronteira

| C | Rust |
|---|---|
| `int`, `unsigned`, `long`, `char` | `c_int`, `c_uint`, `c_long`, `c_char` (em `std::ffi`) |
| `int32_t`, `uint8_t`, `int64_t` | `i32`, `u8`, `i64` |
| `size_t` | `usize` |
| `double`, `float` | `f64`, `f32` |
| `bool` (C99) | `bool` |
| `T*`, `const T*` | `*mut T`, `*const T` |
| `const char*` terminada em `'\0'` | `*const c_char` (de um `CStr`/`CString`) |
| `struct` | struct Rust com `#[repr(C)]` |
| ponteiro de função `int (*f)(int)` | `extern "C" fn(c_int) -> c_int` |
| `void*` | `*mut c_void` |

Use `c_int`, `c_long` e afins quando a declaração em C usa esses tipos: o tamanho deles muda com a plataforma (`long` tem 64 bits no Linux e no macOS, e 32 no Windows). Quando o C usa `int32_t` e companhia, use os tipos Rust de tamanho fixo.

### `#[repr(C)]`

O compilador do Rust reordena os campos de uma struct para economizar espaço, e não garante layout nenhum (o Módulo 04 mostrou isso com enums). Para uma struct atravessar a fronteira, ela precisa do layout de C:

```rust
/// Mesmo layout que `Ponto` em C, graças ao `#[repr(C)]`.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Ponto {
    pub x: f64,
    pub y: f64,
}
```

`#[repr(C)]` põe os campos na ordem declarada, com o mesmo preenchimento que um compilador C usaria. Também existe para enums (`#[repr(C)]` ou `#[repr(u8)]`, `#[repr(i32)]`...), com tags de tamanho fixo. Um enum Rust com dados (`Option<String>`, por exemplo) não tem equivalente em C e não deve atravessar a fronteira.

### Strings: `CStr` e `CString`

Strings de C são sequências de bytes terminadas em `'\0'`, sem tamanho guardado. Strings de Rust (`&str`, `String`) têm tamanho, são UTF-8 e podem conter `'\0'` no meio. Por isso existem dois tipos-ponte em `std::ffi`, num par como `&str`/`String`:

| | emprestado | com posse |
|---|---|---|
| Rust | `&str` | `String` |
| C (terminada em `'\0'`) | `&CStr` | `CString` |

- **Rust → C:** `CString::new("texto")` copia os bytes e acrescenta o `'\0'`. Ele devolve `Err(NulError)` se o texto tiver um `'\0'` no meio, porque o C o leria como o fim da string. `.as_ptr()` dá o `*const c_char`.
- **C → Rust:** `unsafe { CStr::from_ptr(p) }` olha a string C sem copiar; `.to_str()` valida o UTF-8, e `.to_string_lossy()` troca bytes inválidos por `�`.
- **Literais:** `c"unsafe"` (desde a 1.77) é um `&'static CStr` pronto, com o `'\0'` no fim, verificado na compilação.

## Compilando C com `cc` e `build.rs`

Para usar o seu próprio código C, ele precisa ser compilado e ligado ao crate. O Cargo faz isso com um *build script*: um arquivo `build.rs` na raiz do pacote, que o Cargo compila e roda **antes** de compilar o crate. A crate `cc` cuida da parte chata: encontrar o compilador C do sistema (clang, gcc ou MSVC), passar as flags certas para o alvo, gerar a biblioteca estática e avisar o Cargo para ligá-la.

```bash
cargo add --build cc
```

```toml
[build-dependencies]
cc = "1.11"
```

A estrutura do projeto [`codigo/chamando-c`](codigo/chamando-c/):

```text
chamando-c/
├── Cargo.toml
├── build.rs
├── csrc/
│   └── estatisticas.c
└── src/
    ├── lib.rs
    └── main.rs
```

```rust
// build.rs
// Compila csrc/estatisticas.c numa biblioteca estática e pede ao Cargo para
// ligá-la ao crate. O `cc` encontra o compilador C do sistema (clang, gcc ou
// MSVC) e repassa as flags certas para o alvo.
fn main() {
    cc::Build::new()
        .file("csrc/estatisticas.c")
        .warnings(true)
        .compile("estatisticas");
    // Recompila só se o arquivo C mudar.
    println!("cargo::rerun-if-changed=csrc/estatisticas.c");
}
```

O C, resumido (o arquivo completo está em [`csrc/estatisticas.c`](codigo/chamando-c/csrc/estatisticas.c)):

```c
typedef struct { double x; double y; } Ponto;

double media(const double *valores, size_t n);
double distancia(Ponto a, Ponto b);
void aplicar(int32_t *v, size_t n, int32_t (*f)(int32_t));

/* Devolve uma string nova (alocada com malloc). Libere com `liberar_texto`. */
char *saudacao(const char *nome);
void liberar_texto(char *texto);
```

Não é preciso nenhum `#[link]`: o `cc` já emite as instruções de ligação. Para ligar uma biblioteca do sistema que já existe (`libz`, `libsqlite3`), usa-se `println!("cargo::rustc-link-lib=z")` no `build.rs`, ou `#[link(name = "z")]` no bloco `extern`.

## Declarações cruas, API segura

O padrão é separar em duas camadas: um módulo privado `ffi` com as declarações exatamente como em C, e funções públicas seguras que convertem os tipos e justificam cada `unsafe`.

```rust
mod ffi {
    use super::Ponto;
    use std::ffi::c_char;

    // O `unsafe` do bloco diz: "eu garanto que estas assinaturas batem com o
    // código C". O compilador não tem como conferir.
    unsafe extern "C" {
        // Recebe tudo por valor e não tem pré-condição: pode ser `safe`.
        pub safe fn distancia(a: Ponto, b: Ponto) -> f64;
        // As outras recebem ponteiros e ficam `unsafe` (o padrão).
        pub fn media(valores: *const f64, n: usize) -> f64;
        pub fn aplicar(v: *mut i32, n: usize, f: extern "C" fn(i32) -> i32);
        pub fn saudacao(nome: *const c_char) -> *mut c_char;
        pub fn liberar_texto(texto: *mut c_char);
    }
}

/// Média dos valores (0.0 para um slice vazio).
pub fn media(valores: &[f64]) -> f64 {
    // SAFETY: o ponteiro e o tamanho vêm do mesmo slice, válido durante a
    // chamada. A função C só lê `n` elementos e não guarda o ponteiro.
    unsafe { ffi::media(valores.as_ptr(), valores.len()) }
}
```

Quem usa a biblioteca chama `media(&[7.0, 8.0, 9.5])`, sem saber que há C por baixo. Um slice já é exatamente o par (ponteiro, tamanho) que o C espera, e o `&[f64]` garante que os dois combinam.

Repare no detalhe do `SAFETY`: *"não guarda o ponteiro"*. Se a função C guardasse o ponteiro para usar depois (como uma biblioteca de callbacks assíncronos faz), o slice emprestado poderia ser liberado antes, e o wrapper precisaria de outra assinatura. Ler a documentação do lado C faz parte do `unsafe`.

## Quem libera a memória

`saudacao` devolve uma string alocada pelo C com `malloc`. A regra de ouro de FFI: **memória volta para quem a alocou**. O alocador do Rust e o `malloc` do C não são necessariamente o mesmo, então você nunca libera com `drop` algo que veio do `malloc`, nem com `free` algo que veio de um `Box`.

```rust
/// Saudação montada pelo C. Falha se `nome` tiver um byte zero no meio.
pub fn saudacao(nome: &str) -> Result<String, NulError> {
    let nome_c = CString::new(nome)?; // acrescenta o '\0' final
    // SAFETY: `nome_c` é uma string C válida, terminada em '\0', viva até o
    // fim desta função.
    let cru = unsafe { ffi::saudacao(nome_c.as_ptr()) };
    assert!(!cru.is_null(), "saudacao: sem memória");
    // SAFETY: `cru` não é nulo e aponta para uma string terminada em '\0'
    // criada pelo C. Copiamos o conteúdo para uma `String` antes de liberar.
    let texto = unsafe { CStr::from_ptr(cru) }
        .to_string_lossy()
        .into_owned();
    // SAFETY: `cru` veio de `saudacao` e é liberado uma vez, pela função C
    // correspondente (o `free` do C, e não o alocador do Rust).
    unsafe { ffi::liberar_texto(cru) };
    Ok(texto)
}
```

O wrapper copia o texto para uma `String` do Rust e devolve a memória ao C na hora. Quem chama recebe um tipo comum, com dono comum. Se a cópia fosse cara, a alternativa seria um tipo próprio (`TextoC`) com `Drop` chamando `liberar_texto`: RAII sobre um recurso do C, como no Módulo 02.

## Callbacks: ponteiros de função

`aplicar` recebe um ponteiro de função de C. Do lado Rust, isso é uma função `extern "C"`:

```rust
extern "C" fn dobrar(x: i32) -> i32 {
    x * 2
}

let mut v = [1, 2, 3];
aplicar(&mut v, dobrar);
println!("aplicar(dobrar) = {v:?}");
```

```text
aplicar(dobrar) = [2, 4, 6]
```

O Módulo 07 avisou que ponteiros `fn` fazem sentido na fronteira com C. Este é o motivo: um ponteiro de função de C é só um endereço de código, e uma closure que captura variáveis precisa carregar essas variáveis junto. Não cabe:

```rust
// ❌ NÃO COMPILA
fn aplicar(v: &mut [i32], f: extern "C" fn(i32) -> i32) {
    for x in v.iter_mut() {
        *x = f(*x);
    }
}

fn main() {
    let fator = 3;
    let mut v = [1, 2, 3];
    aplicar(&mut v, |x| x * fator);
    println!("{v:?}");
}
```

```text
error[E0308]: mismatched types
  --> src/main.rs:10:21
   |
10 |     aplicar(&mut v, |x| x * fator);
   |     -------         ^^^^^^^^^^^^^ expected fn pointer, found closure
   |     |
   |     arguments to this function are incorrect
   |
   = note: expected fn pointer `extern "C" fn(i32) -> i32`
                 found closure `{closure@src/main.rs:10:21: 10:24}`
note: closures can only be coerced to `fn` types if they do not capture any variables
  --> src/main.rs:10:29
   |
10 |     aplicar(&mut v, |x| x * fator);
   |                             ^^^^^ `fator` captured here
```

APIs de C resolvem isso com um parâmetro extra, o `void *contexto` (ou `user_data`), que o C repassa intocado ao callback. Do lado Rust, você passa um ponteiro para os dados da closure e, dentro de uma função `extern "C"` genérica, converte de volta. É um padrão comum em bindings de bibliotecas de GUI e de eventos.

Um último cuidado: **um panic dentro de um callback chamado pelo C aborta o processo** (lição 06). Callbacks não devem dar panic.

## Quando a assinatura está errada

O `unsafe extern` não é burocracia. Troque, na declaração, `*const f64` por `*const i32` (e o wrapper para `&[i32]`), mantendo o C igual:

```rust
pub fn media(valores: *const i32, n: usize) -> f64;   // o C recebe `const double*`
```

```rust
println!("{:e}", chamando_c::media(&[1, 2, 3, 4]));
```

```text
3.182993688e-314
```

Compila, roda, sai com código 0 e devolve um número sem sentido: o C leu 4 `double`s (32 bytes) de um buffer de 4 `i32` (16 bytes), ou seja, os inteiros reinterpretados e mais 16 bytes de fora do array. Nenhuma verificação existe nessa fronteira, nem na compilação nem em execução. E o Miri não ajuda aqui: ele não executa código C.

```text
error: unsupported operation: can't call foreign function `aplicar` on OS `macos`
  --> src/lib.rs:52:14
   |
52 |     unsafe { ffi::aplicar(v.as_mut_ptr(), v.len(), f) }
   |              ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ unsupported operation occurred here
   |
   = help: this means the program tried to do something Miri does not support; it does not indicate a bug in the program
```

É por isso que, para bibliotecas com mais de meia dúzia de funções, as declarações não devem ser escritas à mão.

## `bindgen`: declarações geradas do cabeçalho

O [`bindgen`](https://rust-lang.github.io/rust-bindgen/) lê um cabeçalho `.h` (usando o libclang) e gera as declarações Rust. Normalmente ele roda no `build.rs`, e o resultado é incluído com `include!(concat!(env!("OUT_DIR"), "/bindings.rs"))`. Para o cabeçalho deste exemplo, o `bindgen` 0.73 gerou:

```rust
/* automatically generated by rust-bindgen 0.73.2 */

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct Ponto {
    pub x: f64,
    pub y: f64,
}
#[allow(clippy::unnecessary_operation, clippy::identity_op)]
const _: () = {
    ["Size of Ponto"][::std::mem::size_of::<Ponto>() - 16usize];
    ["Alignment of Ponto"][::std::mem::align_of::<Ponto>() - 8usize];
    ["Offset of field: Ponto::x"][::std::mem::offset_of!(Ponto, x) - 0usize];
    ["Offset of field: Ponto::y"][::std::mem::offset_of!(Ponto, y) - 8usize];
};
unsafe extern "C" {
    pub fn media(valores: *const f64, n: usize) -> f64;
}
...
unsafe extern "C" {
    pub fn aplicar(
        v: *mut i32,
        n: usize,
        f: ::std::option::Option<unsafe extern "C" fn(arg1: i32) -> i32>,
    );
}
```

Três coisas para notar:

- As declarações já vêm no formato da 2024 (`unsafe extern "C"`).
- O bloco `const _` confere o layout da struct **na compilação**: se o tamanho ou os deslocamentos mudarem, a indexação do array estoura e o crate não compila.
- O ponteiro de função virou `Option<unsafe extern "C" fn ...>`. Em C, um ponteiro de função pode ser `NULL`, e `Option<fn>` é como Rust representa isso, com o mesmo tamanho de um ponteiro. Na declaração à mão, usamos `extern "C" fn` direto porque o lado Rust nunca passa nulo.

O `bindgen` gera só a camada crua. A API segura continua sendo trabalho seu. No ecossistema, o costume é publicar duas crates: `algo-sys` (declarações geradas, ligação com a biblioteca C) e `algo` (a API segura em cima). É o caso de `openssl-sys`/`openssl` e `libsqlite3-sys`/`rusqlite`.

Ele não está nos projetos do curso porque exige o libclang instalado.

## Comparando com C++ e Python

Em C++, chamar C é trivial: o mesmo compilador entende o cabeçalho, e `extern "C" { #include "lib.h" }` basta. A falta de verificação é a mesma de Rust (uma declaração errada é UB), mas lá ela se mistura ao resto do código; em Rust, ela fica no módulo `ffi` e nos blocos `unsafe`.

Em Python, as opções são escrever uma extensão em C contra a API do CPython ou usar `ctypes`, que carrega uma biblioteca e chama funções declarando os tipos em tempo de execução. As duas são o assunto da lição 07, do ponto de vista oposto: o Python chamando Rust.

## Armadilhas comuns

- **`CString::new(s).unwrap().as_ptr()` numa linha só.** O `CString` temporário é descartado no fim da expressão, e o ponteiro fica pendurado. Guarde o `CString` numa variável enquanto o C usa o ponteiro. (O compilador tem um lint para esse caso, `dangling_pointers_from_temporaries`.)
- **Liberar com o alocador errado.** O que veio de `malloc` volta para `free` (ou para a função de liberação da biblioteca); o que veio de `Box::into_raw` volta para `Box::from_raw`.
- **`safe fn` otimista.** `safe` só cabe em funções sem nenhuma pré-condição. Ponteiros, índices, valores proibidos (`abs(INT_MIN)`) e estado global são pré-condições.
- **Struct sem `#[repr(C)]`.** Compila, e o C lê os campos na ordem errada. Toda struct que atravessa a fronteira precisa de `#[repr(C)]`.
- **Esquecer o `rerun-if-changed`.** Sem ele, o Cargo roda o `build.rs` de novo a cada mudança em qualquer arquivo do pacote.

## Resumo

- `unsafe extern "C" { ... }` declara funções de C; o `unsafe` é a sua promessa de que as assinaturas batem. Itens são `unsafe` por padrão; `safe fn` só para funções sem pré-condição.
- Tipos: `c_int` e afins, `#[repr(C)]` para structs, `CString`/`CStr`/`c"..."` para strings, `extern "C" fn` para callbacks.
- `build.rs` + `cc` compilam o `.c` junto com o crate.
- Separe declarações cruas (`mod ffi`) de wrappers seguros; memória volta para quem a alocou.
- Assinatura errada é UB silencioso, e o Miri não roda C. Para bibliotecas grandes, gere as declarações com `bindgen`.

## Exercícios

Veja o exercício 4 em [`exercicios.md`](exercicios.md).

## Leitura complementar

- [The Rust Programming Language, 20.1: Using extern Functions to Call External Code](https://doc.rust-lang.org/stable/book/ch20-01-unsafe-rust.html#using-extern-functions-to-call-external-code)
- [The Rustonomicon: FFI](https://doc.rust-lang.org/nomicon/ffi.html)
- [Edition Guide: Unsafe `extern` blocks](https://doc.rust-lang.org/edition-guide/rust-2024/unsafe-extern.html)
- [Documentação de `std::ffi`](https://doc.rust-lang.org/std/ffi/index.html)
- [The Cargo Book: Build Scripts](https://doc.rust-lang.org/cargo/reference/build-scripts.html) e a [documentação do `cc`](https://docs.rs/cc)
- [The bindgen User Guide](https://rust-lang.github.io/rust-bindgen/)

---

[← Lição anterior](04-comportamento-indefinido-e-miri.md) · [Índice do módulo](README.md) · [Próxima lição →](06-expondo-rust-para-c.md)

# 03 — Slices em profundidade

**Objetivos.** Ao final, você vai saber:

- o que é um *slice* (`&[T]`, `&str`) na memória e por que ele é um *fat pointer*;
- por que funções devem receber `&str` e `&[T]`, e não `&String` e `&Vec<T>`;
- como um slice "amarra" o dono e transforma um bug de índice em erro de compilação;
- conseguir duas `&mut` para partes diferentes de um slice (`split_at_mut`, `get_disjoint_mut`).

## Um slice é ponteiro + tamanho

Um *slice* (fatia) é uma visão emprestada de uma sequência contígua: parte de um `Vec`, de um array ou dos bytes de uma `String`. Ele não tem dono e não aloca nada.

Na memória, uma referência para slice ocupa **dois** `usize`: o endereço do primeiro elemento e a quantidade de elementos. Por carregar o tamanho junto, ela se chama *fat pointer* (ponteiro "gordo"). Compare com `&String`, que é um ponteiro comum:

```rust
use std::mem::size_of;

fn main() {
    println!("String:   {} bytes", size_of::<String>());
    println!("&String:  {} bytes", size_of::<&String>());
    println!("&str:     {} bytes", size_of::<&str>());
    println!("&[i32]:   {} bytes", size_of::<&[i32]>());
    println!("&Vec<i32>: {} bytes", size_of::<&Vec<i32>>());
}
```

```text
String:   24 bytes
&String:  8 bytes
&str:     16 bytes
&[i32]:   16 bytes
&Vec<i32>: 8 bytes
```

(Numa máquina de 64 bits.) A `String` é o trio ptr/len/cap que você viu no M02. Uma `&String` aponta para esse trio. Uma `&str` aponta **direto para os bytes**, e pode começar no meio deles:

![Na stack: s, uma String dona com ptr, len 14 e cap 14, apontando para os bytes "borrow checker" no heap. r, uma &String de 8 bytes, aponta para o próprio s. word, uma &str de 16 bytes com len 6, aponta para o byte 0. tail, uma &str com len 7, aponta para o byte 7, no meio do buffer.](assets/slice-fat-pointer.svg)

Para quem vem de C++: `&str` é o equivalente de `std::string_view`, e `&[T]` é o de `std::span<T>`. O layout é o mesmo (ponteiro + tamanho). A diferença é que, em Rust, o compilador garante que a visão não sobrevive ao dado. Em C++, um `string_view` para uma `std::string` temporária compila e fica pendurado (a lição 04 mostra o caso equivalente em Rust).

## Ranges

Você cria um slice com um intervalo dentro de `[]`:

```rust
fn main() {
    let s = String::from("borrow checker");
    let a = &s[0..6];
    let b = &s[7..];
    let c = &s[..];
    println!("[{a}] [{b}] [{c}]");

    let v = [10, 20, 30, 40, 50];
    let meio: &[i32] = &v[1..4];
    println!("{meio:?} tem {} elementos", meio.len());
}
```

```text
[borrow] [checker] [borrow checker]
[20, 30, 40] tem 3 elementos
```

`a..b` inclui `a` e exclui `b`. Omitir o início significa 0, e omitir o fim significa "até o final". Índice fora do tamanho dá *panic*.

> [!WARNING]
> Em `&str`, os índices são em **bytes**, e o corte precisa cair numa fronteira de caractere. `&"olá"[0..3]` dá *panic*, porque o byte 3 está no meio do `á`. Esse é o assunto de UTF-8 do [Módulo 02](../02-ownership/README.md). Para cortar por caractere, use `char_indices()` ou métodos como `find` e `split`, que já devolvem posições válidas.

## Receba `&str` e `&[T]`, não `&String` e `&Vec<T>`

Uma função que só lê texto deve pedir `&str`. Assim ela aceita qualquer origem:

```rust
fn first_word(s: &str) -> &str {
    match s.find(' ') {
        Some(i) => &s[..i],
        None => s,
    }
}

fn main() {
    let dono = String::from("olá mundo");
    println!("{}", first_word(&dono));            // &String vira &str
    println!("{}", first_word(&dono[5..]));       // um pedaço da String
    println!("{}", first_word("borrow checker")); // literal
}
```

```text
olá
mundo
borrow
```

Passar `&dono` (uma `&String`) onde se espera `&str` funciona por *deref coercion*: o compilador converte `&String` em `&str` sozinho. O mesmo vale para `&Vec<T>` → `&[T]` e para arrays. O mecanismo (a trait `Deref`) fica para os Módulos 06 e 10. Por enquanto, guarde a regra: **no parâmetro, peça o slice**.

```rust
fn sum(xs: &[i32]) -> i32 {
    xs.iter().sum()
}
// sum(&vec), sum(&array) e sum(&array[1..3]) funcionam
```

O clippy avisa quando você esquece:

```text
warning: writing `&String` instead of `&str` involves a new object where a slice will do
 --> src/main.rs:1:20
  |
1 | fn count_spaces(s: &String) -> usize {
  |                    ^^^^^^^
  |
  = help: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.99.0/index.html#ptr_arg
  = note: `#[warn(clippy::ptr_arg)]` on by default
help: change this to
  |
1 - fn count_spaces(s: &String) -> usize {
1 + fn count_spaces(s: &str) -> usize {
  |
```

O nome *new object* no aviso se refere ao chamador: com `&String`, quem tem só um literal ou um pedaço de texto precisaria criar uma `String` nova só para chamar a função.

## O slice amarra o dono

O Rust Book motiva slices com um bug clássico: uma função `first_word` que devolve o **índice** do fim da palavra (`usize`). O índice não tem ligação nenhuma com a string. Se a string for apagada, o índice continua lá, apontando para nada que faça sentido, e o erro só aparece em runtime (ou nunca).

Com um slice, a ligação vira parte do tipo:

```rust
// ❌ NÃO COMPILA
fn first_word(s: &str) -> &str {
    match s.find(' ') {
        Some(i) => &s[..i],
        None => s,
    }
}

fn main() {
    let mut s = String::from("olá mundo");
    let word = first_word(&s);
    s.clear();
    println!("primeira palavra: {word}");
}
```

```text
error[E0502]: cannot borrow `s` as mutable because it is also borrowed as immutable
  --> src/main.rs:11:5
   |
10 |     let word = first_word(&s);
   |                           -- immutable borrow occurs here
11 |     s.clear();
   |     ^^^^^^^^^ mutable borrow occurs here
12 |     println!("primeira palavra: {word}");
   |                                  ---- immutable borrow later used here
```

Repare no rótulo da linha 10: o empréstimo nasceu em `&s`, **na chamada**, e continua vivo em `word`. O compilador sabe que o retorno de `first_word` empresta do argumento só olhando a assinatura (como ele sabe disso é o assunto das lições 04 e 05). `s.clear()` precisa de W em `s`, e `word` ainda está viva. É o E0502 de sempre.

## `&mut [T]`: alterar sem mudar o tamanho

Um slice mutável deixa alterar os elementos:

```rust
fn double_all(xs: &mut [i32]) {
    for x in xs.iter_mut() {
        *x *= 2;
    }
}

fn main() {
    let mut v = vec![1, 2, 3, 4];
    double_all(&mut v[1..3]);
    println!("{v:?}");
    let mut arr = [5, 6];
    double_all(&mut arr);
    println!("{arr:?}");
}
```

```text
[1, 4, 6, 4]
[10, 12]
```

Mas um slice é uma janela de tamanho fixo. Ele não tem `push`, porque crescer exigiria realocar, e o slice não é dono do buffer:

```text
error[E0599]: no method named `push` found for mutable reference `&mut [i32]` in the current scope
 --> src/main.rs:2:8
  |
2 |     xs.push(1);
  |        ^^^^ method not found in `&mut [i32]`
```

Se a função precisa crescer o vetor, ela recebe `&mut Vec<T>`. Essa é a exceção à regra "peça o slice": peça o tipo dono quando precisar das operações dele.

## Duas `&mut` em partes diferentes

Este código é seguro (os elementos 1 e 2 são posições diferentes), mas não compila:

```rust
// ❌ NÃO COMPILA
fn main() {
    let mut a = [1, 2, 3, 4];
    let x = &mut a[1];
    let y = &a[2];
    *x += *y;
    println!("{a:?}");
}
```

```text
error[E0502]: cannot borrow `a[_]` as immutable because it is also borrowed as mutable
 --> src/main.rs:4:13
  |
3 |     let x = &mut a[1];
  |             --------- mutable borrow occurs here
4 |     let y = &a[2];
  |             ^^^^^ immutable borrow occurs here
5 |     *x += *y;
  |     -------- mutable borrow later used here
  |
  = help: use `.split_at_mut(position)` to obtain two mutable non-overlapping sub-slices
```

Note o `a[_]` na mensagem: o borrow checker não compara os valores dos índices (em geral eles só são conhecidos em runtime). Para ele, `a[1]` e `a[2]` são "algum elemento de `a`", e pode ser o mesmo. É uma limitação do checker, e não um programa inseguro. A lição 07 volta a essa diferença.

A biblioteca padrão tem APIs feitas para isso. Elas verificam a separação (em compilação ou em runtime) e devolvem as duas referências:

```rust
fn main() {
    let mut a = [1, 2, 3, 4];
    let (left, right) = a.split_at_mut(2);
    left[1] += right[0];
    println!("{a:?}");

    let mut b = [1, 2, 3, 4];
    if let Ok([x, y]) = b.get_disjoint_mut([0, 3]) {
        std::mem::swap(x, y);
    }
    println!("{b:?}");
    println!("{:?}", b.get_disjoint_mut([1, 1]).is_err());
}
```

```text
[1, 5, 3, 4]
[4, 2, 3, 1]
true
```

- `split_at_mut(mid)` divide o slice em `[..mid]` e `[mid..]`, duas `&mut [T]` que não se sobrepõem.
- `get_disjoint_mut([i, j])` (estável desde a 1.86) devolve `Ok([&mut a[i], &mut a[j]])`, ou `Err` se algum índice for inválido ou se houver índices repetidos. Por isso `[1, 1]` deu erro: seriam duas `&mut` para o mesmo elemento.

Por dentro, essas funções usam `unsafe` (Módulo 15). É o padrão em Rust: a operação que o checker não consegue provar fica isolada numa função pequena e bem testada, com uma interface segura.

> [!NOTE]
> *Slice patterns*, como `[first, .., last]` e `[head, rest @ ..]`, aparecem no Módulo 04, junto com o resto de *pattern matching*.

## Armadilhas comuns

**`&String`/`&Vec<T>` no parâmetro.** Funciona, mas obriga quem chama a ter exatamente aquele tipo. Peça `&str`/`&[T]`, e o clippy (`ptr_arg`) vai parar de reclamar.

**Cortar `&str` no meio de um caractere.** Índices de `&str` são bytes. Use posições devolvidas por `find`, `char_indices` e afins.

**Esperar `push` num `&mut [T]`.** O slice não é dono do buffer e não pode crescer. Receba `&mut Vec<T>`.

**Achar que `a[1]` e `a[2]` são independentes para o checker.** Para ele, são `a[_]`. Use `split_at_mut` ou `get_disjoint_mut`.

## Resumo

- `&[T]` e `&str` são *fat pointers*: endereço + tamanho (16 bytes em 64 bits). `&String` e `&Vec<T>` são ponteiros comuns (8 bytes) para o struct dono.
- `&str` ≈ `std::string_view` e `&[T]` ≈ `std::span<T>`, só que com validade verificada.
- No parâmetro, peça `&str`/`&[T]`. A *deref coercion* converte `&String` e `&Vec<T>` sozinha.
- Um slice devolvido mantém o dono emprestado: mutar o dono enquanto o slice vive é E0502.
- `&mut [T]` altera elementos, mas não o tamanho. Para duas `&mut` disjuntas: `split_at_mut` e `get_disjoint_mut`.

## Exercícios

Veja o exercício 3 em [`exercicios.md`](exercicios.md).

## Leitura complementar

- [The Rust Programming Language, 4.3 The Slice Type](https://doc.rust-lang.org/stable/book/ch04-03-slices.html)
- [std: o tipo `slice`](https://doc.rust-lang.org/std/primitive.slice.html), em especial [`split_at_mut`](https://doc.rust-lang.org/std/primitive.slice.html#method.split_at_mut) e [`get_disjoint_mut`](https://doc.rust-lang.org/std/primitive.slice.html#method.get_disjoint_mut)
- [Clippy: `ptr_arg`](https://rust-lang.github.io/rust-clippy/master/index.html#ptr_arg)
- Código desta lição: [`codigo/slices`](codigo/slices/)

---

[← Lição anterior](02-borrow-checker-e-nll.md) · [Índice do módulo](README.md) · [Próxima lição →](04-lifetimes-em-funcoes.md)

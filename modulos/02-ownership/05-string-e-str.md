# 05 — `String` e `&str`

**Objetivos.** Ao final, você vai saber:

- a diferença entre `String` (dona do texto) e `&str` (vista emprestada de um texto);
- converter de um para o outro;
- escolher o tipo certo para parâmetros, campos e retornos;
- concatenar textos sem brigar com o compilador.

## Dois tipos para texto

Quem chega ao Rust estranha ter dois tipos de texto. Eles existem porque respondem à pergunta deste módulo de jeitos diferentes:

- **`String`** é **dona** do texto. Guarda `ptr`, `len` e `cap` na stack e os bytes no heap, pode crescer, e libera o heap quando sai de escopo. É a que você viu nas lições anteriores.
- **`&str`** (lê-se "string slice", ou fatia de string) é uma **vista emprestada**: um ponteiro e um tamanho (16 bytes) que apontam para bytes UTF-8 que estão **em outro lugar**. Ela não é dona de nada e não libera nada.

```rust
let literal: &str = "oi";                   // bytes dentro do executável
let s: String = String::from("oi, mundo");  // bytes no heap, s é a dona
let fatia: &str = &s[4..9];                 // vista para um pedaço de s
println!("{literal} | {s} | {fatia}");      // oi | oi, mundo | mundo
```

![Na stack: literal (&str, roxo) com ptr e len 2 aponta para os bytes "oi" nos dados do executável, que são só leitura. s (String, verde, dona) com ptr, len 9 e cap 9 aponta para "oi, mundo" no heap. fatia (&str, roxo) = &s[4..9], com ptr e len 5, aponta para o meio desse mesmo buffer e enxerga só "mundo". Só s libera memória.](assets/str-vs-string.svg)

Três coisas para notar no diagrama:

- Um **literal** (`"oi"`) é gravado no próprio executável, e o programa recebe uma `&str` para ele. O tipo completo é `&'static str`: o `'static` diz que a vista vale pelo programa inteiro, porque o executável nunca é "liberado". *Lifetimes* como `'static` são o Módulo 03.
- Uma `&str` pode apontar para o **meio** de uma `String`: `&s[4..9]` não copia nada, só calcula um ponteiro e um tamanho.
- Quem libera memória é só a `String`. As `&str` somem sem fazer nada.

O tipo `str` sozinho (sem `&`) também existe: é "uma sequência de bytes UTF-8 de tamanho qualquer". Como o tamanho não é conhecido na compilação, você quase nunca o vê fora de uma referência. Na prática, pense em `&str` como um tipo só.

> [!NOTE]
> A `&` em `&str` é a mesma das referências que você viu de passagem na [lição 02](02-regras-de-ownership-e-move.md#as-três-saídas-para-o-e0382): um empréstimo. Isso tem uma consequência que o Módulo 03 detalha: uma `&str` que aponta para dentro de uma `String` não pode viver mais que ela. O compilador garante isso.

## Convertendo

De `&str` para `String` (aloca e copia os bytes para o heap):

```rust
let a = String::from("x");
let b = "x".to_string();
let c = "x".to_owned();
let d: String = "x".into();
```

As quatro fazem a mesma coisa. O Rust Book diz que a escolha é questão de estilo. `to_string()` funciona com qualquer coisa que se imprime com `{}`, e `to_owned()` deixa explícita a intenção de "quero uma cópia que seja minha".

De `String` para `&str` (não aloca nada):

```rust
let s = String::from("oi, mundo");
let a: &str = &s;          // o mais comum
let b: &str = s.as_str();  // explícito
let c: &str = &s[..];      // fatia com o texto todo
```

O compilador não converte de `&str` para `String` sozinho, porque isso custaria uma alocação:

```rust
// ❌ NÃO COMPILA
fn main() {
    let s: String = "oi";
    println!("{s}");
}
```

```text
error[E0308]: mismatched types
 --> src/main.rs:2:21
  |
2 |     let s: String = "oi";
  |            ------   ^^^^ expected `String`, found `&str`
  |            |
  |            expected due to this
  |
help: try using a conversion method
  |
2 |     let s: String = "oi".to_string();
  |                         ++++++++++++
```

## Qual usar onde

A regra prática, que vale para quase todo código:

| Onde | Use | Por quê |
|---|---|---|
| parâmetro que só **lê** o texto | `&str` | aceita literal, `String` e fatia, sem tirar a posse de ninguém |
| campo de struct | `String` | o struct precisa ser dono do que guarda (há exceções, no Módulo 03) |
| retorno de texto **novo** | `String` | quem chamou vai ser o dono do resultado |

Uma função que recebe `&str` aceita todas as formas de texto:

```rust
fn contar_vogais(s: &str) -> usize {
    s.chars().filter(|c| "aeiouAEIOU".contains(*c)).count()
}

fn main() {
    let nome = String::from("Ferris");
    println!("{}", contar_vogais("banana"));     // literal: 3
    println!("{}", contar_vogais(&nome));        // &String: 2
    println!("{}", contar_vogais(&nome[0..3]));  // fatia "Fer": 1
    println!("{nome}");                          // nome continua válida
}
```

`&nome` tem tipo `&String`, e mesmo assim é aceito onde se espera `&str`. O compilador faz essa conversão sozinho porque ela é de graça (é só pegar o `ptr` e o `len` da `String`). O nome do mecanismo é *deref coercion*, explicado mais adiante (Módulos 06 e 10).

Se a mesma função pedisse `String`, os literais deixariam de funcionar, e quem tem uma `String` teria que entregá-la (ou cloná-la):

```rust
// ❌ NÃO COMPILA
fn tamanho(s: String) -> usize {
    s.len()
}

fn main() {
    println!("{}", tamanho("olá"));
}
```

```text
error[E0308]: mismatched types
 --> src/main.rs:6:28
  |
6 |     println!("{}", tamanho("olá"));
  |                    ------- ^^^^^ expected `String`, found `&str`
  |                    |
  |                    arguments to this function are incorrect
  |
note: function defined here
 --> src/main.rs:1:4
  |
1 | fn tamanho(s: String) -> usize {
  |    ^^^^^^^ ---------
help: try using a conversion method
  |
6 |     println!("{}", tamanho("olá".to_string()));
  |                                 ++++++++++++
```

O `help:` sugere converter na chamada, mas a correção certa é na assinatura: `fn tamanho(s: &str)`.

Quando a função **cria** um texto, ela devolve `String`, e o parâmetro continua `&str`:

```rust
fn saudacao(nome: &str) -> String {
    format!("Olá, {nome}!")
}
```

Código completo em [`codigo/texto`](codigo/texto/).

### Comparando com C++

O par `std::string` / `std::string_view` (C++17) é quase o mesmo desenho: `string` é dona, `string_view` é ponteiro + tamanho. A diferença é quem garante que a vista não sobrevive ao dono:

```cpp
std::string_view saudacao(const std::string& nome) {
    std::string s = "Olá, " + nome;
    return s;   // devolve uma vista para um buffer que morre aqui
}
```

O `clang++ -Wall` avisa neste caso direto (`-Wreturn-stack-address`), mas compila, e em casos menos óbvios nem avisa. Em Rust, o equivalente é um erro de compilação:

```rust
// ❌ NÃO COMPILA
fn gritar(s: &str) -> &str {
    let alto = s.to_uppercase();
    &alto
}
```

```text
error[E0515]: cannot return reference to local variable `alto`
 --> src/main.rs:3:5
  |
3 |     &alto
  |     ^^^^^ returns a reference to data owned by the current function
```

`alto` é uma `String` local, liberada no fim da função. Uma `&str` para ela ficaria pendente. A correção é devolver a dona: `fn gritar(s: &str) -> String` e `alto` no fim, sem `&`. Por que o compilador sabe disso, e como ele raciocina sobre quanto tempo cada referência vale, é o Módulo 03.

### Comparando com Python

O `str` do Python é imutável e compartilhado: `b = a` cria outra referência para o mesmo objeto, e você nunca precisa pensar em quem é o dono. O Rust obriga a escolher: `String` quando você precisa ser dono (guardar, alterar, devolver), `&str` quando só vai olhar. Em troca, uma função que recebe `&str` deixa claro, na assinatura, que não vai guardar nem alterar o texto.

## Concatenando

Juntar dois textos parece trivial, mas o operador `+` tem uma assinatura que conta uma história. Para `String`, ele é:

```rust
fn add(self, s: &str) -> String
```

O lado esquerdo é recebido **por valor** (`self`, um move), e o lado direito é emprestado (`&str`). O `+` reaproveita o buffer da esquerda, acrescenta os bytes da direita e devolve a mesma `String`, possivelmente realocada. É eficiente, mas gera três erros clássicos:

```rust
// ❌ NÃO COMPILA
fn main() {
    let a = String::from("Olá, ");
    let b = String::from("mundo");
    let c = a + b;
    println!("{c}");
}
```

```text
error[E0308]: mismatched types
 --> src/main.rs:4:17
  |
4 |     let c = a + b;
  |                 ^ expected `&str`, found `String`
  |
help: consider borrowing here
  |
4 |     let c = a + &b;
  |                 +
```

O lado direito precisa ser `&str`. Com `a + &b` compila, mas aí `a` foi movida:

```rust
// ❌ NÃO COMPILA
fn main() {
    let a = String::from("Olá, ");
    let b = String::from("mundo");
    let c = a + &b;
    println!("{a} {b} {c}");
}
```

```text
error[E0382]: borrow of moved value: `a`
 --> src/main.rs:5:16
  |
2 |     let a = String::from("Olá, ");
  |         - move occurs because `a` has type `String`, which does not implement the `Copy` trait
3 |     let b = String::from("mundo");
4 |     let c = a + &b;
  |             - value moved here
5 |     println!("{a} {b} {c}");
  |                ^ value borrowed here after move
...
```

E dois literais não se somam, porque nenhum dos dois é dono de um buffer que possa crescer:

```rust
// ❌ NÃO COMPILA
fn main() {
    let c = "Olá, " + "mundo";
    println!("{c}");
}
```

```text
error[E0369]: cannot add `&str` to `&str`
 --> src/main.rs:2:21
  |
2 |     let c = "Olá, " + "mundo";
  |             ------- ^ ------- &str
  |             |       |
  |             |       `+` cannot be used to concatenate two `&str` strings
  |             &str
  |
  = note: string concatenation requires an owned `String` on the left
help: create an owned `String` from a string reference
  |
2 |     let c = "Olá, ".to_owned() + "mundo";
  |                    +++++++++++
```

Na maioria dos casos, prefira `format!`, que só empresta os argumentos e devolve uma `String` nova:

```rust
let a = String::from("Olá, ");
let b = String::from("mundo");
let c = format!("{a}{b}!");                 // ✅ não move nada
println!("{c} | a = {a:?} | b = {b:?}");    // Olá, mundo! | a = "Olá, " | b = "mundo"
```

Para acrescentar a uma `String` que você já tem, altere no lugar:

```rust
let mut t = String::from("ab");
t.push_str("cd");   // acrescenta uma &str
t.push('!');        // acrescenta um char
t += "?";           // o mesmo que push_str
println!("{t}");    // abcd!?
```

## Armadilhas comuns

### `&String` como parâmetro

Compila, mas restringe à toa: a função deixa de aceitar literais e fatias sem conversão. Numa função privada, o clippy avisa:

```rust
fn tamanho(s: &String) -> usize {
    s.len()
}
```

```text
warning: writing `&String` instead of `&str` involves a new object where a slice will do
 --> src/main.rs:1:15
  |
1 | fn tamanho(s: &String) -> usize {
  |               ^^^^^^^
  |
  = help: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.99.0/index.html#ptr_arg
  = note: `#[warn(clippy::ptr_arg)]` on by default
help: change this to
  |
1 - fn tamanho(s: &String) -> usize {
1 + fn tamanho(s: &str) -> usize {
  |
```

(Em funções `pub` de uma biblioteca, o clippy fica quieto por padrão, para não sugerir uma mudança de API. A regra continua valendo.) O mesmo vale para `&Vec<T>` como parâmetro: prefira a fatia `&[T]`.

### Esperar conversão automática de `&str` para `String`

E0308 em `let s: String = "oi";` e em chamadas a funções que pedem `String`. Use `.to_string()`, ou mude a assinatura para `&str`, se a função só lê.

### Usar a `String` da esquerda depois de `+`

Ela foi movida para dentro do resultado. Use `format!` se ainda precisar dela.

### Devolver `&str` que aponta para uma `String` local (E0515)

Se a função cria o texto, ela devolve `String`.

## Resumo

- `String` é dona: heap, cresce, libera no `drop`. `&str` é uma vista emprestada (ponteiro + tamanho) para bytes que estão em outro lugar.
- Literais são `&'static str` e vivem dentro do executável.
- `String::from`, `to_string`, `to_owned` e `into` criam `String` a partir de `&str`. `&s` e `as_str()` dão a `&str` de uma `String`, sem copiar.
- Parâmetro que só lê: `&str`. Campo e retorno de texto novo: `String`.
- `+` consome a `String` da esquerda e pede `&str` à direita. `format!` não move nada e costuma ser a melhor escolha.
- Uma `&str` não pode sobreviver ao dono dos bytes (E0515). O Módulo 03 explica como o compilador verifica isso.

## Exercícios

Veja o exercício 4 em [`exercicios.md`](exercicios.md).

## Leitura complementar

- [The Rust Programming Language, 8.2 Storing UTF-8 Encoded Text with Strings](https://doc.rust-lang.org/stable/book/ch08-02-strings.html) (seções "Creating a New String" e "Updating a String")
- [The Rust Programming Language, 4.3 The Slice Type](https://doc.rust-lang.org/stable/book/ch04-03-slices.html) (seção "String Slices")
- Documentação de [`String`](https://doc.rust-lang.org/std/string/struct.String.html) e [`str`](https://doc.rust-lang.org/std/primitive.str.html)
- [100 Exercises to Learn Rust: References in memory](https://rust-exercises.com/100-exercises/03_ticket_v1/10_references_in_memory.html)

---

[← Lição anterior](04-drop-e-raii.md) · [Índice do módulo](README.md) · [Próxima lição →](06-strings-utf8.md)

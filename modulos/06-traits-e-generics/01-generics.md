# 01 — Generics

**Objetivos.** Ao final, você vai saber:

- escrever funções, structs, enums e blocos `impl` com parâmetros de tipo (`<T>`);
- quando o compilador infere o tipo sozinho e quando você precisa do *turbofish* (`::<T>`);
- o que é monomorfização, e o que ela custa (e o que não custa);
- a diferença essencial entre generics do Rust e templates do C++.

## O problema: a mesma função, tipo por tipo

Você precisa do maior elemento de uma lista de inteiros e, depois, de uma lista de caracteres:

```rust
fn maior_i32(lista: &[i32]) -> &i32 {
    let mut maior = &lista[0];
    for item in lista {
        if item > maior {
            maior = item;
        }
    }
    maior
}

fn maior_char(lista: &[char]) -> &char {
    let mut maior = &lista[0];
    for item in lista {
        if item > maior {
            maior = item;
        }
    }
    maior
}
```

Os dois corpos são idênticos; só o tipo muda. A solução é a mesma de C++, Java ou TypeScript: um parâmetro de tipo. Você declara o parâmetro entre `<>` depois do nome da função e usa esse nome no lugar do tipo concreto:

```rust
// ❌ NÃO COMPILA
fn largest<T>(list: &[T]) -> &T {
    let mut largest = &list[0];
    for item in list {
        if item > largest {
            largest = item;
        }
    }
    largest
}
```

```text
error[E0369]: binary operation `>` cannot be applied to type `&T`
 --> src/main.rs:4:17
  |
4 |         if item > largest {
  |            ---- ^ ------- &T
  |            |
  |            &T
  |
help: consider restricting type parameter `T` with trait `PartialOrd`
  |
1 | fn largest<T: std::cmp::PartialOrd>(list: &[T]) -> &T {
  |             ++++++++++++++++++++++
```

O compilador recusa porque `T` pode ser **qualquer** tipo, e nem todo tipo sabe se comparar com `>`. Ele não vai esperar alguém chamar `largest` com um tipo sem comparação para descobrir o problema: exige que a assinatura diga o que `T` sabe fazer. O `help:` mostra como: `T: PartialOrd` quer dizer "qualquer `T` que implemente a *trait* `PartialOrd`", a trait por trás de `<` e `>`. Isso se chama *trait bound* (restrição de trait), e é o assunto da [lição 03](03-trait-bounds-where-e-impl-trait.md). Com o bound, a função compila e serve para `i32`, `char`, `&str`, `f64`...

Uma função genérica que não faz nada com `T` além de mover os valores de um lado para o outro não precisa de bound nenhum:

```rust
fn escolher<T>(primeiro: bool, a: T, b: T) -> T {
    if primeiro { a } else { b }
}

println!("{} {}", escolher(true, 1, 2), escolher(false, "a", "b")); // 1 b
```

Mover um valor é algo que todo tipo sabe fazer. Somar, comparar, imprimir ou clonar já exige um bound. Você já viu uma função assim no Módulo 02: `std::mem::drop` tem a assinatura `pub fn drop<T>(_x: T)`. Ela recebe um valor de qualquer tipo e não faz nada com ele, e é justamente por isso que funciona: o valor é movido para dentro e destruído no fim da função.

## Structs e enums genéricos

O mesmo `<T>` vale para tipos. Um ponto cujas coordenadas podem ser inteiras ou reais:

```rust
struct Ponto<T> {
    x: T,
    y: T,
}

let inteiro = Ponto { x: 5, y: 10 };   // Ponto<i32>
let real = Ponto { x: 3.0, y: 4.0 };  // Ponto<f64>
```

Os dois campos usam o **mesmo** `T`, então precisam ter o mesmo tipo:

```rust
// ❌ NÃO COMPILA
let p = Ponto { x: 5, y: 4.0 };
```

```text
error[E0308]: mismatched types
 --> src/main.rs:7:30
  |
7 |     let p = Ponto { x: 5, y: 4.0 };
  |                              ^^^ expected integer, found floating-point number
```

Pelo primeiro campo, o compilador concluiu que `T` é um inteiro; o `4.0` contradiz isso. Se você quer tipos independentes, declare dois parâmetros: `struct Par<A, B> { a: A, b: B }`. Use quantos precisar, mas, se a lista começar a crescer muito, é sinal de que o tipo está fazendo coisas demais.

Enums genéricos você já usa desde o Módulo 04. `Option` e `Result` não têm nada de especial: são enums comuns da biblioteca padrão, declarados assim:

```rust
enum Option<T> {
    Some(T),
    None,
}

enum Result<T, E> {
    Ok(T),
    Err(E),
}
```

`Option<i32>` e `Option<String>` são tipos diferentes, gerados a partir do mesmo molde.

## `impl` genérico e `impl` para um tipo só

Para escrever métodos num tipo genérico, o `impl` também declara o parâmetro:

```rust
impl<T> Ponto<T> {
    fn x(&self) -> &T {
        &self.x
    }
}
```

O primeiro `<T>` (depois de `impl`) **declara** o parâmetro; o segundo (em `Ponto<T>`) **usa**. Leia como "para todo tipo `T`, eis os métodos de `Ponto<T>`". Sem o `impl<T>`, o compilador acharia que `T` é o nome de um tipo concreto que você esqueceu de definir.

Você também pode escrever métodos que só existem para um tipo concreto:

```rust
impl Ponto<f64> {
    fn distancia_da_origem(&self) -> f64 {
        (self.x * self.x + self.y * self.y).sqrt()
    }
}
```

Aqui não há `<T>` depois de `impl`: o bloco vale só para `Ponto<f64>`. Chamar o método num `Ponto<i32>` não compila, e o compilador diz onde ele existe:

```text
error[E0599]: no method named `distancia_da_origem` found for struct `Ponto<{integer}>` in the current scope
...
   = note: the method was found for `Ponto<f64>`
```

O meio-termo, "métodos só para os `T` que sabem fazer tal coisa", é o `impl` condicional da lição 03.

Um método pode ter parâmetros de tipo próprios, além dos do `impl`. O exemplo clássico do Rust Book junta dois pares de tipos diferentes:

```rust
struct Par<A, B> {
    a: A,
    b: B,
}

impl<A, B> Par<A, B> {
    fn misturar<C, D>(self, outro: Par<C, D>) -> Par<A, D> {
        Par { a: self.a, b: outro.b }
    }
}

let p1 = Par { a: 5, b: 10.4 };
let p2 = Par { a: "oi", b: 'c' };
println!("{:?}", p1.misturar(p2)); // Par { a: 5, b: 'c' }
```

`A` e `B` pertencem ao tipo (vêm do `impl`); `C` e `D` pertencem só ao método.

> [!NOTE]
> No Módulo 03 você viu `<'a>`. Lifetimes também são parâmetros genéricos, e os dois tipos de parâmetro convivem na mesma lista, com os lifetimes primeiro: `fn primeiro<'a, T>(v: &'a [T]) -> &'a T`. Combinações mais avançadas, como `T: 'a`, ficam para o Módulo 13.

## Inferência e turbofish

Na maioria das vezes você não escreve o tipo: o compilador deduz pelo uso. Em `let mut w = Vec::new(); w.push(1.5);`, ele vê o `push` e conclui que `w` é `Vec<f64>`.

Quando não há pistas suficientes, ele pede ajuda:

```rust
// ❌ NÃO COMPILA
let n = "42".parse().unwrap();
```

```text
error[E0284]: type annotations needed
    --> src/main.rs:2:9
     |
   2 |     let n = "42".parse().unwrap();
     |         ^        ----- type must be known at this point
...
help: consider giving `n` an explicit type
     |
   2 |     let n: /* Type */ = "42".parse().unwrap();
     |          ++++++++++++
```

`parse` é genérica no tipo de retorno (`pub fn parse<F: FromStr>(&self) -> Result<F, F::Err>`), e nada no código diz qual `F` você quer. Há duas saídas: anotar a variável (`let n: i32 = ...`) ou dizer o tipo direto na chamada, com a sintaxe `::<T>`, apelidada de *turbofish* pelo formato:

```rust
let n = "42".parse::<i32>().unwrap();
let v = Vec::<u8>::with_capacity(4);
```

O `::` antes do `<` é obrigatório nas expressões. Sem ele, `parse<i32>()` seria lido como uma comparação `parse < i32 > ()`. Em tipos (`let v: Vec<u8>`) não há ambiguidade, e o `<` vai direto. O turbofish vai aparecer bastante no Módulo 07, em `collect::<Vec<_>>()`, em que o `_` pede ao compilador para completar o resto.

## Monomorfização: uma cópia por tipo

Quanto custa um generic em tempo de execução? Nada. O compilador faz o mesmo que um template de C++: para cada combinação de tipos concretos que o programa usa, ele gera uma cópia da função com os tipos já substituídos. Isso se chama *monomorfização* (de "mono", uma forma só). Depois disso, cada cópia é compilada e otimizada como se você a tivesse escrito à mão.

Dá para ver as cópias no executável. O projeto [`codigo/monomorfizacao`](codigo/monomorfizacao/) tem uma função genérica e uma que usa *trait object* (`&dyn Display`, assunto da [lição 06](06-trait-objects-e-dyn.md)), cada uma chamada com três tipos:

```rust
use std::fmt::Display;

fn mostra<T: Display>(x: T) -> String {
    format!("<{x}>")
}

fn mostra_dyn(x: &dyn Display) -> String {
    format!("<{x}>")
}

fn main() {
    println!("{} {} {}", mostra(1), mostra("a"), mostra(2.5));
    println!("{} {} {}", mostra_dyn(&1), mostra_dyn(&"a"), mostra_dyn(&2.5));
}
```

A ferramenta `nm` lista os símbolos do binário (no macOS e no Linux):

```bash
cargo build
nm -C target/debug/monomorfizacao | grep mostra
```

```text
00000001000017f4 t monomorfizacao::mostra::<&str>
0000000100001884 t monomorfizacao::mostra::<f64>
000000010000190c t monomorfizacao::mostra::<i32>
0000000100000b5c t monomorfizacao::mostra_dyn
```

Três cópias de `mostra`, uma por tipo, e uma só de `mostra_dyn`. A versão `dyn` funciona de outro jeito: em vez de gerar código por tipo, ela recebe junto com o valor uma tabela de funções e decide em tempo de execução qual `fmt` chamar.

![Esquerda: o código-fonte com uma função genérica mostra<T> vira, depois da compilação, três funções separadas, mostra::<i32>, mostra::<&str> e mostra::<f64>, cada uma chamada diretamente. Direita: a função mostra_dyn existe uma vez só e recebe um fat pointer; na hora da chamada, ela consulta a vtable do tipo para achar o fmt certo.](assets/monomorfizacao-x-dyn.svg)

O custo da monomorfização aparece em outros lugares:

- **tamanho do binário:** cada tipo usado gera mais código;
- **tempo de compilação:** cada cópia é compilada e otimizada separadamente.

Em troca, cada cópia pode ser otimizada para o seu tipo (o compilador pode fazer *inlining*, ou seja, colar o corpo da função no lugar da chamada). Na prática, isso quase nunca é problema, e generics são a ferramenta padrão. A [lição 07](07-estatico-x-dinamico-e-composicao.md) volta à escolha entre as duas formas.

## Comparando com C++ e Python

Os templates do C++ também monomorfizam. A diferença está em **quando** o corpo é verificado:

```cpp
template <class T>
T soma(T a, T b) {
    return a + b;
}

struct Ponto { int x, y; };

int main() {
    soma(1, 2);              // ok
    // soma(Ponto{}, Ponto{}); // só aqui o erro aparece
}
```

O template acima compila enquanto ninguém chamar `soma` com um tipo sem `operator+`. O erro só aparece **na instanciação**, dentro do corpo do template, e é daí que vêm as mensagens de 200 linhas. Em Rust, o corpo é verificado **na definição**, contra os bounds declarados: se a assinatura não diz que `T` sabe somar, `a + b` não compila, mesmo que ninguém chame a função. Quem usa a função só precisa olhar a assinatura.

O paralelo mais próximo em C++ moderno são os *concepts* (C++20): `template <std::totally_ordered T>` é parecido com `T: PartialOrd`. A diferença é que um concept é estrutural (basta o tipo ter as operações), enquanto uma trait é nominal (o tipo precisa declarar `impl Trait`). A lição 02 volta a isso.

Em Python, a função genérica é a função comum: qualquer objeto entra, e o erro aparece quando a operação falha.

```python
from typing import TypeVar

T = TypeVar("T")

def escolher(primeiro: bool, a: T, b: T) -> T:
    return a if primeiro else b
```

O `TypeVar` é o `<T>` do Python, e `TypeVar("T", bound=Comparavel)` lembra um trait bound. A diferença é quem confere: em Python, o mypy (se você rodar); em Rust, o compilador, sempre. E em tempo de execução não há cópias: é uma função só, que trata todos os objetos do mesmo jeito.

## Armadilhas comuns

**Esquecer o `<T>` depois de `impl`.** O compilador procura um tipo chamado `T` e não acha:

```rust
// ❌ NÃO COMPILA
impl Ponto<T> {
    fn x(&self) -> &T { &self.x }
}
```

```text
error[E0425]: cannot find type `T` in this scope
 --> src/main.rs:2:12
  |
2 | impl Ponto<T> {
  |            ^ not found in this scope
  |
help: you might be missing a type parameter
  |
2 | impl<T> Ponto<T> {
  |     +++
```

**Achar que dá para especializar.** Não existe, em Rust estável, uma forma de escrever uma versão genérica de um método e uma versão especial para `f64` que "ganha" da genérica. Os blocos `impl Ponto<f64>` acrescentam **métodos novos**, mas não substituem os de `impl<T>`. Se você declarar o mesmo método nos dois, o compilador acusa duplicidade:

```text
error[E0592]: duplicate definitions with name `nome`
 --> src/main.rs:3:5
  |
3 |     fn nome(&self) -> &'static str { "generico" }
  |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ duplicate definitions for `nome`
...
6 |     fn nome(&self) -> &'static str { "f64" }
  |     ------------------------------ other definition for `nome`
```

**Esperar que a função funcione "porque o tipo que eu uso sabe fazer isso".** Em C++, o que vale é o tipo da chamada. Em Rust, o que vale é o bound. Se você usa um método de `T` no corpo, o bound tem que estar na assinatura.

## Resumo

- `<T>` declara um parâmetro de tipo em funções, structs, enums e blocos `impl`. `Option<T>` e `Result<T, E>` são enums genéricos comuns.
- O corpo genérico só pode fazer com `T` o que os bounds permitem. Sem bound, só dá para mover o valor.
- `impl<T> Ponto<T>` vale para todo `T`; `impl Ponto<f64>` acrescenta métodos só para um tipo.
- O tipo costuma ser inferido. Quando não for, anote a variável ou use o turbofish: `parse::<i32>()`.
- Monomorfização: uma cópia da função por tipo concreto. Custo zero em tempo de execução; o preço é binário maior e compilação mais lenta.
- Diferente dos templates do C++, o corpo é verificado na definição, contra os bounds.

## Exercícios

Veja o exercício 1 em [`exercicios.md`](exercicios.md).

## Leitura complementar

- [The Rust Programming Language, 10.1 Generic Data Types](https://doc.rust-lang.org/stable/book/ch10-01-syntax.html)
- [Rust by Example: Generics](https://doc.rust-lang.org/rust-by-example/generics.html)
- [Comprehensive Rust: Generics](https://google.github.io/comprehensive-rust/generics.html)
- [Effective Rust, Item 12: Understand the trade-offs between generics and trait objects](https://effective-rust.com/generics.html)
- [cppreference: Constraints and concepts](https://en.cppreference.com/w/cpp/language/constraints)

---

[Índice do módulo](README.md) · [Próxima lição →](02-definindo-e-implementando-traits.md)

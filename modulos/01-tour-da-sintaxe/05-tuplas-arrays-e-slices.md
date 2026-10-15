# 05 — Tuplas, arrays e slices

**Objetivos.** Ao final, você vai saber:

- agrupar valores de tipos diferentes numa tupla e desestruturá-la;
- usar arrays de tamanho fixo e entender por que o índice é `usize`;
- o que acontece com um índice fora dos limites;
- o que é um *slice* (`&[T]`), numa primeira visão.

## Tuplas

Uma tupla agrupa um número fixo de valores, que podem ter tipos diferentes:

```rust
let pessoa = ("Ana", 31, 1.68);           // tipo (&str, i32, f64)
println!("{} tem {} anos", pessoa.0, pessoa.1);

let (nome, idade, altura) = pessoa;       // desestruturação
println!("{nome}, {idade}, {altura}");
```

O acesso é por posição (`.0`, `.1`...), e a desestruturação funciona como no Python (`nome, idade, altura = pessoa`). O uso mais comum é devolver mais de um valor de uma função:

```rust
fn min_max(a: i32, b: i32) -> (i32, i32) {
    if a < b { (a, b) } else { (b, a) }
}

let (menor, maior) = min_max(9, 4); // 4 e 9
```

O `()` da [lição 03](03-expressoes-blocos-e-funcoes.md) é exatamente isto: a tupla vazia.

Quando os campos precisam de nome, use uma *struct* (Módulo 04). Uma tupla com mais de três ou quatro elementos já fica difícil de ler.

## Arrays: tamanho fixo, parte do tipo

```rust
let notas = [7.5, 8.0, 9.5];   // tipo [f64; 3]
let zeros = [0u8; 4];          // [0, 0, 0, 0]: valor e quantidade
println!("{}", notas.len());   // 3
```

O tipo de um array é `[T; N]`: o tamanho `N` faz parte do tipo, então `[f64; 3]` e `[f64; 4]` são tipos diferentes. Ele fica na *stack* (pilha), sem alocação, como um `std::array<double, 3>` do C++.

Um array não cresce. Para uma lista que muda de tamanho, use `Vec`, da [lição 07](07-kit-vec-e-hashmap.md). Na prática, `Vec` é muito mais comum, e o array aparece quando o tamanho é mesmo fixo (as coordenadas de um ponto, os dias da semana).

O `for` percorre os elementos:

```rust
let notas = [7.5, 8.0, 9.5];
let mut soma = 0.0;
for n in notas {
    soma += n;
}
println!("média: {}", soma / notas.len() as f64); // média: 8.333333333333334
```

Repare no `notas.len() as f64`: `len()` devolve `usize`, e não há conversão implícita para `f64` (veja a [lição 02](02-tipos-primitivos-e-conversoes.md)).

## Índices são `usize` e são verificados

O índice de um array (e de um `Vec`, e de um slice) tem que ser `usize`:

```rust
// ❌ NÃO COMPILA
fn main() {
    let v = [10, 20, 30];
    let i: i32 = 1;
    println!("{}", v[i]);
}
```

```text
error[E0277]: the type `[{integer}]` cannot be indexed by `i32`
   --> src/main.rs:4:22
    |
  4 |     println!("{}", v[i]);
    |                      ^ slice indices are of type `usize` or ranges of `usize`
    |
    = help: the trait `SliceIndex<[{integer}]>` is not implemented for `i32`
...
    = note: required for `[{integer}]` to implement `Index<i32>`
```

A correção é declarar o índice como `usize` desde o começo. Se ele vem de um cálculo com sinal, `i as usize` resolve, mas pense antes: um `-1 as usize` vira um número enorme.

Todo acesso por índice é verificado. Se o índice é uma constante fora dos limites, o build quebra:

```rust
// ❌ NÃO COMPILA
fn main() {
    let a = [1, 2, 3];
    println!("{}", a[5]);
}
```

```text
error: this operation will panic at runtime
 --> src/main.rs:3:20
  |
3 |     println!("{}", a[5]);
  |                    ^^^^ index out of bounds: the length is 3 but the index is 5
  |
  = note: `#[deny(unconditional_panic)]` on by default
```

Se o índice só é conhecido em tempo de execução, o programa para com *panic*:

```rust
// 💥 PANIC EM RUNTIME
fn main() {
    let a = [1, 2, 3];
    let i: usize = "5".parse().unwrap();
    println!("{}", a[i]);
}
```

```text
thread 'main' (190418) panicked at src/main.rs:4:20:
index out of bounds: the len is 3 but the index is 5
```

Em C++, `v[i]` fora dos limites é comportamento indefinido: pode ler lixo, corromper memória ou parecer funcionar. Só o `.at(i)` verifica. Em Rust, a verificação é o padrão, nos perfis debug **e** release. Quando o índice pode estar fora, use `.get(i)`, que devolve um `Option` em vez de dar *panic* (veja a [lição 07](07-kit-vec-e-hashmap.md)).

> [!NOTE]
> A verificação de limites tem um custo pequeno, e o compilador remove muitas delas quando consegue provar que o índice é válido. Percorrer com `for x in &v`, em vez de indexar, evita a verificação por completo.

## Slices: uma vista de um pedaço

Um *slice* (fatia) é uma vista **emprestada** de uma sequência contígua de elementos: parte de um array, de um `Vec` ou o todo. O tipo é `&[T]`:

```rust
let a = [10, 20, 30, 40, 50];
let meio: &[i32] = &a[1..4];   // [20, 30, 40]
let inicio = &a[..2];          // [10, 20]
let fim = &a[3..];             // [40, 50]
```

O `&` indica um empréstimo: o slice não copia os elementos, ele aponta para eles. Por enquanto, guarde só a receita: **o `&` dá acesso a um valor sem tomar posse dele**. O que isso significa (e as regras que o compilador aplica) é o assunto dos Módulos [02](../02-ownership/README.md) e [03](../03-borrowing-e-lifetimes/README.md).

O uso típico é como parâmetro: uma função que recebe `&[i32]` aceita um array inteiro, parte de um array ou um `Vec`, sem copiar nada:

```rust
fn soma_tudo(numeros: &[i32]) -> i32 {
    let mut total = 0;
    for n in numeros {
        total += n;
    }
    total
}

let a = [10, 20, 30, 40, 50];
println!("{}", soma_tudo(&a));        // 150
println!("{}", soma_tudo(&a[1..4]));  // 90
```

Não há índice negativo. O `a[-2:]` do Python vira `&a[a.len() - 2..]`.

## Armadilhas comuns

- **Índice `i32`** (E0277 `slice indices are of type usize`): declare o índice como `usize`.
- **Índice fora dos limites**: lint que barra o build (se for constante) ou *panic* em runtime. Quando o índice pode estar fora, use `.get(i)`.
- **Esperar que o array cresça.** Arrays têm tamanho fixo. Para crescer, use `Vec`.
- **Tupla grande demais.** Com mais de três ou quatro campos, uma *struct* com nomes fica mais clara.

## Resumo

- Tuplas agrupam valores de tipos diferentes: acesso com `.0`, `.1` e desestruturação com `let (a, b) = t;`.
- Arrays `[T; N]` têm tamanho fixo, que faz parte do tipo.
- Índices são `usize`, e todo acesso é verificado: lint na compilação ou *panic* em runtime, nunca lixo de memória.
- Um slice `&[T]` é uma vista emprestada de um pedaço de uma sequência. É o tipo ideal de parâmetro para "uma lista de coisas para ler".

## Exercícios

Veja o exercício 5 em [`exercicios.md`](exercicios.md).

## Leitura complementar

- [The Rust Programming Language, 3.2 Data Types (tuplas e arrays)](https://doc.rust-lang.org/stable/book/ch03-02-data-types.html#compound-types)
- [Comprehensive Rust: Tuples and Arrays](https://google.github.io/comprehensive-rust/tuples-and-arrays.html)
- [std: o tipo primitivo `slice`](https://doc.rust-lang.org/std/primitive.slice.html)

---

[← Lição anterior](04-controle-de-fluxo.md) · [Índice do módulo](README.md) · [Próxima lição →](06-macros-e-formatacao.md)

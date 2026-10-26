# 01 — `Box` e tipos recursivos

**Objetivos.** Ao final, você vai saber:

- o que é um `Box<T>` e como ele fica na memória;
- por que um tipo recursivo precisa de `Box` (E0072);
- quando usar `Box` e quando ele é desnecessário;
- como `Box` se compara ao `std::unique_ptr` do C++.

## Um dono, um valor no heap

Você já usou `Box` sem estudá-lo: `Box<dyn Error>` no Módulo 05, `Box<dyn Forma>` no Módulo 06, `Box<dyn Fn>` no Módulo 07. Lá ele era "um ponteiro dono para um valor no heap". É exatamente isso, e não muito mais:

```rust
let b = Box::new(5); // aloca um i32 no heap; `b` é o dono
let x = *b + 1;      // `*b` acessa o valor
println!("{b} {x}"); // Display atravessa a caixa
```

```text
5 6
```

`b` é um ponteiro de 8 bytes na stack, e o `5` mora no heap. Quando `b` sai de escopo, o `Drop` da `Box` libera o heap. As regras do Módulo 02 continuam valendo: `let c = b;` move a posse, e usar `b` depois é E0382. Um `Box` é o caso mais simples de *smart pointer* (ponteiro inteligente): um tipo que se comporta como ponteiro e, além disso, gerencia o recurso para onde aponta.

Os tamanhos, medidos no [`codigo/box-e-recursao`](codigo/box-e-recursao/src/main.rs) (máquina de 64 bits):

```text
i32:               4 bytes
Box<i32>:          8 bytes
Option<Box<i32>>:  8 bytes
Box<[i32]>:       16 bytes
Box<str>:         16 bytes
Vec<i32>:         24 bytes
Expr:             24 bytes
```

Duas linhas merecem atenção. `Option<Box<i32>>` tem os mesmos 8 bytes: um `Box` nunca é nulo, então o compilador usa o ponteiro nulo para representar `None`, sem campo extra. E `Box<[i32]>` e `Box<str>` têm 16, porque apontam para um tipo sem tamanho fixo e carregam o comprimento junto (o *fat pointer* do Módulo 06).

![Na stack, b: Box<i32> aponta para um 5 no heap. e: Expr guarda a tag Soma e duas Box, que apontam para Expr::Num(1) e Expr::Neg, que por sua vez aponta para Expr::Num(2). Embaixo, Option<Box<i32>> igual a None é só um ponteiro nulo de 8 bytes, sem alocação.](assets/box-layout.svg)

### Comparando com C++

`Box<T>` é o `std::unique_ptr<T>`, e `Box::new(x)` é o `std::make_unique<T>(x)`:

```cpp
auto b = std::make_unique<int>(5);
auto c = std::move(b);  // b agora é nulo
std::cout << *b;        // compila; comportamento indefinido em tempo de execução
```

```rust
let b = Box::new(5);
let c = b;              // b foi movido: usar b daqui em diante é E0382
```

Três diferenças: não existe `Box` nulo (quando a ausência faz sentido, ela fica explícita no tipo, com `Option<Box<T>>`, e custa zero bytes a mais); usar depois do move é erro de compilação, e não UB; e `*b` pode **mover o valor para fora** da caixa, o que nenhum outro ponteiro permite:

```rust
let b = Box::new(String::from("dentro da caixa"));
let s: String = *b; // move a String para fora; a caixa é liberada
```

Em Python, a pergunta nem aparece: todo objeto já mora no heap, e toda variável é uma referência. Em Rust, o padrão é o valor ficar onde foi declarado (na stack, dentro de uma struct, dentro de um `Vec`). O heap é uma escolha explícita.

## Quando você precisa de `Box`

Na maioria do código, você não precisa: `Vec`, `String` e `HashMap` já guardam os dados no heap e cuidam da memória. `Box` aparece em três situações.

**1. Um tipo que contém a si mesmo.** É o caso clássico, e é onde o compilador obriga.

**2. Um valor cujo tamanho não se conhece na compilação**, como `dyn Trait`: `Box<dyn Error>`, `Vec<Box<dyn Forma>>`. Você já fez isso no Módulo 06.

**3. Um valor grande que você quer mover barato.** Mover um `Box` copia 8 bytes, por maior que seja o conteúdo. O caso típico é a variante enorme de um enum, que o clippy aponta com `large_enum_variant` (Módulo 04): como o enum tem o tamanho da maior variante, uma variante com um array de 1 KB faz todas ocuparem 1 KB. Trocar o campo por `Box<[u8; 1024]>` deixa a variante com 8 bytes.

## Tipos recursivos

Uma árvore de expressões aritméticas: um número, uma soma de duas expressões, um produto, uma negação. A primeira tentativa é escrever exatamente isso, `Soma(Expr, Expr)`, e ela não compila. O erro aparece igual no exemplo mais curto possível, a lista encadeada do Rust Book:

```rust
// ❌ NÃO COMPILA
#[derive(Debug)]
enum Lista {
    Cons(i32, Lista),
    Vazia,
}
```

```text
error[E0072]: recursive type `Lista` has infinite size
 --> src/main.rs:2:1
  |
2 | enum Lista {
  | ^^^^^^^^^^
3 |     Cons(i32, Lista),
  |               ----- recursive without indirection
  |
help: insert some indirection (e.g., a `Box`, `Rc`, or `&`) to break the cycle
  |
3 |     Cons(i32, Box<Lista>),
  |               ++++     +
```

O que o compilador está dizendo: o tamanho de `Lista` é o tamanho de um `i32` mais o tamanho de uma `Lista`, que é um `i32` mais uma `Lista`... A conta não fecha. Em C++ você nunca escreveria `struct No { int v; No prox; };` pelo mesmo motivo: o membro precisa ser `No*` ou `unique_ptr<No>`. A *indirection* (indireção) que o `help:` pede é isso: guardar um ponteiro, que tem tamanho fixo, em vez do valor.

```rust
// ✅
#[derive(Debug)]
enum Expr {
    Num(f64),
    Soma(Box<Expr>, Box<Expr>),
    Mult(Box<Expr>, Box<Expr>),
    Neg(Box<Expr>),
}
```

Agora `Expr` tem 24 bytes: a tag mais dois ponteiros, no pior caso. Avaliar é um `match` recursivo, e o `Box` é transparente: `a.avaliar()` funciona com `a: &Box<Expr>` porque a chamada de método atravessa a caixa sozinha (*auto-deref*, explicado na próxima lição).

```rust
use Expr::{Mult, Neg, Num, Soma};

impl Expr {
    fn avaliar(&self) -> f64 {
        match self {
            Num(x) => *x,
            Soma(a, b) => a.avaliar() + b.avaliar(),
            Mult(a, b) => a.avaliar() * b.avaliar(),
            Neg(a) => -a.avaliar(),
        }
    }
}
```

Para montar árvores sem espalhar `Box::new` pelo código, o projeto tem funções auxiliares (`soma`, `mult`, `neg`). Um método que **consome** a árvore usa o `*caixa` para tirar o valor de dentro, como nesta simplificação de dupla negação:

```rust
fn simplificar(self) -> Expr {
    match self {
        Neg(dentro) => match *dentro {
            Neg(e) => e.simplificar(),
            outro => Neg(Box::new(outro.simplificar())),
        },
        // ... Soma, Mult e Num reconstroem a árvore
    }
}
```

```text
(2 * (3 + --4)) = 14
(2 * (3 + 4)) = 14
```

O `match *dentro` move a `Expr` para fora do `Box`: o padrão `Neg(e)` pega o `Box<Expr>` de dentro sem copiar nada. Código completo em [`codigo/box-e-recursao`](codigo/box-e-recursao/src/main.rs).

> [!NOTE]
> E a lista encadeada? Ela é o exemplo de livro, mas raramente o certo na prática: um `Vec` é mais rápido em quase tudo, porque os elementos ficam contíguos na memória. O livro [Too Many Linked Lists](https://rust-unofficial.github.io/too-many-lists/) mostra, em detalhe, por que listas encadeadas são difíceis em Rust. Leia como exercício de *ownership*, sabendo que foi escrito para uma edition antiga.

### `Option<Box<T>>`: o ponteiro que pode faltar

Árvores binárias e listas costumam ter filhos opcionais. O tipo idiomático é `Option<Box<No>>`, que, como você viu, tem os mesmos 8 bytes de um ponteiro cru. O exercício 1 constrói uma árvore de busca assim. Dois métodos do `Option` aparecem muito nesse tipo de código:

- `take()` tira o valor e deixa `None` no lugar, útil para mover um filho para fora através de um `&mut`;
- `as_deref()` transforma `&Option<Box<T>>` em `Option<&T>`.

## `Box<[T]>` e `Box<str>`

`vec.into_boxed_slice()` e `String::into_boxed_str()` (ou `.into()`) jogam fora a capacidade extra: o `Box<[T]>` guarda só ponteiro e comprimento (16 bytes, contra 24 do `Vec`). Vale para dados que não crescem mais, guardados aos milhares. Na dúvida, fique com `Vec` e `String`.

## Armadilhas comuns

**Envolver tudo em `Box` "para ficar no heap".** Quem vem de Java ou de C++ antigo tende a fazer `Box::new` em todo objeto. Em Rust isso só acrescenta uma alocação e uma indireção. Um `Vec<Box<Ponto>>` é quase sempre pior que um `Vec<Ponto>`. Use `Box` nas três situações da lista acima.

**`Box<Vec<T>>`.** O `Vec` já guarda os elementos no heap. A caixa por fora só acrescenta um nível de ponteiro; o clippy avisa (`box_collection`).

**Esperar que `Box<T>` mude o comportamento de `&T`.** Uma função que só lê deve pedir `&T`, e não `&Box<T>`. Graças à *deref coercion*, um `&Box<T>` vira `&T` sozinho na chamada, e a função fica mais geral (clippy: `borrowed_box`).

**Recursão profunda no `Drop`.** Descartar uma lista de um milhão de nós encadeados com `Box` faz um milhão de chamadas de `drop` aninhadas, e pode estourar a pilha. É um dos motivos do conselho acima (use `Vec`). Quem precisa mesmo de uma lista longa escreve um `Drop` iterativo, como o Too Many Linked Lists mostra.

## Resumo

- `Box<T>` é um dono único de um valor no heap: um ponteiro de 8 bytes que libera a memória no `drop`. É o `unique_ptr` do C++, sem estado nulo e sem uso depois do move.
- `Option<Box<T>>` também tem 8 bytes: o ponteiro nulo representa `None`.
- Tipos recursivos precisam de indireção (E0072): `Box<Expr>` no lugar de `Expr`.
- Use `Box` para tipos recursivos, `dyn Trait` e valores grandes que você quer mover barato. Fora disso, `Vec`, `String` e valores comuns bastam.
- `*b` move o valor para fora do `Box`, o que só o `Box` permite.

## Exercícios

Veja o exercício 1 em [`exercicios.md`](exercicios.md).

## Leitura complementar

- [The Rust Programming Language, 15.1 Using `Box<T>` to Point to Data on the Heap](https://doc.rust-lang.org/stable/book/ch15-01-box.html)
- [Documentação de `std::boxed`](https://doc.rust-lang.org/std/boxed/index.html) (inclui o layout garantido do `Box`)
- [Comprehensive Rust: Box](https://google.github.io/comprehensive-rust/smart-pointers/box.html)
- [Learning Rust With Entirely Too Many Linked Lists, cap. 2](https://rust-unofficial.github.io/too-many-lists/first.html)

---

[← Índice do módulo](README.md) · [Próxima lição →](02-deref-e-drop.md)

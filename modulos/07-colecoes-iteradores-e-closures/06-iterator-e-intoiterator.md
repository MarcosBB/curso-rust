# 06 — `Iterator` e `IntoIterator`

**Objetivos.** Ao final, você vai saber:

- o que é a trait `Iterator` e o que o `for` faz por baixo;
- a diferença entre `iter()`, `iter_mut()` e `into_iter()`, e o papel de `IntoIterator`;
- por que aparece `&&x` num `filter`, e por que `iter().collect()` dá `Vec<&T>`;
- receber "qualquer coisa que dá para percorrer" com `impl IntoIterator`.

## Um objeto que sabe dar o próximo

Num `for (int i = 0; i < n; i++)` do C, quatro coisas ficam espalhadas: o estado (`i`), a condição de parada, o avanço e o acesso ao elemento (`v[i]`). Um *iterador* junta tudo num objeto com um único método: "me dê o próximo, se houver".

Em Rust, isso é a trait `Iterator`:

```rust
trait Iterator {
    type Item;
    fn next(&mut self) -> Option<Self::Item>;
    // ... e uns 75 métodos com implementação padrão (lição 07)
}
```

`type Item` é um *tipo associado* (Módulo 06): cada iterador diz que tipo de item produz. `next` devolve `Some(item)` enquanto houver itens e `None` quando acabar. E recebe `&mut self`, porque avançar muda o estado do iterador:

```rust
let v = vec![10, 20, 30];
let mut it = v.iter();
println!("{:?}", it.next());
println!("{:?}", it.next());
println!("{:?}", it.next());
println!("{:?}", it.next());
```

```text
Some(10)
Some(20)
Some(30)
None
```

Sem o `mut` em `let mut it`, o primeiro `next` dá o E0596. Repare também que `Some(10)` é, na verdade, um `Some(&10)`: o `Debug` de uma referência mostra o valor. `v.iter()` produz **referências** para os elementos, e isso vai importar daqui a pouco.

Comparando: o protocolo de iteração do Python tem `__iter__` (dá um iterador) e `__next__` (dá o próximo ou lança `StopIteration`). Em Rust, `__next__` é `next`, e o `StopIteration` é o `None`. O `__iter__` é a próxima trait. Do lado do C++, um iterador é um **par** (`begin` e `end`) que você compara a cada passo. Em Rust, é um objeto só, que sabe quando acabou.

## O que o `for` faz

O `for` é açúcar sintático. Este laço:

```rust
for x in v {
    println!("{x}");
}
```

vira, conceitualmente:

```rust
match IntoIterator::into_iter(v) {
    mut iter => loop {
        match iter.next() {
            Some(x) => println!("{x}"),
            None => break,
        }
    },
}
```

O `for` não chama `next` direto em `v`. Ele primeiro chama `IntoIterator::into_iter(v)` para **obter** um iterador, e então chama `next` até vir `None`.

## `IntoIterator`: "dá para percorrer"

```rust
trait IntoIterator {
    type Item;
    type IntoIter: Iterator<Item = Self::Item>;
    fn into_iter(self) -> Self::IntoIter;
}
```

`IntoIterator` é a trait das coisas que **sabem produzir** um iterador. Repare no `self` por valor: `into_iter` consome o que recebe. Isso explica a regra do [kit do Módulo 01](../01-tour-da-sintaxe/07-kit-vec-e-hashmap.md) e o *move* do [Módulo 02](../02-ownership/02-regras-de-ownership-e-move.md): `for x in v` passa `v` para `into_iter` e, por isso, consome o vetor.

E por que `for x in &v` não consome? Porque `Vec<T>` tem **três** implementações de `IntoIterator`, uma para cada forma de acesso:

| Você escreve | Implementação usada | Itens | Atalho | O `Vec` depois |
|---|---|---|---|---|
| `for x in v` | `IntoIterator for Vec<T>` | `T` (por valor) | `v.into_iter()` | consumido |
| `for x in &v` | `IntoIterator for &Vec<T>` | `&T` | `v.iter()` | intacto |
| `for x in &mut v` | `IntoIterator for &mut Vec<T>` | `&mut T` | `v.iter_mut()` | intacto, talvez alterado |

`&v` é um valor do tipo `&Vec<T>`, e `into_iter` consome **a referência**, não o vetor. `iter()` e `iter_mut()` são métodos de conveniência que fazem o mesmo que `(&v).into_iter()` e `(&mut v).into_iter()`:

```rust
let mut v = vec![String::from("a"), String::from("b")];

for s in v.iter() {
    // s: &String
    print!("{s} ");
}
for s in v.iter_mut() {
    // s: &mut String
    s.push('!');
}
for s in v.into_iter() {
    // s: String (o vetor é consumido)
    print!("{s} ");
}
println!();
```

```text
a b a! b! 
```

Todo `Iterator` também é `IntoIterator` (ele devolve a si mesmo), e é por isso que você pode escrever `for x in v.iter().rev()`. As outras coleções seguem o mesmo padrão: `for (k, v) in &mapa` produz `(&K, &V)`. Arrays têm `into_iter` por valor desde a edition 2021 (`for s in [String::from("x")]` dá `String`), e `Box<[T]>` desde a edition 2024.

## O porquê do `&&x` no `filter`

O [kit do Módulo 01](../01-tour-da-sintaxe/08-kit-closures.md) avisou que, às vezes, o parâmetro do `filter` é uma referência para uma referência. Agora dá para explicar:

```rust
// ❌ NÃO COMPILA
fn main() {
    let v = vec![1, 5, 2, 8];
    let grandes: Vec<&i32> = v.iter().filter(|x| x > 3).collect();
    println!("{grandes:?}");
}
```

```text
error[E0308]: mismatched types
 --> src/main.rs:3:54
  |
3 |     let grandes: Vec<&i32> = v.iter().filter(|x| x > 3).collect();
  |                                                      ^ expected `&&_`, found integer
  |
  = note: expected reference `&&_`
                  found type `{integer}`
```

Duas camadas de `&`, cada uma com o seu motivo:

1. `v.iter()` produz `&i32` (o `Item` é `&i32`).
2. `filter` precisa **olhar** o item para decidir e, se aprovar, **entregá-lo** adiante. Se passasse o item por valor para o predicado, não teria mais o item para entregar. Então o predicado recebe `&Self::Item`, uma referência para o item. Com `Item = &i32`, isso dá `&&i32`.

Há quatro saídas equivalentes. Escolha a que lê melhor:

```rust
let v = vec![1, 5, 2, 8];
let a: Vec<&i32> = v.iter().filter(|x| **x > 3).collect(); // desreferencia duas vezes
let b: Vec<&i32> = v.iter().filter(|&&x| x > 3).collect(); // desestrutura no padrão
let c: Vec<i32> = v.iter().copied().filter(|&x| x > 3).collect(); // copia antes
let d: Vec<i32> = v.iter().map(|x| x * 10).collect(); // map recebe &i32
println!("{a:?} {b:?} {c:?} {d:?}");
```

```text
[5, 8] [5, 8] [5, 8] [10, 50, 20, 80]
```

`map`, ao contrário de `filter`, recebe o item **por valor** (ele vai transformar o item, e não devolvê-lo). Com `iter()`, o item é `&i32`, e `x * 10` funciona porque a multiplicação está implementada para `&i32`. Com tuplas e structs (`p.idade > 30`), o `&&` não aparece, porque o acesso a campo desreferencia sozinho.

`copied()` (para tipos `Copy`) e `cloned()` (para tipos `Clone`) transformam um iterador de `&T` num iterador de `T`. Para números, `.copied()` logo depois do `.iter()` costuma ser o mais limpo.

## `iter()` empresta, então `collect` monta referências

```rust
// ❌ NÃO COMPILA
fn main() {
    let nomes = vec![String::from("Ana"), String::from("Bia")];
    let copia: Vec<String> = nomes.iter().collect();
    println!("{copia:?}");
}
```

```text
error[E0277]: a value of type `Vec<String>` cannot be built from an iterator over elements of type `&String`
    --> src/main.rs:3:43
     |
   3 |     let copia: Vec<String> = nomes.iter().collect();
     |                                           ^^^^^^^ value of type `Vec<String>` cannot be built from `std::iter::Iterator<Item=&String>`
     |
help: the trait `FromIterator<&String>` is not implemented for `Vec<String>`
      but trait `FromIterator<String>` is implemented for it
...
note: the method call chain might not have had the expected associated types
    --> src/main.rs:3:36
     |
   2 |     let nomes = vec![String::from("Ana"), String::from("Bia")];
     |                 ---------------------------------------------- this expression has type `Vec<String>`
   3 |     let copia: Vec<String> = nomes.iter().collect();
     |                                    ^^^^^^ `Iterator::Item` is `&String` here
```

A última nota é a que importa: depois de `.iter()`, o `Item` é `&String`. Um iterador de referências monta um `Vec<&String>`. Você decide qual dos três quer:

```rust
let nomes = vec![String::from("Ana"), String::from("Bia")];
let refs: Vec<&String> = nomes.iter().collect(); // empresta
let copias: Vec<String> = nomes.iter().cloned().collect(); // copia
let movidas: Vec<String> = nomes.into_iter().collect(); // consome `nomes`
```

A escolha entre `iter()` e `into_iter()` é a mesma entre emprestar e tomar posse, que você pratica desde o [Módulo 02](../02-ownership/README.md). Se você não precisa mais da coleção original, `into_iter()` evita as cópias.

## Um iterador se gasta

`sum`, `count`, `collect` e outros consumidores recebem o iterador **por valor** (`self`). Depois de usá-lo, ele foi movido:

```rust
// ❌ NÃO COMPILA
fn main() {
    let v = vec![1, 2, 3];
    let it = v.iter();
    let total: i32 = it.sum();
    let n = it.count();
    println!("{total} {n}");
}
```

```text
error[E0382]: use of moved value: `it`
    --> src/main.rs:5:13
     |
   3 |     let it = v.iter();
     |         -- move occurs because `it` has type `std::slice::Iter<'_, i32>`, which does not implement the `Copy` trait
   4 |     let total: i32 = it.sum();
     |                         ----- `it` moved due to this method call
   5 |     let n = it.count();
     |             ^^ value used here after move
     |
note: `std::iter::Iterator::sum` takes ownership of the receiver `self`, which moves `it`
...
```

Não é só formalidade: o `sum` percorreu o iterador até o fim, e um segundo uso não teria nada para contar. Em Python, um gerador esgotado devolve vazio em silêncio (`sum(g)` e depois `len(list(g))` dá `0`). Em Rust, o compilador barra. A correção é criar outro iterador a partir da coleção: `v.iter().count()`, ou simplesmente `v.len()`.

### Consumir só uma parte: `by_ref`

Às vezes você quer consumir o começo de um iterador e continuar depois com o resto. `by_ref()` empresta o iterador (como `&mut it`), e o adaptador gasta só o que precisa:

```rust
let texto = "titulo: Relatório\nautor: Ana\n\nlinha 1\nlinha 2";
let mut linhas = texto.lines();

let cabecalho: Vec<&str> = linhas.by_ref().take_while(|l| !l.is_empty()).collect();
let corpo: Vec<&str> = linhas.collect();
println!("{cabecalho:?}");
println!("{corpo:?}");
```

```text
["titulo: Relatório", "autor: Ana"]
["linha 1", "linha 2"]
```

Repare que a linha vazia sumiu: `take_while` precisa **consumir** o primeiro item que falha no teste para descobrir que deve parar. Aqui, é o que queríamos (a linha vazia é só o separador). Em outros casos, é uma surpresa. Se o item que falha precisa ficar, use `peekable` e `next_if` (lição 07).

## Receber qualquer coisa percorrível

Como o `for` aceita qualquer `IntoIterator`, uma função também pode aceitar. É o jeito mais flexível de receber uma sequência:

```rust
fn total(precos: impl IntoIterator<Item = u32>) -> u32 {
    let mut soma = 0;
    for p in precos {
        soma += p;
    }
    soma
}

fn main() {
    let v = vec![10, 20];
    println!("{}", total(v.iter().copied())); // um iterador
    println!("{}", total(v)); // um Vec, por valor
    println!("{}", total([1, 2, 3])); // um array
    println!("{}", total(1..=4)); // um range
}
```

```text
30
30
6
10
```

O `impl IntoIterator<Item = u32>` (a sintaxe `impl Trait` em argumento, do [Módulo 06](../06-traits-e-generics/README.md)) diz: "aceito qualquer coisa que produza `u32`s". Compare com `&[u32]`, que só aceitaria dados já guardados num vetor ou array. E com `Item = String`, a função pode receber as `String` **por valor** e ficar com elas, sem copiar.

### Lembra do minigrep?

No [Projeto 1](../../projetos/01-minigrep/README.md), `Config::from_args` recebe `args: &[String]`, uma fatia emprestada. Por isso precisa copiar o padrão (`to_string`) e a opção desconhecida (`clone`), e o `main` precisa juntar `env::args()` num `Vec` antes de chamar. Com o que você viu aqui, a assinatura melhor é:

```rust
pub fn from_args(
    args: impl IntoIterator<Item = String>,
    ignore_case_env: bool,
) -> Result<Config, ArgsError>
```

O `main` passa `env::args().skip(1)` direto, os testes passam um `Vec<String>`, e cada `String` é **movida** para dentro do `Config`, sem cópia. É a parte (b) do exercício 6.

## Armadilhas comuns

- **`let it = v.iter();` e depois `it.next()`** (E0596): `next` precisa de `&mut self`. Declare `let mut it`.
- **`filter(|x| x > 3)` sobre `.iter()`** (E0308, "expected `&&_`"): use `**x`, `|&&x|` ou `.copied()` antes.
- **`iter().collect()` esperando `Vec<String>`** (E0277): o item é `&String`. Use `cloned()` ou `into_iter()`.
- **Usar o iterador depois de `sum`/`count`/`collect`** (E0382): crie outro, ou use `by_ref()` quando quer continuar de onde parou.
- **`into_iter()` quando ainda vai usar a coleção**: ele a consome. Para só ler, `iter()`.

## Resumo

- `Iterator` tem `type Item` e `next(&mut self) -> Option<Item>`. `None` é o fim.
- O `for` chama `IntoIterator::into_iter` e depois `next` até `None`.
- `Vec<T>` tem três `IntoIterator`: por valor (`T`), `&Vec` (`&T`, o `iter()`) e `&mut Vec` (`&mut T`, o `iter_mut()`).
- `filter` passa `&Item` ao predicado: com `iter()`, isso dá `&&T`. `map` passa o item por valor.
- Consumidores tomam o iterador por valor; `by_ref()` empresta para consumir só uma parte.
- `impl IntoIterator<Item = T>` aceita `Vec`, arrays, ranges e iteradores.

## Exercícios

Veja os exercícios 2, 4 e 6 em [`exercicios.md`](exercicios.md).

## Leitura complementar

- [The Rust Programming Language, 13.2 Processing a Series of Items with Iterators](https://doc.rust-lang.org/stable/book/ch13-02-iterators.html)
- [Documentação de `std::iter`](https://doc.rust-lang.org/std/iter/index.html): as três formas de iteração e o `for` desaçucarado
- [Documentação de `IntoIterator`](https://doc.rust-lang.org/std/iter/trait.IntoIterator.html)
- [Comprehensive Rust: Iterators](https://google.github.io/comprehensive-rust/iterators.html), em especial *IntoIterator*

---

[← Lição anterior](05-fn-fnmut-fnonce.md) · [Índice do módulo](README.md) · [Próxima lição →](07-adaptadores-e-consumidores.md)

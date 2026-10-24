# 08 — Implementando seu próprio iterador

**Objetivos.** Ao final, você vai saber:

- implementar `Iterator` para uma struct e ganhar todos os adaptadores de graça;
- escrever um iterador que empresta dados (com *lifetime*);
- fazer o `for x in &minha_colecao` funcionar com `IntoIterator`;
- criar iteradores sem struct (`from_fn`, `successors`) e devolver iteradores de funções.

## Uma struct com estado e um `next`

Para criar um iterador, você precisa de duas coisas: uma struct que guarda o estado da iteração e uma implementação de `Iterator` com o `type Item` e o `next`:

```rust
struct Regressiva {
    atual: u32,
}

impl Iterator for Regressiva {
    type Item = u32;

    fn next(&mut self) -> Option<u32> {
        if self.atual == 0 {
            return None;
        }
        self.atual -= 1;
        Some(self.atual + 1)
    }
}

fn main() {
    let v: Vec<u32> = Regressiva { atual: 3 }.collect();
    println!("{v:?}");

    let soma: u32 = Regressiva { atual: 10 }.filter(|n| n % 2 == 0).map(|n| n * n).sum();
    let pares: Vec<(u32, char)> = Regressiva { atual: 3 }.zip("abc".chars()).collect();
    println!("{soma} {pares:?}");
}
```

```text
[3, 2, 1]
220 [(3, 'a'), (2, 'b'), (1, 'c')]
```

Você escreveu só o `next`, e `collect`, `filter`, `map`, `sum`, `zip` e os outros ~75 métodos da lição 07 já funcionam. Eles têm implementação padrão na trait, escrita em termos de `next` (o [Módulo 06](../06-traits-e-generics/README.md) explica métodos padrão). É a mesma vantagem de implementar `__next__` em Python e ganhar `sum`, `zip` e `itertools` de graça, só que verificada em tempo de compilação.

Esquecer o tipo associado dá um erro direto:

```rust
// ❌ NÃO COMPILA
struct Regressiva {
    atual: u32,
}

impl Iterator for Regressiva {
    fn next(&mut self) -> Option<u32> {
        None
    }
}
```

```text
error[E0046]: not all trait items implemented, missing: `Item`
 --> src/main.rs:5:1
  |
5 | impl Iterator for Regressiva {
  | ^^^^^^^^^^^^^^^^^^^^^^^^^^^^ missing `Item` in implementation
  |
  = help: implement the missing item: `type Item = /* Type */;`
```

### Terminar com `?`

Um iterador pode terminar por qualquer motivo, não só por "acabaram os dados". O Fibonacci em `u64` estoura perto do 93º termo, e em *debug* isso seria um *panic*. Com `checked_add` e o `?` em `Option` (do [Módulo 05](../05-tratamento-de-erros/02-result-e-o-operador-interrogacao.md)), o estouro vira o fim do iterador:

```rust
struct Fibonacci {
    a: u64,
    b: u64,
}

impl Iterator for Fibonacci {
    type Item = u64;

    fn next(&mut self) -> Option<u64> {
        let atual = self.a;
        let proximo = self.a.checked_add(self.b)?; // estourou: fim do iterador
        self.a = self.b;
        self.b = proximo;
        Some(atual)
    }
}

fn fibonacci() -> Fibonacci {
    Fibonacci { a: 0, b: 1 }
}

fn main() {
    let dez: Vec<u64> = fibonacci().take(10).collect();
    println!("{dez:?}");
    println!("termos devolvidos: {}", fibonacci().count());
    let soma_pares: u64 = fibonacci().take_while(|&x| x < 100).filter(|x| x % 2 == 0).sum();
    println!("{soma_pares}");
}
```

```text
[0, 1, 1, 2, 3, 5, 8, 13, 21, 34]
termos devolvidos: 92
44
```

Uma função construtora (`fibonacci()`) esconde os campos iniciais, e quem usa só vê "um iterador de `u64`".

### O que não vem de graça

Alguns métodos exigem mais que `next`. `rev`, por exemplo, precisa andar de trás para frente (o `main` abaixo usa a `Regressiva` do início da lição, e as linhas do erro contam o arquivo inteiro):

```rust
// ❌ NÃO COMPILA
fn main() {
    let v: Vec<u32> = Regressiva { atual: 3 }.rev().collect();
    println!("{v:?}");
}
```

```text
error[E0277]: the trait bound `Regressiva: DoubleEndedIterator` is not satisfied
    --> src/main.rs:18:47
     |
  18 |     let v: Vec<u32> = Regressiva { atual: 3 }.rev().collect();
     |                                               ^^^ unsatisfied trait bound
     |
help: the trait `DoubleEndedIterator` is not implemented for `Regressiva`
...
note: required by a bound in `rev`
...
3441 |         Self: Sized + DoubleEndedIterator,
     |                       ^^^^^^^^^^^^^^^^^^^ required by this bound in `Iterator::rev`
```

`DoubleEndedIterator` acrescenta um `next_back`. Implemente-o se fizer sentido para o seu tipo (aqui, seria contar de baixo para cima). `ExactSizeIterator` (com `len`) é outra trait opcional do mesmo tipo. Para a maioria dos iteradores, `next` basta.

## Um iterador que empresta

Iteradores sobre dados que já existem guardam uma referência para eles, e a struct carrega um *lifetime*, como as structs com referência do [Módulo 03](../03-borrowing-e-lifetimes/06-lifetimes-em-structs-e-static.md). Este devolve as palavras de um texto como fatias, sem copiar nada:

```rust
struct Palavras<'a> {
    resto: &'a str,
}

impl<'a> Iterator for Palavras<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<&'a str> {
        let texto = self.resto.trim_start();
        if texto.is_empty() {
            return None;
        }
        let fim = texto.find(' ').unwrap_or(texto.len());
        let (palavra, resto) = texto.split_at(fim);
        self.resto = resto;
        Some(palavra)
    }
}

fn palavras(texto: &str) -> Palavras<'_> {
    Palavras { resto: texto }
}

fn main() {
    let texto = String::from("  borrow   checker feliz ");
    let v: Vec<&str> = palavras(&texto).collect();
    println!("{v:?}");
    println!("{:?}", palavras(&texto).max_by_key(|p| p.len()));
}
```

```text
["borrow", "checker", "feliz"]
Some("checker")
```

O ponto importante é `type Item = &'a str`: os itens vivem tanto quanto **o texto**, e não tanto quanto o iterador. Por isso dá para coletar as palavras num `Vec` e descartar o iterador. O `Palavras<'_>` no retorno de `palavras` é o lifetime anônimo do [Módulo 03](../03-borrowing-e-lifetimes/05-elisao-e-lifetime-anonimo.md): "o mesmo do argumento".

## `for x in &colecao`: implementar `IntoIterator`

Uma coleção sua não é um iterador; ela **produz** iteradores. Sem mais nada, o `for` não sabe o que fazer com ela:

```rust
// ❌ NÃO COMPILA
struct Playlist {
    musicas: Vec<String>,
}

fn main() {
    let p = Playlist { musicas: vec![String::from("Asa Branca")] };
    for m in &p {
        println!("{m}");
    }
}
```

```text
error[E0277]: `&Playlist` is not an iterator
 --> src/main.rs:7:14
  |
7 |     for m in &p {
  |              ^^ `&Playlist` is not an iterator
  |
  = help: the trait `Iterator` is not implemented for `&Playlist`
  = note: required for `&Playlist` to implement `IntoIterator`
```

A `note:` diz o que falta: `IntoIterator for &Playlist`. Quando os dados já estão num `Vec`, você não precisa escrever um iterador: basta **delegar** para o do `Vec`. A convenção da std é oferecer as três formas, como o `Vec` faz (lição 06), e um método `iter()`:

```rust
struct Playlist {
    musicas: Vec<String>,
}

impl Playlist {
    fn iter(&self) -> std::slice::Iter<'_, String> {
        self.musicas.iter()
    }
}

impl<'a> IntoIterator for &'a Playlist {
    type Item = &'a String;
    type IntoIter = std::slice::Iter<'a, String>;

    fn into_iter(self) -> Self::IntoIter {
        self.musicas.iter()
    }
}

impl IntoIterator for Playlist {
    type Item = String;
    type IntoIter = std::vec::IntoIter<String>;

    fn into_iter(self) -> Self::IntoIter {
        self.musicas.into_iter()
    }
}

fn main() {
    let p = Playlist { musicas: vec![String::from("Asa Branca"), String::from("Garota de Ipanema")] };
    for m in &p {
        println!("tocando {m}");
    }
    println!("{}", p.iter().map(|m| m.len()).sum::<usize>());
    let todas: Vec<String> = p.into_iter().collect();
    println!("{todas:?}");
}
```

```text
tocando Asa Branca
tocando Garota de Ipanema
27
["Asa Branca", "Garota de Ipanema"]
```

`std::slice::Iter<'a, T>` é o tipo que `Vec::iter()` devolve, e `std::vec::IntoIter<T>` o que `Vec::into_iter()` devolve. Os dois tipos associados de `IntoIterator` (`Item` e `IntoIter`) são exatamente esses. A implementação para `&mut Playlist` seguiria o mesmo molde, com `std::slice::IterMut`.

## Devolver um iterador de uma função

Quando o iterador é uma cadeia de adaptadores, o tipo real é impronunciável (algo como `Filter<RangeInclusive<u32>, {closure}>`). `impl Iterator<Item = T>` no retorno resolve, como o `impl Fn` da lição 05:

```rust
fn multiplos(de: u32, ate: u32) -> impl Iterator<Item = u32> {
    (1..=ate).filter(move |n| n % de == 0)
}

fn longas<'a>(palavras: &'a [&'a str], min: usize) -> impl Iterator<Item = &'a str> {
    palavras.iter().copied().filter(move |p| p.len() >= min)
}

fn main() {
    let m: Vec<u32> = multiplos(3, 20).collect();
    let l: Vec<&str> = longas(&["a", "casa", "sol", "janela"], 4).collect();
    println!("{m:?} {l:?}");
}
```

```text
[3, 6, 9, 12, 15, 18] ["casa", "janela"]
```

O `move` é obrigatório: as closures capturam `de` e `min`, que são parâmetros e morrem quando a função retorna (o E0373 da lição 04). E quem chama decide o que fazer com o iterador: coletar, somar, pegar só os 3 primeiros. Nada é calculado à toa. Na edition 2024, o `impl Iterator` no retorno captura automaticamente os lifetimes dos parâmetros (o [Módulo 06](../06-traits-e-generics/README.md) trata da regra), então `longas` não precisa de nenhuma anotação extra para devolver um iterador que empresta `palavras`.

## Iteradores sem struct

Para iteradores pequenos, escrever uma struct é cerimônia demais. A std tem atalhos em `std::iter`:

```rust
use std::iter;

// from_fn: a closure FnMut guarda o estado
let mut n = 0;
let contador = iter::from_fn(|| {
    n += 1;
    if n <= 3 { Some(n) } else { None }
});
println!("{:?}", contador.collect::<Vec<_>>());

// successors: cada item é calculado a partir do anterior
let potencias: Vec<u8> = iter::successors(Some(1u8), |x| x.checked_mul(2)).collect();
println!("{potencias:?}");

let collatz: Vec<u64> = iter::successors(Some(6u64), |&x| {
    if x == 1 { None } else if x % 2 == 0 { Some(x / 2) } else { Some(3 * x + 1) }
})
.collect();
println!("{collatz:?}");

// repeat_n, once, empty
let linha: String = iter::repeat_n('-', 5).chain(iter::once('>')).collect();
let nada: Vec<i32> = iter::empty().collect();
println!("{linha} {nada:?}");
```

```text
[1, 2, 3]
[1, 2, 4, 8, 16, 32, 64, 128]
[6, 3, 10, 5, 16, 8, 4, 2, 1]
-----> []
```

- `from_fn(closure)`: cada `next` chama a closure. O estado mora nas variáveis capturadas, e por isso a closure é `FnMut` (lição 05).
- `successors(primeiro, f)`: começa em `primeiro` e calcula o próximo a partir do anterior, até `f` devolver `None`. Com `checked_*`, para no estouro.
- `repeat_n(x, n)`, `once(x)`, `empty()` e `repeat(x)` (infinito) são as peças prontas para montar com `chain`.

### E os geradores do Python?

Em Python, um gerador com `yield` é o jeito natural de escrever um iterador:

```python
def fibonacci():
    a, b = 0, 1
    while True:
        yield a
        a, b = b, a + b
```

Rust estável **não tem** `yield`. Os blocos `gen` existem só no compilador *nightly*, e a palavra `gen` foi reservada na edition 2024 justamente para isso. Hoje, o equivalente é uma das três formas desta lição: struct + `impl Iterator` (quando há estado com nome e lógica), `from_fn`/`successors` (quando o estado cabe numa closure) ou uma cadeia de adaptadores devolvida como `impl Iterator`. A diferença de modelo é a mesma da lição 07: o gerador do Python **empurra** valores com `yield`; o iterador Rust é **puxado** por `next`, e você escreve explicitamente onde o estado fica entre uma chamada e outra.

> [!NOTE]
> `0..n` cria um `std::ops::Range`, que já é um iterador e por isso não é `Copy` (copiar um iterador sem querer seria uma fonte de bugs). Desde a 1.95/1.96, existem os tipos novos `core::range::Range`, `RangeInclusive` e `RangeFrom`, que são `Copy` e **não** são iteradores, só `IntoIterator`: `let r = core::range::Range::from(1..4);` pode ser copiado e depois percorrido com `r.into_iter()`. A sintaxe `a..b` continua criando o tipo antigo; a troca deve acontecer numa edition futura. Por ora, é só para saber que existe.

Código completo em [`codigo/iterador-proprio`](codigo/iterador-proprio/src/lib.rs), com testes: os iteradores desta lição e uma `RegressivaDupla` que implementa `DoubleEndedIterator` (e por isso aceita `rev()`).

## Armadilhas comuns

- **Esquecer `type Item`** (E0046): siga o `help:`.
- **Chamar `rev` num iterador seu** (E0277, `DoubleEndedIterator`): implemente `next_back` ou desista do `rev` (colete num `Vec` e inverta).
- **`for x in &colecao` sem `IntoIterator for &Colecao`** (E0277, "is not an iterator"): implemente, delegando para o iterador do campo interno.
- **`Item` com o lifetime do iterador em vez do lifetime dos dados**: num iterador que empresta, o item deve ser `&'a T`, ligado aos dados. Não tente devolver uma referência para um campo do próprio iterador: o `next(&mut self)` não permite que o item viva mais que a chamada.
- **Iterador infinito sem `take`**: `collect`, `count`, `max` e `sum` nunca terminam. Use `take`, `take_while` ou `find`.
- **Esquecer o `move`** ao devolver `impl Iterator` com closures que capturam parâmetros (E0373).

## Resumo

- Implemente `Iterator` com `type Item` e `next(&mut self) -> Option<Item>`, e todos os adaptadores e consumidores vêm de graça.
- Um iterador que empresta tem lifetime: `struct Palavras<'a>` com `type Item = &'a str`.
- Para `for x in &colecao`, implemente `IntoIterator for &Colecao`, em geral delegando para `slice::Iter`. Ofereça também um método `iter()`.
- `-> impl Iterator<Item = T>` devolve cadeias de adaptadores sem nomear o tipo.
- Sem struct: `iter::from_fn`, `iter::successors`, `repeat_n`, `once`. Rust estável não tem `yield`.

## Exercícios

Veja os exercícios 4 e 6 em [`exercicios.md`](exercicios.md).

## Leitura complementar

- [The Rust Programming Language, 13.2 (seção "The `Iterator` Trait and the `next` Method")](https://doc.rust-lang.org/stable/book/ch13-02-iterators.html#the-iterator-trait-and-the-next-method)
- [Documentação de `std::iter`, seção *Implementing Iterator*](https://doc.rust-lang.org/std/iter/index.html#implementing-iterator)
- Documentação de [`iter::from_fn`](https://doc.rust-lang.org/std/iter/fn.from_fn.html) e [`iter::successors`](https://doc.rust-lang.org/std/iter/fn.successors.html)
- [Comprehensive Rust: IntoIterator](https://google.github.io/comprehensive-rust/iterators/intoiterator.html)
- [Documentação de `std::range`](https://doc.rust-lang.org/std/range/index.html) (os tipos novos)

---

[← Lição anterior](07-adaptadores-e-consumidores.md) · [Índice do módulo](README.md)

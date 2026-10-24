# 05 — `Fn`, `FnMut` e `FnOnce`

**Objetivos.** Ao final, você vai saber:

- o que cada uma das três traits promete, e como o corpo da closure decide qual ela implementa;
- que *bound* pedir numa função que recebe uma closure;
- ler os erros E0525, E0507 e E0382 com closures;
- usar ponteiros `fn`, guardar closures em structs e devolver closures (`impl Fn` x `Box<dyn Fn>`).

## As três traits

A lição anterior mostrou que uma closure é uma struct com os capturados como campos. Chamar a closure é chamar um método dessa struct, e há três formas de receber o `self`. Cada uma é uma *trait* da std (simplificadas aqui):

```text
trait FnOnce<Args> { type Output; fn call_once(self, args: Args) -> Self::Output; }
trait FnMut<Args>: FnOnce<Args> { fn call_mut(&mut self, args: Args) -> Self::Output; }
trait Fn<Args>: FnMut<Args> { fn call(&self, args: Args) -> Self::Output; }
```

- **`FnOnce`** recebe `self` por valor: a chamada consome a closure. Ela pode mover os capturados para fora, mas só pode ser chamada uma vez.
- **`FnMut`** recebe `&mut self`: pode alterar os capturados, e ser chamada várias vezes (mas não ao mesmo tempo).
- **`Fn`** recebe `&self`: só lê. Pode ser chamada várias vezes, inclusive por vários lugares ao mesmo tempo.

Você não implementa essas traits à mão. O compilador implementa, olhando o **corpo** da closure:

| O corpo... | A closure implementa |
|---|---|
| move algum capturado para fora (devolve, passa por valor, `drop`) | só `FnOnce` |
| altera algum capturado, sem mover nada para fora | `FnMut` (e `FnOnce`) |
| só lê (ou não captura nada) | `Fn` (e `FnMut` e `FnOnce`) |

A sintaxe `Fn(i32) -> i32` é o açúcar para escrever essas traits num *bound*: "algo que pode ser chamado com um `i32` e devolve um `i32`".

As três formam uma hierarquia: `Fn` é *subtrait* de `FnMut`, que é *subtrait* de `FnOnce` (o [Módulo 06](../06-traits-e-generics/README.md) explica subtraits). Na prática, isso quer dizer que **toda `Fn` serve onde se pede `FnMut`, e toda `FnMut` serve onde se pede `FnOnce`**:

![Três retângulos aninhados. O maior, verde, é FnOnce: call_once(self), toda closure está aqui; só nele ficam as que movem algo para fora, como || nome; pedem unwrap_or_else e thread::spawn. Dentro, vermelho, FnMut: call_mut(&mut self), só nele as que alteram o capturado, como || n += 1; pedem map, filter, retain, sort_by_key e for_each. No centro, roxo, Fn: call(&self), só lê, como |x| x + k. À direita, duas notas: quem recebe uma closure pede o conjunto maior que funcionar; quem escreve tenta ficar no menor, Fn.](assets/fn-traits.svg)

## Que bound pedir: dois pontos de vista

```rust
fn uma_vez<F: FnOnce() -> String>(f: F) -> String {
    f()
}

fn duas_vezes<F: FnMut()>(mut f: F) {
    f();
    f();
}

fn em_qualquer_lugar<F: Fn(i32) -> i32>(f: F) -> i32 {
    f(1) + f(2)
}

fn main() {
    let nome = String::from("Ana");
    println!("{}", uma_vez(|| nome)); // move `nome` para fora: só FnOnce

    let mut cliques = 0;
    duas_vezes(|| cliques += 1); // altera: FnMut
    println!("{cliques}");

    let k = 10;
    println!("{}", em_qualquer_lugar(|x| x * k)); // só lê: Fn
}
```

```text
Ana
2
30
```

Repare no `mut f` em `duas_vezes`: chamar uma `FnMut` exige `&mut`, como o E0596 da lição anterior.

**Quem recebe** uma closure deve pedir o bound **mais permissivo** que dá conta do trabalho, porque assim aceita mais closures:

- vai chamar uma vez só? Peça `FnOnce`. É o caso de `Option::unwrap_or_else`, que só chama a closure se for `None`;
- vai chamar várias vezes, uma de cada vez? Peça `FnMut`. É o caso de `map`, `filter`, `retain`, `sort_by_key` e `for_each`;
- só peça `Fn` se precisar chamar através de `&self` ou de várias threads ao mesmo tempo.

**Quem escreve** a closure quer o contrário: que ela seja `Fn`, porque aí serve em qualquer lugar. Uma closure que só lê passa em todos os bounds.

## Quando a closure não serve: E0525 e E0507

Uma closure que move um capturado para fora é só `FnOnce`. Passá-la para quem pede `Fn` dá um erro, e o código do erro depende de **onde** a closure foi escrita. Criada antes da chamada:

```rust
// ❌ NÃO COMPILA
fn chamar_duas_vezes<F: Fn()>(f: F) {
    f();
    f();
}

fn main() {
    let nome = String::from("Ana");
    let c = move || {
        let dono = nome;
        println!("{dono}");
    };
    chamar_duas_vezes(c);
}
```

```text
error[E0525]: expected a closure that implements the `Fn` trait, but this closure only implements `FnOnce`
  --> src/main.rs:8:13
   |
 8 |     let c = move || {
   |             ^^^^^^^ this closure implements `FnOnce`, not `Fn`
 9 |         let dono = nome;
   |                    ---- closure is `FnOnce` because it moves the variable `nome` out of its environment
...
12 |     chamar_duas_vezes(c);
   |     ----------------- - the requirement to implement `Fn` derives from here
   |     |
   |     required by a bound introduced by this call
   |
note: required by a bound in `chamar_duas_vezes`
  --> src/main.rs:1:25
   |
 1 | fn chamar_duas_vezes<F: Fn()>(f: F) {
   |                         ^^^^ required by this bound in `chamar_duas_vezes`
```

A mensagem conta a história inteira: a closure é `FnOnce` por causa da linha 9, e a exigência de `Fn` vem do bound da linha 1. Agora a **mesma** closure, escrita direto na chamada:

```rust
// ❌ NÃO COMPILA
fn chamar_duas_vezes<F: Fn()>(f: F) {
    f();
    f();
}

fn main() {
    let nome = String::from("Ana");
    chamar_duas_vezes(|| {
        let dono = nome;
        println!("{dono}");
    });
}
```

```text
error[E0507]: cannot move out of `nome`, a captured variable in an `Fn` closure
 --> src/main.rs:9:20
  |
7 |     let nome = String::from("Ana");
  |         ---- captured outer variable
8 |     chamar_duas_vezes(|| {
  |                       -- captured by this `Fn` closure
9 |         let dono = nome;
  |                    ^^^^ move occurs because `nome` has type `String`, which does not implement the `Copy` trait
  |
help: `Fn` and `FnMut` closures require captured values to be able to be consumed multiple times, but `FnOnce` closures may consume them only once
 --> src/main.rs:1:25
  |
1 | fn chamar_duas_vezes<F: Fn()>(f: F) {
  |                         ^^^^
help: consider borrowing here
  |
9 |         let dono = &nome;
  |                    +
...
```

É o mesmo problema visto do outro lado. Quando a closure nasce dentro da chamada, o compilador já sabe que ela precisa ser `Fn` e trata o corpo como o de uma `Fn`. Aí o erro deixa de ser "a closure não é `Fn`" e vira "você não pode mover para fora numa `Fn`" (o E0507 do [Módulo 02](../02-ownership/02-regras-de-ownership-e-move.md), de mover para fora de um empréstimo). Muito material, inclusive o Book, mostra só o E0525. Se aparecer o E0507 com "a captured variable in an `Fn` closure" (ou "`FnMut` closure"), é o mesmo caso.

As saídas são as do `help:`: emprestar em vez de mover (`&nome`), clonar dentro da closure (`nome.clone()`), ou, se a função de fato chama uma vez só, relaxar o bound para `FnOnce`.

### Chamar duas vezes uma `FnOnce`

```rust
// ❌ NÃO COMPILA
fn main() {
    let nome = String::from("Ana");
    let consome = move || nome;
    let a = consome();
    let b = consome();
    println!("{a} {b}");
}
```

```text
error[E0382]: use of moved value: `consome`
 --> src/main.rs:5:13
  |
4 |     let a = consome();
  |             --------- `consome` moved due to this call
5 |     let b = consome();
  |             ^^^^^^^ value used here after move
  |
note: closure cannot be invoked more than once because it moves the variable `nome` out of its environment
 --> src/main.rs:3:27
  |
3 |     let consome = move || nome;
  |                           ^^^^
note: this value implements `FnOnce`, which causes it to be moved when called
```

`call_once(self)` consome a closure: depois da primeira chamada, ela não existe mais. É um *use after move* comum, e as notas apontam a causa.

### Emprestar a closure: `&mut F` também é `FnMut`

Passar uma closure para `for_each` (que recebe `F` por valor) a consumiria. Se você quer continuar usando a closure depois, passe uma referência para ela: `&mut F` implementa `FnMut` quando `F` implementa, e `&F` implementa `Fn`:

```rust
let mut total = 0;
let mut acumula = |x: i32| total += x;
[1, 2, 3].into_iter().for_each(&mut acumula); // empresta a closure
acumula(10); // e ela continua aqui
println!("{total}");
```

```text
16
```

## Ponteiros `fn`

O [kit do Módulo 01](../01-tour-da-sintaxe/08-kit-closures.md) mostrou que dá para passar o nome de uma função no lugar de uma closure (`.map(dobro)`). Funções também têm um tipo que você pode escrever: `fn(i32) -> i32`, um ponteiro de função, como o `int (*)(int)` do C. Closures **que não capturam nada** também viram ponteiro `fn`:

```rust
// ❌ NÃO COMPILA
fn aplicar(f: fn(i32) -> i32, x: i32) -> i32 {
    f(x)
}

fn dobro(x: i32) -> i32 {
    x * 2
}

fn main() {
    println!("{}", aplicar(dobro, 4));
    println!("{}", aplicar(|x| x + 1, 4));
    let fator = 3;
    println!("{}", aplicar(|x| x * fator, 4));
}
```

```text
error[E0308]: mismatched types
  --> src/main.rs:13:28
   |
13 |     println!("{}", aplicar(|x| x * fator, 4));
   |                    ------- ^^^^^^^^^^^^^ expected fn pointer, found closure
   |                    |
   |                    arguments to this function are incorrect
   |
   = note: expected fn pointer `fn(i32) -> i32`
                 found closure `{closure@src/main.rs:13:28: 13:31}`
note: closures can only be coerced to `fn` types if they do not capture any variables
  --> src/main.rs:13:36
   |
13 |     println!("{}", aplicar(|x| x * fator, 4));
   |                                    ^^^^^ `fator` captured here
```

As duas primeiras chamadas compilam. A terceira não: uma closure que captura `fator` precisa de um lugar para guardá-lo, e um ponteiro de função é só um endereço de código. Por isso, num parâmetro, prefira `F: Fn(i32) -> i32` a `fn(i32) -> i32`: o bound aceita funções, closures sem captura **e** closures com captura. Ponteiros `fn` fazem sentido em tabelas fixas de funções e na fronteira com C (Módulo 15).

Isso também explica um detalhe: um `if`/`else` que escolhe entre duas closures **sem captura** compila, porque as duas viram o mesmo tipo `fn(i32) -> i32`:

```rust
let dobrar = std::env::args().count() > 1;
let f = if dobrar { |x: i32| x * 2 } else { |x: i32| x + 1 };
println!("{}", f(10));
```

Com captura, não compila, como você vai ver logo abaixo.

## Guardar uma closure numa struct

Como uma closure não tem nome de tipo, a struct precisa de um parâmetro genérico com bound:

```rust
struct Filtro<P> {
    pred: P,
}

impl<P: Fn(i32) -> bool> Filtro<P> {
    fn aplicar(&self, v: &[i32]) -> Vec<i32> {
        let mut saida = Vec::new();
        for &x in v {
            if (self.pred)(x) {
                saida.push(x);
            }
        }
        saida
    }
}

fn main() {
    let limite = 5;
    let f = Filtro { pred: |x| x > limite };
    println!("{:?}", f.aplicar(&[3, 8, 5, 10]));
}
```

```text
[8, 10]
```

Dois detalhes. Para chamar a closure guardada num campo, use parênteses: `(self.pred)(x)`. Sem eles, `self.pred(x)` procura um **método** chamado `pred`. E o bound é `Fn`, e não `FnMut`, porque `aplicar` recebe `&self`: com `&self` não dá para chamar uma `FnMut`.

Escrever a trait direto como tipo do campo não funciona:

```rust
// ❌ NÃO COMPILA
struct Filtro {
    pred: Fn(i32) -> bool,
}
```

```text
error[E0782]: expected a type, found a trait
 --> src/main.rs:2:11
  |
2 |     pred: Fn(i32) -> bool,
  |           ^^^^^^^^^^^^^^^
  |
help: you can add the `dyn` keyword if you want a trait object
  |
2 |     pred: dyn Fn(i32) -> bool,
  |           +++
```

`Fn(i32) -> bool` é uma trait, não um tipo. Seguir o `help:` à risca também não resolve: `dyn Fn(...)` não tem tamanho conhecido, e você não consegue criar a struct. Há duas soluções de verdade: o genérico `P` acima, ou `Box<dyn Fn(i32) -> bool>`, um *trait object* no heap (o [Módulo 06](../06-traits-e-generics/README.md) explica `dyn`). Use o genérico quando a struct guarda uma closure só, conhecida na criação. Use `Box<dyn Fn>` quando precisa misturar closures diferentes no mesmo lugar, como numa lista.

## Devolver uma closure

`-> impl Fn(i32) -> i32` diz "devolvo algum tipo que implementa `Fn(i32) -> i32`, sem dizer qual". Funciona porque cada função devolve **um** tipo concreto, ainda que sem nome. O `move` é obrigatório se a closure captura algo local (o E0373 da lição anterior). O problema aparece quando há dois caminhos:

```rust
// ❌ NÃO COMPILA
fn ajuste(soma: bool, k: i32) -> impl Fn(i32) -> i32 {
    if soma {
        move |x| x + k
    } else {
        move |x| x * k
    }
}
```

```text
error[E0308]: `if` and `else` have incompatible types
 --> src/main.rs:5:9
  |
2 | /     if soma {
3 | |         move |x| x + k
  | |         --------------
  | |         |
  | |         the expected closure
  | |         expected because of this
4 | |     } else {
5 | |         move |x| x * k
  | |         ^^^^^^^^^^^^^^ expected closure, found a different closure
6 | |     }
  | |_____- `if` and `else` have incompatible types
  |
  = note: expected closure `{closure@src/main.rs:3:9: 3:17}`
             found closure `{closure@src/main.rs:5:9: 5:17}`
  = note: no two closures, even if identical, have the same type
  = help: consider boxing your closure and/or using it as a trait object
help: you could change the return type to be a boxed trait object
  |
1 - fn ajuste(soma: bool, k: i32) -> impl Fn(i32) -> i32 {
1 + fn ajuste(soma: bool, k: i32) -> Box<dyn Fn(i32) -> i32> {
...
```

"*No two closures, even if identical, have the same type*": são duas structs diferentes, e `impl Fn` precisa de uma só. (Sem o `k` capturado, as duas virariam ponteiros `fn` e o código compilaria, como você viu acima. Por isso o exemplo captura.) O `help:` dá a saída: devolver um `Box<dyn Fn>`, que apaga o tipo concreto atrás de um ponteiro. Com isso, closures diferentes cabem até no mesmo `Vec`:

```rust
// ✅
fn ajuste(soma: bool, k: i32) -> Box<dyn Fn(i32) -> i32> {
    if soma {
        Box::new(move |x| x + k)
    } else {
        Box::new(move |x| x * k)
    }
}

fn main() {
    let etapas: Vec<Box<dyn Fn(i32) -> i32>> = vec![ajuste(true, 3), ajuste(false, 2), Box::new(|x| x - 1)];
    let mut v = 10;
    for f in &etapas {
        v = f(v);
    }
    println!("{} {v}", ajuste(true, 3)(10));
}
```

```text
13 25
```

### O mapa do C++

| C++ | Rust | Custo |
|---|---|---|
| parâmetro de template `F` (`template <class F> void f(F g)`) | `F: Fn(A) -> R` ou `impl Fn(A) -> R` | estático, monomorfizado, sem indireção |
| `std::function<R(A)>` | `Box<dyn Fn(A) -> R>` | chamada indireta e, em geral, uma alocação |
| ponteiro de função `R (*)(A)` | `fn(A) -> R` | indireto, sem captura |
| lambda com `mutable` | closure `FnMut` + `let mut` | |

Uma diferença: `std::function` exige que a lambda guardada seja copiável. `Box<dyn Fn>` não exige nada disso, só pode ser movido (o mais parecido em C++ é o `std::move_only_function` do C++23). E não há em C++ um equivalente direto a `Box<dyn FnOnce>`, um "chamável uma vez só" verificado pelo compilador.

Do lado do Python, toda função é um objeto que você guarda numa lista sem pensar em tipo. Em Rust, você escolhe entre genérico (rápido, um tipo só) e `Box<dyn Fn>` (flexível, com indireção). E uma função aninhada que usa `nonlocal` para alterar uma variável de fora é o equivalente de uma `FnMut`.

> [!NOTE]
> Desde a 1.85 existem *async closures* (`async || ...`) e as traits `AsyncFn`, `AsyncFnMut` e `AsyncFnOnce`. Chamar uma delas devolve uma *future*, que não faz nada até ser executada. Elas aparecem no Módulo 12 (programação assíncrona).

Código completo em [`codigo/closures-na-pratica`](codigo/closures-na-pratica/src/lib.rs): uma função com bound `FnMut` (tentar de novo), uma com `FnOnce` (valor padrão preguiçoso), uma fábrica de contadores (`impl FnMut`), composição de funções e um validador com `Vec<Box<dyn Fn>>`, todos com testes.

## Armadilhas comuns

- **Bound restrito demais**: pedir `Fn` numa função que chama a closure uma vez só recusa closures que movem. Peça `FnOnce`.
- **E0525 ou E0507 "in an `Fn` closure"**: a closure move algo para fora e o bound pede `Fn`/`FnMut`. Empreste, clone ou relaxe o bound.
- **`self.pred(x)`** em vez de `(self.pred)(x)`: o compilador procura um método chamado `pred` (E0599).
- **Campo do tipo `Fn(...)`** (E0782): use um genérico `P: Fn(...)` ou `Box<dyn Fn(...)>`.
- **Dois caminhos devolvendo closures diferentes com `impl Fn`** (E0308): use `Box<dyn Fn>`.
- **Parâmetro `fn(...)` recusando closure com captura** (E0308): troque por um bound `Fn`.

## Resumo

- `FnOnce` (`self`), `FnMut` (`&mut self`) e `Fn` (`&self`). O corpo decide: mover para fora → só `FnOnce`; alterar → `FnMut`; só ler → `Fn`.
- Toda `Fn` é `FnMut`, e toda `FnMut` é `FnOnce`. Quem recebe pede o bound mais permissivo; quem escreve tenta ficar em `Fn`.
- O mesmo erro aparece como E0525 (closure criada antes) ou E0507 (closure escrita na chamada).
- `fn(A) -> R` é um ponteiro de função; só closures sem captura viram um. Prefira bounds `Fn*`.
- Para guardar: genérico `P: Fn(...)` ou `Box<dyn Fn(...)>`. Para devolver: `impl Fn` com `move`, ou `Box<dyn Fn>` quando há mais de um caminho.

## Exercícios

Veja os exercícios 1 e 5 em [`exercicios.md`](exercicios.md).

## Leitura complementar

- [The Rust Programming Language, 13.1 (seção "Moving Captured Values Out of Closures and the `Fn` Traits")](https://doc.rust-lang.org/stable/book/ch13-01-closures.html#moving-captured-values-out-of-closures-and-the-fn-traits)
- [The Rust Programming Language, 20.4 Advanced Functions and Closures](https://doc.rust-lang.org/stable/book/ch20-04-advanced-functions-and-closures.html) (ponteiros `fn` e devolver closures)
- Documentação de [`Fn`](https://doc.rust-lang.org/std/ops/trait.Fn.html), [`FnMut`](https://doc.rust-lang.org/std/ops/trait.FnMut.html) e [`FnOnce`](https://doc.rust-lang.org/std/ops/trait.FnOnce.html)
- [Effective Rust, Item 2: Use the type system to express common behavior](https://effective-rust.com/use-types-2.html) ("use the most general `Fn*` trait that works")
- [Comprehensive Rust: Closure traits](https://google.github.io/comprehensive-rust/closures/traits.html)

---

[← Lição anterior](04-closures-e-captura.md) · [Índice do módulo](README.md) · [Próxima lição →](06-iterator-e-intoiterator.md)

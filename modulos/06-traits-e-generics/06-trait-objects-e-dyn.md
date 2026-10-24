# 06 — Trait objects e `dyn`

**Objetivos.** Ao final, você vai saber:

- guardar valores de tipos diferentes numa mesma coleção com `Box<dyn Trait>`;
- como um *trait object* fica na memória (*fat pointer* e *vtable*), e o que isso custa;
- por que `Box<dyn Error>` quer dizer `Box<dyn Error + 'static>`, o detalhe que o Módulo 05 deixou pendente;
- o que torna uma trait *dyn compatible*, e como contornar quando ela não é;
- converter `&dyn Sub` em `&dyn Super` (*trait upcasting*) e usar `Any` para descobrir o tipo concreto.

## O problema: uma lista de formas diferentes

Com generics, `fn area_total<T: Forma>(formas: &[T])` aceita um slice de quadrados ou um slice de círculos. Mas não um slice com **os dois**: `T` é um tipo só, escolhido na compilação. Para uma lista que mistura tipos, você precisa de outra ferramenta:

```rust
trait Forma {
    fn area(&self) -> f64;
}

let formas: Vec<Box<dyn Forma>> = vec![
    Box::new(Quadrado { lado: 2.0 }),
    Box::new(Circulo { raio: 1.0 }),
];
for f in &formas {
    println!("{:<20} área {:.2}", f.nome(), f.area());
}
```

```text
quadrado de lado 2   área 4.00
círculo de raio 1    área 3.14
```

`dyn Forma` é um *trait object*: "um valor de algum tipo que implementa `Forma`, decidido em tempo de execução". O `Box` (Módulo 10; por ora, "um ponteiro dono para um valor no heap") é necessário porque cada elemento pode ter um tamanho diferente, e o `Vec` precisa de elementos do mesmo tamanho. O código completo, com os outros exemplos desta lição, está em [`codigo/formas`](codigo/formas/).

Este é o papel do `virtual` do C++ e do *duck typing* do Python:

```cpp
struct Forma {
    virtual double area() const = 0;
    virtual ~Forma() = default;
};
std::vector<std::unique_ptr<Forma>> formas;
```

```python
formas = [Quadrado(2.0), Circulo(1.0)]  # qualquer objeto com .area()
```

A diferença para o Python é a de sempre: só entra no `Vec` quem implementa `Forma`, e o compilador confere. Tentar colocar uma `String` lá:

```text
error[E0277]: the trait bound `String: Forma` is not satisfied
  --> src/main.rs:18:9
   |
18 |         Box::new(String::from("triângulo")),
   |         ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ the trait `Forma` is not implemented for `String`
   |
help: the trait `Forma` is implemented for `Quadrado`
...
   = note: required for the cast from `Box<String>` to `Box<dyn Forma>`
```

A última nota mostra o que acontece em cada `Box::new(...)`: um `Box<Quadrado>` é convertido (*coerced*) em `Box<dyn Forma>`. Essa conversão só existe se o tipo implementa a trait.

## Por dentro: *fat pointer* e *vtable*

Um `&dyn Forma` (ou `Box<dyn Forma>`) não é um ponteiro comum. É um *fat pointer*, com duas partes:

1. o ponteiro para os **dados** (o `Quadrado` em si);
2. o ponteiro para a **vtable** do tipo concreto: uma tabela, gerada pelo compilador e guardada na memória estática do programa, com os endereços dos métodos daquele tipo para aquela trait, mais o tamanho, o alinhamento e a função que destrói o valor.

Os tamanhos confirmam (máquina de 64 bits):

```rust
println!(
    "&Quadrado: {} bytes, &dyn Forma: {} bytes, Box<dyn Forma>: {} bytes, &[u8]: {} bytes",
    size_of::<&Quadrado>(),
    size_of::<&dyn Forma>(),
    size_of::<Box<dyn Forma>>(),
    size_of::<&[u8]>()
);
```

```text
&Quadrado: 8 bytes, &dyn Forma: 16 bytes, Box<dyn Forma>: 16 bytes, &[u8]: 16 bytes
```

Você já conhecia um fat pointer: a slice do Módulo 03, que carrega o ponteiro e o tamanho. O trait object carrega o ponteiro e a vtable.

![Na stack, o Vec formas aponta para um buffer no heap com dois fat pointers, cada um com um ponteiro de dados e um ponteiro de vtable. Os ponteiros de dados levam a um Quadrado e a um Circulo no heap, que guardam só os seus campos. Os ponteiros de vtable levam a duas tabelas na memória estática, uma por tipo, com drop, tamanho, alinhamento e os métodos area, nome e fmt. Embaixo, a comparação: em C++ o vptr fica dentro de cada objeto; em Rust a vtable fica no ponteiro.](assets/fat-pointer-vtable.svg)

Uma chamada `f.area()` com `f: &dyn Forma` lê a vtable no fat pointer, pega a entrada de `area` e chama essa função passando o ponteiro de dados como `&self`. É o mesmo mecanismo do `virtual` do C++, com uma diferença de lugar:

- **Em C++**, o ponteiro para a vtable (o *vptr*) fica **dentro do objeto**. Toda instância de uma classe com métodos virtuais carrega esses 8 bytes extras, quer você use polimorfismo ou não.
- **Em Rust**, a vtable vai **no ponteiro**, e só quando você escreve `dyn`. O `Quadrado` continua com 8 bytes, o mesmo de sempre, e o mesmo tipo pode ser usado com generics (sem vtable nenhuma) em outra parte do programa.

O layout exato da vtable não é garantido pela linguagem; o desenho mostra a ideia.

### O custo

Cada chamada por `dyn` é uma indireção: ler a vtable e pular para um endereço que só se conhece em tempo de execução. Isso também impede o compilador de fazer *inlining*. O custo é pequeno, raramente aparece num perfil, e é o mesmo de um método virtual em C++ ou de qualquer chamada em Python. O que se ganha: uma cópia só da função (a [lição 01](01-generics.md) mostrou isso com o `nm`) e a liberdade de misturar tipos.

## `dyn` precisa de um ponteiro

`dyn Forma` sozinho é um tipo sem tamanho conhecido: pode ser um `Quadrado` de 8 bytes ou outra coisa de 200. Tipos assim se chamam *dynamically sized types* (DSTs), como `str` e `[T]`, e só podem ser usados atrás de um ponteiro: `&dyn Forma`, `&mut dyn Forma`, `Box<dyn Forma>`, e os `Rc`/`Arc` do Módulo 10. Sem ponteiro:

```rust
// ❌ NÃO COMPILA
use std::fmt::Display;

fn main() {
    let itens: Vec<dyn Display> = Vec::new();
}
```

```text
error[E0277]: the size for values of type `dyn std::fmt::Display` cannot be known at compilation time
   --> src/main.rs:4:16
    |
  4 |     let itens: Vec<dyn Display> = Vec::new();
    |                ^^^^^^^^^^^^^^^^ doesn't have a size known at compile-time
    |
    = help: the trait `Sized` is not implemented for `dyn std::fmt::Display`
note: required by an implicit `Sized` bound in `Vec`
...
```

A mensagem cita uma trait que você ainda não viu: `Sized`, a marcadora de "tem tamanho conhecido na compilação". Todo parâmetro genérico tem um bound `T: Sized` **implícito**, e é por isso que `Vec<dyn Display>` não serve. Quando uma função genérica aceita também DSTs, ela relaxa o bound com `?Sized` ("talvez não `Sized`"), a única trait que se escreve com `?`. Você já viu isso no blanket impl de `ToString` (lição 03): `impl<T: Display + ?Sized> ToString for T`, que é o que faz `str` ter `to_string()`. É só para ler assinaturas; raramente você vai escrever `?Sized`.

O mesmo vale para parâmetros: `fn mostrar(x: dyn Display)` não compila, e o compilador sugere as duas saídas, `impl Display` (genérico) ou `&dyn Display` (trait object).

> [!WARNING]
> Na edition 2024, esquecer o `dyn` é erro: `Box<Display>` dá `error[E0782]: expected a type, found a trait`, com a sugestão de acrescentar `dyn`. Em código antigo (edition 2015), você pode encontrar `Box<Display>` sem `dyn`; é a mesma coisa, escrita do jeito velho.

## O `'static` escondido em `Box<dyn Trait>`

No Módulo 05, a assinatura de `source()` tinha um detalhe que ficou para depois:

```rust
fn source(&self) -> Option<&(dyn Error + 'static)>
```

Um trait object esconde o tipo concreto, e esse tipo pode conter referências. `Box<dyn Display>` poderia guardar um `&str` emprestado de uma variável local, por exemplo. Para o borrow checker continuar funcionando, todo trait object tem um *lifetime bound*: `dyn Trait + 'a` quer dizer "o valor escondido não contém referências que vivam menos que `'a`".

Quando você não escreve o bound, o compilador escolhe um padrão (o *default object lifetime bound*):

- em `Box<dyn Trait>`, o padrão é `'static`: `Box<dyn Display>` é `Box<dyn Display + 'static>`;
- em `&'a dyn Trait`, o padrão é o lifetime da própria referência: `&'a (dyn Trait + 'a)`.

`'static` aqui **não** quer dizer "vive o programa inteiro". Quer dizer "não contém referências emprestadas" (ou só referências `'static`), como no Módulo 03. Uma `String`, um `i32` e um `ConfigError` com um `io::Error` dentro são `'static`; um `&str` emprestado de uma variável local não é. Por isso esta função não compila:

```rust
// ❌ NÃO COMPILA
use std::fmt::Display;

fn embrulhar(s: &str) -> Box<dyn Display> {
    Box::new(s)
}
```

```text
error: lifetime may not live long enough
 --> src/main.rs:4:5
  |
3 | fn embrulhar(s: &str) -> Box<dyn Display> {
  |                 - let's call the lifetime of this reference `'1`
4 |     Box::new(s)
  |     ^^^^^^^^^^^ returning this value requires that `'1` must outlive `'static`
  |
help: to declare that the trait object captures data from argument `s`, you can add an explicit `'_` lifetime bound
  |
3 | fn embrulhar(s: &str) -> Box<dyn Display + '_> {
  |                                          ++++
```

O `Box<dyn Display>` prometeu "nada emprestado", e `s` é emprestado. As saídas: aceitar a sugestão (`+ '_`, "a caixa vive no máximo tanto quanto `s`"), ou guardar um valor próprio (`Box::new(s.to_string())`).

Agora o `source()` faz sentido. `Option<&(dyn Error + 'static)>` é "talvez uma referência para um erro que **não contém empréstimos**". Os erros da cadeia precisam ser assim para poderem ser guardados, passados adiante, levados para outra thread e, principalmente, para funcionar o `downcast_ref` que você usou no Módulo 05: descobrir o tipo concreto exige `'static`, como mostra a seção sobre `Any`, mais abaixo. E o `Box<dyn Error>` da função `main` é `Box<dyn Error + 'static>` pelo mesmo motivo.

## *Dyn compatibility*: nem toda trait vira `dyn`

Para existir `dyn Trait`, o compilador precisa montar uma vtable: uma entrada por método, cada uma uma função concreta que recebe o ponteiro de dados. Algumas traits não permitem isso. Elas não são *dyn compatible* (o nome antigo, que você vai encontrar em material mais velho, é *object safety*). Três casos comuns, cada um com o motivo exato no erro.

**Um método devolve `Self`.**

```rust
// ❌ NÃO COMPILA
trait Forma {
    fn area(&self) -> f64;
    fn duplicar(&self) -> Self;
}

// ...
let formas: Vec<Box<dyn Forma>> = vec![Box::new(Quadrado(2.0))];
```

```text
error[E0038]: the trait `Forma` is not dyn compatible
  --> src/main.rs:14:29
   |
14 |     let formas: Vec<Box<dyn Forma>> = vec![Box::new(Quadrado(2.0))];
   |                             ^^^^^ `Forma` is not dyn compatible
   |
note: for a trait to be dyn compatible it needs to allow building a vtable
      for more information, visit <https://doc.rust-lang.org/reference/items/traits.html#dyn-compatibility>
  --> src/main.rs:3:27
   |
 1 | trait Forma {
   |       ----- this trait is not dyn compatible...
 2 |     fn area(&self) -> f64;
 3 |     fn duplicar(&self) -> Self;
   |                           ^^^^ ...because method `duplicar` references the `Self` type in its return type
   = help: consider moving `duplicar` to another trait
   = help: only type `Quadrado` implements `Forma`; consider using it directly instead.
```

Por quê: chamado num `dyn Forma`, `duplicar` devolveria um valor de tipo e tamanho desconhecidos. Não há como reservar espaço para ele.

**A trait exige `Clone`** (ou qualquer coisa que implique `Sized`):

```text
error[E0038]: the trait `Forma` is not dyn compatible
...
 1 | trait Forma: Clone {
   |       -----  ^^^^^ ...because it requires `Self: Sized`
```

`clone()` devolve `Self`, o mesmo problema. Por isso não existe `Box<dyn Forma>` clonável com uma supertrait `Clone`. (Há crates que resolvem isso, como `dyn-clone`, com um método que devolve `Box<dyn Forma>`.)

**Um método é genérico.**

```text
error[E0038]: the trait `Registro` is not dyn compatible
...
 2 |     fn registrar<T: std::fmt::Display>(&self, valor: T);
   |        ^^^^^^^^^ ...because method `registrar` has generic type parameters
```

Um método genérico tem uma versão para cada `T` (monomorfização), potencialmente infinitas. Não cabe numa tabela de tamanho fixo. A correção costuma ser trocar o genérico por um trait object: `fn registrar(&self, valor: &dyn Display)`.

A intuição que resume a lista: **se não dá para pôr numa tabela de funções, a trait não vira `dyn`**. A [Reference](https://doc.rust-lang.org/reference/items/traits.html#dyn-compatibility) tem a lista completa (também ficam de fora, por exemplo, `async fn` e métodos que devolvem `impl Trait`). Tipos associados, como o `Item` de `Iterator`, são permitidos: `dyn Iterator<Item = u32>` existe, e você só precisa dizer o tipo.

### A saída: `where Self: Sized`

Se o método problemático não é necessário via `dyn`, você pode excluí-lo da vtable com `where Self: Sized`. A trait volta a ser dyn compatible, e o método continua existindo para os tipos concretos:

```rust
trait Forma {
    fn area(&self) -> f64;
    fn duplicar(&self) -> Self
    where
        Self: Sized;
}
```

`Box<dyn Forma>` passa a compilar. Só a chamada de `duplicar` num trait object é recusada:

```text
error: the `duplicar` method cannot be invoked on a trait object
  --> src/main.rs:18:20
   |
 5 |         Self: Sized;
   |               ----- this has a `Sized` requirement
...
18 |     let _copia = f.duplicar();
   |                    ^^^^^^^^
```

Num `Quadrado` concreto, `q.duplicar()` funciona normalmente. A outra saída, que o próprio `help:` sugere, é mover o método para uma segunda trait. O exercício 5 pede as duas.

## Trait upcasting e `Any`

Desde o Rust 1.86, se `Sub` tem `Super` como supertrait, um `&dyn Sub` converte para `&dyn Super` (o mesmo vale para `Box`, `Rc` e `Arc`). Isso se chama *trait upcasting*. Com `Debug` como supertrait de `Forma`:

```rust
trait Forma: Debug + Any {
    fn area(&self) -> f64;
}

fn imprimir_debug(x: &dyn Debug) {
    println!("debug: {x:?}");
}

let primeira: &dyn Forma = &*formas[0]; // &Box<dyn Forma> -> &dyn Forma
imprimir_debug(primeira);              // &dyn Forma -> &dyn Debug
```

```text
debug: Quadrado { lado: 2.0 }
```

O caso mais útil é `Any`, a trait de `std::any` que todo tipo `'static` implementa sozinho. Um `&dyn Any` sabe responder qual é o seu tipo concreto, e `downcast_ref::<T>()` devolve `Some(&T)` se acertar:

```rust
for f in &formas {
    let qualquer: &dyn Any = &**f;
    match qualquer.downcast_ref::<Circulo>() {
        Some(c) => println!("é um Circulo, raio {}", c.raio),
        None => println!("não é um Circulo"),
    }
}
```

```text
não é um Circulo
é um Circulo, raio 1
```

É o equivalente do `dynamic_cast` do C++ e do `isinstance` do Python. E é o mecanismo por trás do `downcast_ref` de `Box<dyn Error>` do Módulo 05. Duas restrições:

- **Só para tipos `'static`.** A identidade do tipo (`TypeId`) não distingue lifetimes, então um tipo com referências emprestadas não pode entrar num `dyn Any`. Daí o `'static` em `dyn Error + 'static`.
- **Não é reflexão.** `Any` só sabe responder "você é exatamente o tipo `T`?". Não lista campos nem métodos.

> [!NOTE]
> Antes da 1.86, o upcasting não existia, e o truque era acrescentar um método `fn as_any(&self) -> &dyn Any { self }` em cada implementação. Você vai encontrar isso em muito código e material antigo, inclusive em livros que dizem que o upcasting é impossível. Hoje basta pôr `Any` como supertrait.

Use `downcast` como último recurso. Se você está perguntando "qual é o tipo?" para decidir o que fazer, provavelmente o comportamento devia estar num método da trait, ou o conjunto de tipos é fechado e devia ser um `enum` (próxima lição).

## Armadilhas comuns

**`Vec<dyn Trait>` ou `fn f(x: dyn Trait)`.** `dyn Trait` não tem tamanho; precisa de ponteiro: `Vec<Box<dyn Trait>>`, `&dyn Trait`. O E0277 com `Sized` é o sintoma.

**Devolver um trait object com empréstimo dentro.** `Box<dyn Trait>` é `'static` por padrão. Se o valor guarda uma referência, escreva `Box<dyn Trait + '_>` ou guarde um valor próprio.

**Trait com `Clone` ou `-> Self` usada como `dyn`.** E0038. Use `where Self: Sized` no método ou separe em duas traits.

**`dyn` por reflexo.** Quem vem de Java ou de C++ com herança tende a escrever `Box<dyn Trait>` em todo lugar. Se a coleção é de um tipo só, ou se o tipo é conhecido na compilação, generics são mais simples e mais rápidos. A próxima lição dá o critério.

## Resumo

- `dyn Trait` é um valor de algum tipo que implementa a trait, decidido em tempo de execução. Fica sempre atrás de um ponteiro (`&`, `Box`, `Rc`, `Arc`).
- `&dyn Trait` e `Box<dyn Trait>` são fat pointers: dados + vtable (16 bytes). Diferente do C++, o objeto não carrega vptr.
- Custo: uma indireção por chamada e nada de inlining. Ganho: uma cópia só do código e coleções heterogêneas.
- `Box<dyn Trait>` = `Box<dyn Trait + 'static>`: o valor escondido não pode ter empréstimos. É o `'static` de `dyn Error + 'static`.
- Uma trait é dyn compatible se cabe numa vtable: nada de `-> Self`, métodos genéricos ou `Self: Sized`. `where Self: Sized` exclui um método da vtable.
- Desde a 1.86, `&dyn Sub` vira `&dyn Super` (upcasting). Com `Any` como supertrait, `downcast_ref` recupera o tipo concreto.

## Exercícios

Veja os exercícios 2 e 5 em [`exercicios.md`](exercicios.md).

## Leitura complementar

- [The Rust Programming Language, 18.2 Using Trait Objects to Abstract over Shared Behavior](https://doc.rust-lang.org/stable/book/ch18-02-trait-objects.html)
- [The Rust Reference: Dyn compatibility](https://doc.rust-lang.org/reference/items/traits.html#dyn-compatibility) e [Default trait object lifetimes](https://doc.rust-lang.org/reference/lifetime-elision.html#default-trait-object-lifetimes)
- [Anúncio do Rust 1.86 (trait upcasting)](https://blog.rust-lang.org/2025/04/03/Rust-1.86.0/)
- [Documentação de `std::any`](https://doc.rust-lang.org/std/any/index.html)
- [Comprehensive Rust: dyn Trait](https://google.github.io/comprehensive-rust/generics/dyn-trait.html)
- [Effective Rust, Item 12](https://effective-rust.com/generics.html) (atenção: diz que não existe upcasting, o que deixou de valer na 1.86)

---

[← Lição anterior](05-conversoes-from-into-tryfrom.md) · [Índice do módulo](README.md) · [Próxima lição →](07-estatico-x-dinamico-e-composicao.md)

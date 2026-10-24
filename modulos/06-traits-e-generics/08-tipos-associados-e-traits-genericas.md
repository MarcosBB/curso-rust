# 08 — Tipos associados e traits genéricas

**Objetivos.** Ao final, você vai saber:

- ler e escrever traits com tipos associados (`type Item;`) e traits com parâmetros genéricos (`From<T>`);
- escolher entre os dois: "uma implementação por tipo" ou "várias";
- por que duas implementações que se sobrepõem dão E0119, mesmo com blanket impls;
- ler assinaturas como `Add<Rhs = Self>`;
- desfazer ambiguidades com a sintaxe qualificada (`Trait::metodo(&x)` e `<Tipo as Trait>::funcao()`).

## Dois jeitos de pôr um tipo numa trait

Você já viu os dois sem reparar. Em `TryFrom` (lição 05) e em `FromStr`, havia uma linha `type Error = ...;`. Em `From<T>`, o tipo vem entre `<>`. Compare `Iterator` e `From`, simplificadas:

```rust
trait Iterator {
    type Item;                                 // tipo associado
    fn next(&mut self) -> Option<Self::Item>;
}

trait From<T> {                                // parâmetro genérico
    fn from(valor: T) -> Self;
}
```

Os dois parecem "uma trait que fala de outro tipo", mas dizem coisas diferentes.

**Tipo associado: quem implementa escolhe, uma vez só.** Um contador que produz números implementa `Iterator` com `Item = u32`, e pronto: o tipo dos itens é parte do que o contador **é**.

```rust
struct Contador {
    atual: u32,
}

impl Iterator for Contador {
    type Item = u32;

    fn next(&mut self) -> Option<u32> {
        if self.atual < 3 {
            self.atual += 1;
            Some(self.atual)
        } else {
            None
        }
    }
}
```

Não faz sentido o mesmo contador ser também um iterador de `String`, e o compilador proíbe:

```rust
// ❌ NÃO COMPILA
impl Iterator for Contador {
    type Item = u32;
    fn next(&mut self) -> Option<u32> { None }
}

impl Iterator for Contador {
    type Item = String;
    fn next(&mut self) -> Option<String> { None }
}
```

```text
error[E0119]: conflicting implementations of trait `Iterator` for type `Contador`
  --> src/main.rs:10:1
   |
 5 | impl Iterator for Contador {
   | -------------------------- first implementation here
...
10 | impl Iterator for Contador {
   | ^^^^^^^^^^^^^^^^^^^^^^^^^^ conflicting implementation for `Contador`
```

A vantagem para quem usa: como só existe uma implementação, o compilador sempre sabe o `Item`. Ninguém precisa escrever `Iterator<u32>` em cada uso; `for n in contador` já sabe que `n` é `u32`. Por isso tipos associados também são chamados de tipos de **saída**: eles são consequência do tipo que implementa.

**Parâmetro genérico: quem usa escolhe, e pode haver várias implementações.** `From<f64>` e `From<i32>` são, para o compilador, duas traits diferentes, e um tipo pode implementar as duas:

```rust
struct Celsius(f64);

impl From<f64> for Celsius {
    fn from(v: f64) -> Self {
        Celsius(v)
    }
}

impl From<i32> for Celsius {
    fn from(v: i32) -> Self {
        Celsius(f64::from(v))
    }
}

let a: Celsius = 30.5.into();
let b = Celsius::from(10);
```

Aqui o tipo de origem é uma **entrada**: quem chama decide qual conversão quer, e o compilador escolhe a implementação pelo tipo do argumento.

A regra para as suas traits:

| Pergunta | Use |
|---|---|
| Para um dado tipo, só existe uma resposta? (o item de um iterador, o erro de um parse) | tipo associado |
| O mesmo tipo pode fazer isso com vários tipos diferentes? (converter de vários tipos, comparar com vários tipos) | parâmetro genérico |

Num bound, o tipo associado se fixa com `Nome = Tipo`: `fn soma(it: impl Iterator<Item = u32>) -> u32`, ou `T: Iterator<Item = String>`. Dentro da trait ou de um `impl`, ele se escreve `Self::Item`; fora, em código genérico, `T::Item`. Você vai usar isso o tempo todo no Módulo 07, em que `Iterator` é a estrela. E `IntoIterator`, a trait do `for`, tem dois tipos associados: `Item` e `IntoIter` (o tipo do iterador que ela produz).

## Coerência: nunca duas implementações para o mesmo caso

O E0119 acima é uma regra geral, a mesma ideia da *orphan rule* (lição 02): para cada par trait + tipo, existe **no máximo uma** implementação no programa inteiro. Ela vale também para blanket impls, e às vezes surpreende:

```rust
// ❌ NÃO COMPILA
use std::fmt::Display;

trait Descrever {
    fn descrever(&self) -> String;
}

impl<T: Display> Descrever for T {
    fn descrever(&self) -> String {
        format!("<{self}>")
    }
}

impl Descrever for i32 {
    fn descrever(&self) -> String {
        format!("inteiro {self}")
    }
}
```

```text
error[E0119]: conflicting implementations of trait `Descrever` for type `i32`
  --> src/main.rs:13:1
   |
 7 | impl<T: Display> Descrever for T {
   | -------------------------------- first implementation here
...
13 | impl Descrever for i32 {
   | ^^^^^^^^^^^^^^^^^^^^^^ conflicting implementation for `i32`
```

`i32` implementa `Display`, então o blanket impl já cobre `i32`. Em C++, você escreveria uma especialização do template e a mais específica ganharia. Rust estável não tem especialização: as duas implementações colidem. As saídas são escolher uma estratégia só (ou blanket, ou implementações tipo a tipo) ou usar um newtype para o caso especial.

## Parâmetros com valor padrão: `Add<Rhs = Self>`

Tipos associados e parâmetros genéricos podem aparecer juntos. A trait por trás do operador `+` é assim:

```rust
trait Add<Rhs = Self> {
    type Output;
    fn add(self, rhs: Rhs) -> Self::Output;
}
```

Dá para ler com o que você já sabe:

- `Rhs` (*right-hand side*, o lado direito do `+`) é um parâmetro genérico, porque um tipo pode somar com vários outros: um `Instante + Duracao` e um `Instante + Segundos`.
- `= Self` é um **valor padrão**: quem escreve só `Add` está dizendo `Add<Self>`, "somo com o meu próprio tipo".
- `Output` é tipo associado, porque, fixados os dois lados, o resultado é um só.

Implementar operadores para os seus tipos (`impl Add for Vetor`) é a *sobrecarga de operadores*, assunto do Módulo 13. Por ora, basta saber ler a assinatura: ela aparece na documentação de todo tipo numérico.

## Quando os nomes colidem: sintaxe qualificada

Um tipo pode implementar duas traits que têm métodos com o mesmo nome, e ainda ter um método próprio com esse nome:

```rust
trait Piloto {
    fn voar(&self) -> &'static str;
}

trait Mago {
    fn voar(&self) -> &'static str;
}

struct Humano;

impl Piloto for Humano { fn voar(&self) -> &'static str { "decolando" } }
impl Mago for Humano { fn voar(&self) -> &'static str { "levitando" } }
impl Humano { fn voar(&self) -> &'static str { "balançando os braços" } }
```

`h.voar()` chama o método **inerente** (o do `impl Humano`), que tem prioridade. Para chamar o de uma trait, diga qual:

```rust
let h = Humano;
println!("{} / {} / {}", h.voar(), Piloto::voar(&h), Mago::voar(&h));
```

```text
balançando os braços / decolando / levitando
```

Sem o método inerente, `h.voar()` fica ambíguo, e o compilador lista os candidatos e sugere a forma qualificada:

```text
error[E0034]: multiple applicable items in scope
  --> src/main.rs:10:22
   |
10 |     println!("{}", h.voar());
   |                      ^^^^ multiple `voar` found
   |
note: candidate #1 is defined in an impl of the trait `Mago` for the type `Humano`
...
note: candidate #2 is defined in an impl of the trait `Piloto` for the type `Humano`
...
help: disambiguate the method for candidate #1
   |
10 -     println!("{}", h.voar());
10 +     println!("{}", Mago::voar(&h));
```

`Piloto::voar(&h)` funciona porque `voar` recebe `&self`: o argumento diz qual é o tipo. Funções associadas sem `self` (como `new` ou `default`) não têm esse argumento, e chamar pela trait não basta:

```rust
// ❌ NÃO COMPILA
trait Animal { fn nome_do_filhote() -> String; }
struct Cachorro;
impl Animal for Cachorro { fn nome_do_filhote() -> String { "filhote".into() } }

fn main() {
    println!("{}", Animal::nome_do_filhote());
}
```

```text
error[E0790]: cannot call associated function on trait without specifying the corresponding `impl` type
 --> src/main.rs:6:20
  |
1 | trait Animal { fn nome_do_filhote() -> String; }
  |                ------------------------------- `Animal::nome_do_filhote` defined here
...
6 |     println!("{}", Animal::nome_do_filhote());
  |                    ^^^^^^^^^^^^^^^^^^^^^^^^^ cannot call associated function of trait
  |
help: use the fully-qualified path to the only available implementation
  |
6 |     println!("{}", <Cachorro as Animal>::nome_do_filhote());
  |                    ++++++++++++       +
```

`<Cachorro as Animal>::nome_do_filhote()` é a **sintaxe totalmente qualificada**: "a função `nome_do_filhote` da implementação de `Animal` para `Cachorro`". Ela funciona em qualquer caso, e é a forma que o compilador usa nas mensagens (você viu `<i32 as Display>::fmt` no diagrama da lição 01). No dia a dia, você só a escreve quando há ambiguidade.

É o que substitui, em Rust, a resolução de herança múltipla: em C++, `obj.Piloto::voar()` escolhe a base; em Python, o MRO decide sozinho, e às vezes de um jeito que surpreende. Em Rust, nada é decidido por ordem de declaração: ou não há ambiguidade, ou você escreve qual quer.

## Armadilhas comuns

**Parâmetro genérico onde devia ser tipo associado.** Uma `trait Conteiner<T> { fn pegar(&self, i: usize) -> Option<&T>; }` obriga todo uso a dizer `T` (`impl Conteiner<i32>`), e permite um mesmo tipo ser `Conteiner<i32>` e `Conteiner<String>`, o que raramente faz sentido. Com `type Item;`, o tipo dos elementos sai do próprio contêiner.

**Esperar especialização.** Um blanket impl `impl<T: Display> MinhaTrait for T` impede qualquer implementação específica para tipos que já têm `Display` (E0119). Decida antes qual estratégia usar.

**Esquecer de fixar o tipo associado no bound.** Em `fn soma(it: impl Iterator) -> u32`, nada diz que os itens são `u32`, e o `total += x` do corpo falha com `error[E0277]: cannot add-assign `<impl Iterator as Iterator>::Item` to `{integer}``. Escreva `impl Iterator<Item = u32>`.

## Resumo

- Tipo associado (`type Item;`): o implementador escolhe, uma implementação por tipo. Parâmetro genérico (`From<T>`): quem usa escolhe, várias implementações possíveis.
- Em bounds, fixe o tipo associado com `Trait<Item = Tipo>`; dentro do código, `Self::Item` ou `T::Item`.
- Coerência: no máximo uma implementação para cada par trait + tipo. Duas, mesmo que uma venha de um blanket impl, dão E0119. Não há especialização.
- `Add<Rhs = Self>` combina parâmetro com valor padrão e tipo associado (`Output`). A sobrecarga de operadores é o Módulo 13.
- Ambiguidade de nomes: `Trait::metodo(&x)` para métodos; `<Tipo as Trait>::funcao()` para funções sem `self`.

## Exercícios

Veja o exercício 4 em [`exercicios.md`](exercicios.md) (o `type Error` de `TryFrom` e o `type Err` de `FromStr` são tipos associados).

## Leitura complementar

- [The Rust Programming Language, 20.2 Advanced Traits](https://doc.rust-lang.org/stable/book/ch20-02-advanced-traits.html) (tipos associados, parâmetros padrão, sintaxe totalmente qualificada)
- [Comprehensive Rust: Associated Types](https://google.github.io/comprehensive-rust/methods-and-traits/associated-types.html)
- [The Rust Reference: Associated items](https://doc.rust-lang.org/reference/items/associated-items.html)
- [Documentação de `std::ops::Add`](https://doc.rust-lang.org/std/ops/trait.Add.html)

---

[← Lição anterior](07-estatico-x-dinamico-e-composicao.md) · [Índice do módulo](README.md)

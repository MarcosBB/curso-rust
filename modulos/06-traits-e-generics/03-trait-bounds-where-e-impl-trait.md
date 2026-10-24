# 03 — Trait bounds, `where` e `impl Trait`

**Objetivos.** Ao final, você vai saber:

- restringir parâmetros genéricos com *trait bounds* (`T: Trait`, `+`, `where`);
- usar `impl Trait` em argumentos e no retorno, e a diferença para `<T: Trait>`;
- ler os três erros mais comuns com generics (E0369, E0599, E0277);
- escrever `impl` condicionais e reconhecer *blanket impls* como o de `ToString`;
- escrever funções que aceitam "qualquer destino de escrita", como o `run` do minigrep vai fazer.

## O que `T` sabe fazer

Na lição 01, `largest<T>` não compilou porque nada garantia que `T` soubesse comparar. Um *trait bound* diz exatamente isso: "`T` pode ser qualquer tipo, **desde que** implemente esta trait".

```rust
fn largest<T: PartialOrd>(list: &[T]) -> &T {
    let mut largest = &list[0];
    for item in list {
        if item > largest {
            largest = item;
        }
    }
    largest
}

println!("{}", largest(&[3, 7, 2]));           // 7
println!("{}", largest(&['r', 'u', 's', 't'])); // u
```

O bound é um contrato nos dois sentidos:

- **Dentro da função**, você só pode usar o que as traits do bound oferecem. `PartialOrd` dá `<`, `>`, `<=` e `>=`; não dá `+`, nem `clone`, nem `{}`.
- **Quem chama** só pode passar tipos que implementam a trait. Isso é verificado em cada chamada.

Para exigir mais de uma trait, junte com `+`:

```rust
use std::fmt::Display;

fn notificar<T: Resumo + Display>(item: &T) {
    println!("Novidade! {} — {item}", item.resumir());
}
```

Quando a assinatura cresce, os bounds podem ir para uma cláusula `where`, depois do tipo de retorno. É a mesma coisa, só mais legível:

```rust
fn repetir<T>(valor: T, vezes: usize) -> Vec<String>
where
    T: Display + Clone,
{
    let mut saida = Vec::new();
    for _ in 0..vezes {
        saida.push(valor.clone().to_string());
    }
    saida
}
```

Comparando com C++20 e Python:

```cpp
// C++20: concept no lugar do typename, ou cláusula requires
template <std::totally_ordered T>
const T& largest(const std::vector<T>& list);

template <class T>
    requires std::copyable<T>
std::vector<T> repetir(T valor, std::size_t vezes);
```

```python
from typing import TypeVar
from typing import Protocol

class Comparavel(Protocol):
    def __gt__(self, other, /) -> bool: ...

T = TypeVar("T", bound=Comparavel)

def largest(lista: list[T]) -> T: ...
```

`T: PartialOrd` faz o papel do concept, e o `where` faz o papel do `requires`. A diferença que importa é a da lição 01: em Rust, o corpo é conferido contra o bound na definição. Se o corpo usar algo que o bound não promete, o erro aparece mesmo que ninguém chame a função.

## `impl Trait` no argumento

Para o caso simples, "um parâmetro de algum tipo que implementa X", existe uma forma curta:

```rust
fn mostrar(item: &impl Display) {
    println!("{item}");
}
```

`&impl Display` é açúcar para `fn mostrar<T: Display>(item: &T)`: o compilador cria um parâmetro genérico anônimo, e a monomorfização acontece igual. Há duas diferenças práticas.

**Cada `impl Trait` é um parâmetro separado.** Com `<T>`, os dois argumentos precisam ser do **mesmo** tipo:

```rust
// ❌ NÃO COMPILA
fn dois<T: Display>(a: &T, b: &T) {
    println!("{a} {b}");
}

fn dois_livres(a: &impl Display, b: &impl Display) {
    println!("{a} {b}");
}

fn main() {
    dois_livres(&1, &"um"); // ok: dois tipos diferentes
    dois(&1, &"um");
}
```

```text
error[E0308]: mismatched types
 --> src/main.rs:8:14
  |
8 |     dois(&1, &"um");
  |     ----     ^^^^^ expected `&{integer}`, found `&&str`
  |     |
  |     arguments to this function are incorrect
```

O primeiro argumento fixou `T` como inteiro, e o segundo não bate. Às vezes é isso que você quer (comparar dois valores do mesmo tipo, por exemplo); às vezes não.

**Não dá para usar turbofish.** O parâmetro não tem nome, então `mostrar::<i32>(5)` dá `error[E0107]: function takes 0 generic arguments but 1 generic argument was supplied`, com a nota "`impl Trait` cannot be explicitly specified as a generic argument".

Regra prática: use `impl Trait` quando o parâmetro é simples e aparece uma vez; use `<T: Trait>` quando precisa ligar vários argumentos ao mesmo tipo, usar `T` no retorno, ou deixar quem chama escolher o tipo com turbofish.

## Aplicação: escrever em "qualquer lugar"

A biblioteca padrão tem duas traits para entrada e saída de bytes: `std::io::Write` (quem recebe bytes: arquivos, stdout, sockets, `Vec<u8>`) e `std::io::Read` (quem fornece). Uma função que escreve em `&mut impl Write` funciona com todos eles:

```rust
use std::io::{self, Write};

pub fn relatorio(linhas: &[&str], out: &mut impl Write) -> io::Result<()> {
    for (i, linha) in linhas.iter().enumerate() {
        writeln!(out, "{}: {linha}", i + 1)?;
    }
    Ok(())
}
```

No programa, ela recebe o stdout. No teste, um `Vec<u8>` em memória, que você inspeciona depois:

```rust
// no main
let mut stdout = io::stdout().lock();
relatorio(&["alfa", "beta", "gama"], &mut stdout)?;

// no teste
let mut buf: Vec<u8> = Vec::new();
relatorio(&["alfa", "beta"], &mut buf).unwrap();
assert_eq!(String::from_utf8(buf).unwrap(), "1: alfa\n2: beta\n");
```

O lado da entrada é igual. `BufRead` é a trait de quem lê linha por linha, e `&[u8]` a implementa, o que faz um literal de bytes servir de "arquivo" nos testes:

```rust
pub fn contar_linhas<R: BufRead>(fonte: R) -> io::Result<usize> {
    let mut total = 0;
    for linha in fonte.lines() {
        linha?;
        total += 1;
    }
    Ok(total)
}

assert_eq!(contar_linhas(&b"um\ndois\ntres\n"[..]).unwrap(), 3);
```

O código completo, com os testes, está em [`codigo/relatorio`](codigo/relatorio/).

### Lembra do minigrep?

No Projeto 1, a função `run` escreve direto no stdout:

```rust
fn run(cfg: &Config) -> io::Result<u8> {
    let mut stdout = io::stdout().lock();
    // ...
    writeln!(stdout, "{line}")?;
```

Por isso ela ficou no `main.rs` e só pode ser testada rodando o executável inteiro (os testes de integração em `tests/cli.rs`). Com o que você acabou de ver, a mudança é pequena: `run` passa a **receber** o destino.

```rust
pub fn run(cfg: &Config, out: &mut impl Write) -> io::Result<u8>
```

O `main` passa `&mut io::stdout().lock()`, e os testes unitários passam um `Vec<u8>` e conferem a saída e o código de retorno, sem processo nenhum. É o exercício 6 deste módulo.

## Os três erros que você mais vai ver

Todos os três vêm com a correção sugerida. Vale reconhecer de cara.

**E0369: operador sem bound.** Usar `>`, `+`, `==` em `T` sem a trait correspondente. Foi o erro de `largest` na lição 01: `help: consider restricting type parameter `T` with trait `PartialOrd``.

**E0599: método sem bound.** Chamar um método de trait em `T` sem o bound:

```rust
// ❌ NÃO COMPILA
fn duplicar<T>(a: T) -> (T, T) {
    (a.clone(), a)
}
```

```text
error[E0599]: no method named `clone` found for type parameter `T` in the current scope
 --> src/main.rs:2:8
  |
1 | fn duplicar<T>(a: T) -> (T, T) {
  |             - method `clone` not found for this type parameter
2 |     (a.clone(), a)
  |        ^^^^^ method not found in `T`
  |
  = help: items from traits can only be used if the type parameter is bounded by the trait
help: the following trait defines an item `clone`, perhaps you need to restrict type parameter `T` with it:
  |
1 | fn duplicar<T: Clone>(a: T) -> (T, T) {
  |              +++++++
```

**E0277: o tipo não implementa a trait do bound.** Agora o erro é de quem chama:

```rust
// ❌ NÃO COMPILA
struct Ponto {
    x: i32,
    y: i32,
}

fn mostrar(item: &impl std::fmt::Display) {
    println!("{item}");
}

fn main() {
    let p = Ponto { x: 1, y: 2 };
    mostrar(&p);
}
```

```text
error[E0277]: `Ponto` doesn't implement `std::fmt::Display`
  --> src/main.rs:12:13
   |
12 |     mostrar(&p);
   |     ------- ^^ unsatisfied trait bound
   |     |
   |     required by a bound introduced by this call
   |
help: the trait `std::fmt::Display` is not implemented for `Ponto`
...
note: required by a bound in `mostrar`
  --> src/main.rs:6:24
   |
 6 | fn mostrar(item: &impl std::fmt::Display) {
   |                        ^^^^^^^^^^^^^^^^^ required by this bound in `mostrar`
```

A `note` aponta o bound que exigiu a trait. Quando a cadeia é longa (a função chama outra, que exige outra coisa), siga as notas de baixo para cima até achar o seu código.

A diferença entre os dois últimos: E0599 é **dentro** da função genérica (falta bound), e E0277 é **na chamada** (o tipo não cumpre o bound). Compare com C++ sem concepts, em que os dois casos viram o mesmo erro, gerado dentro do template.

## `impl` condicional

Um tipo genérico pode ter métodos que só existem quando `T` cumpre certos bounds. O Módulo 04 prometeu isto: cada bloco `impl` pode ter as suas condições.

```rust
use std::fmt::Display;

struct Par<T> {
    x: T,
    y: T,
}

// Para todo `T`.
impl<T> Par<T> {
    fn new(x: T, y: T) -> Self {
        Self { x, y }
    }
}

// Só para os `T` que sabem se comparar e se mostrar.
impl<T: Display + PartialOrd> Par<T> {
    fn maior(&self) -> String {
        if self.x >= self.y {
            format!("x = {}", self.x)
        } else {
            format!("y = {}", self.y)
        }
    }
}
```

`Par::new(3, 7).maior()` funciona. Com um tipo que não compara nem se mostra, `new` continua existindo, mas `maior` não:

```text
error[E0599]: the method `maior` exists for struct `Par<SemOrdem>`, but its trait bounds were not satisfied
    --> src/main.rs:20:7
...
note: the following trait bounds were not satisfied:
      `SemOrdem: PartialOrd`
      `SemOrdem: std::fmt::Display`
...
help: consider annotating `SemOrdem` with `#[derive(PartialEq, PartialOrd)]`
```

A biblioteca padrão usa isso em todo lugar. Vetores e slices de `T` têm `sort()` só quando `T: Ord`, e `contains()` só quando `T: PartialEq`. `Option<T>` tem `unwrap_or_default()` só quando `T: Default`.

## *Blanket impls*: uma implementação para muitos tipos

O mesmo truque vale para implementar uma trait. A biblioteca padrão tem, em essência, isto:

```rust
impl<T: Display + ?Sized> ToString for T {
    // monta uma String usando o Display de T
}
```

Leia como "todo tipo que implementa `Display` implementa `ToString`". Uma implementação que cobre uma família inteira de tipos se chama *blanket impl* (implementação "cobertor"). É por isso que `3.to_string()` funciona, que o `Ponto` da lição 02 ganhou `to_string()` sem você escrever nada, e que o minigrep pôde chamar `e.to_string()` nos erros. (O `?Sized` permite que `T` seja um tipo sem tamanho fixo, como `str`; a [lição 06](06-trait-objects-e-dyn.md) explica.)

Na documentação de qualquer trait, a seção *Implementors* lista os blanket impls. Quando um método aparece num tipo "do nada", é quase sempre um deles.

> [!NOTE]
> Por causa da orphan rule, você só pode escrever blanket impls para traits **suas**: `impl<T: Display> MinhaTrait for T`. A lição 08 mostra um efeito colateral: depois de um blanket impl, você não pode mais escrever uma implementação específica para um tipo que ele já cobre.

## `impl Trait` no retorno

`impl Trait` também pode aparecer no tipo de retorno, com outro significado: "esta função devolve **algum** tipo concreto que implementa a trait, e eu não vou dizer qual".

```rust
fn rotulo(nome: &str, valor: i32) -> impl Display {
    format!("{nome}={valor}")
}
```

Quem chama só pode usar o que `Display` oferece. O tipo real (`String`) continua conhecido pelo compilador, então não há custo nenhum. Para que serve esconder? Para tipos com nomes enormes ou sem nome. Closures e cadeias de iteradores (Módulo 07) têm tipos que você não consegue escrever, e `-> impl Iterator<Item = u32>` é a forma de devolvê-los.

A restrição: é **um** tipo concreto só. A função não pode devolver um tipo num caminho e outro tipo no outro:

```rust
// ❌ NÃO COMPILA
fn descrever(curto: bool) -> impl Display {
    if curto {
        42
    } else {
        "quarenta e dois"
    }
}
```

```text
error[E0308]: `if` and `else` have incompatible types
 --> src/main.rs:7:9
  |
4 | /     if curto {
5 | |         42
  | |         -- expected because of this
6 | |     } else {
7 | |         "quarenta e dois"
  | |         ^^^^^^^^^^^^^^^^^ expected integer, found `&str`
8 | |     }
  | |_____- `if` and `else` have incompatible types
  |
help: you could change the return type to be a boxed trait object
  |
3 - fn descrever(curto: bool) -> impl Display {
3 + fn descrever(curto: bool) -> Box<dyn Display> {
  |
```

O compilador precisa saber o tamanho e o tipo do valor de retorno na compilação, e aqui seriam dois. A sugestão do `help:`, `Box<dyn Display>`, é a ferramenta para "um de vários tipos, decidido em tempo de execução", assunto da [lição 06](06-trait-objects-e-dyn.md).

> [!NOTE]
> Na edition 2024, um `-> impl Trait` "captura" todos os parâmetros genéricos e lifetimes em escopo: o compilador assume que o valor devolvido pode guardar referências que vieram nos argumentos. Às vezes isso faz o borrow checker recusar um uso que parecia inocente, com a nota *"this call may capture more lifetimes than intended, because Rust 2024 has adjusted the `impl Trait` lifetime capture rules"* e a sugestão de escrever `+ use<T>`. Se você topar com isso, siga o `help:`. O assunto completo fica para o Módulo 13.

## Armadilhas comuns

**Bound demais.** Exija só o que o corpo usa. Um `T: Clone + Debug + Display + Default` "por garantia" reduz os tipos aceitos e engana quem lê a assinatura.

**Esquecer que o retorno de `impl Trait` é um tipo só.** `if`/`else` ou `match` com tipos diferentes dá E0308. Se o conjunto de tipos é conhecido, um `enum` resolve; se não é, `Box<dyn Trait>`.

**Confundir `impl Trait` no argumento com `dyn Trait`.** `&impl Display` é genérico (uma cópia da função por tipo); `&dyn Display` é uma função só com dispatch em tempo de execução. Os dois aceitam "qualquer `Display`", mas funcionam de jeitos diferentes (lição 06).

**Usar um `T` onde queria dois.** `fn f<T: Display>(a: &T, b: &T)` exige o mesmo tipo nos dois. Se não é essa a intenção, use dois parâmetros (`<A, B>`) ou dois `impl Display`.

## Resumo

- `T: Trait` restringe o que `T` pode ser e libera, dentro da função, o que a trait oferece. Junte com `+`; use `where` quando a lista crescer.
- `impl Trait` no argumento é açúcar para um genérico anônimo: cada ocorrência é um tipo independente, e não aceita turbofish.
- `&mut impl Write` faz uma função escrever no stdout, num arquivo ou num `Vec<u8>`. É o que torna o `run` do minigrep testável.
- E0369 e E0599: falta bound na função genérica. E0277: o tipo passado não cumpre o bound. Os três sugerem a correção.
- `impl<T: Bound> Tipo<T>` cria métodos condicionais; `impl<T: Bound> Trait for T` é um blanket impl (como `ToString` para todo `Display`).
- `-> impl Trait` devolve um tipo concreto escondido, sempre o mesmo. Para tipos diferentes, `Box<dyn Trait>`.

## Exercícios

Veja os exercícios 1 e 6 em [`exercicios.md`](exercicios.md).

## Leitura complementar

- [The Rust Programming Language, 10.2: Traits as Parameters e Trait Bound Syntax](https://doc.rust-lang.org/stable/book/ch10-02-traits.html#traits-as-parameters)
- [The Rust Reference: Trait and lifetime bounds](https://doc.rust-lang.org/reference/trait-bounds.html) e [impl Trait](https://doc.rust-lang.org/reference/types/impl-trait.html)
- [Documentação de `std::io::Write`](https://doc.rust-lang.org/std/io/trait.Write.html) (veja a lista de *Implementors*)
- [Comprehensive Rust: Read and Write](https://google.github.io/comprehensive-rust/std-traits/read-and-write.html)
- [Edition Guide: RPIT lifetime capture rules](https://doc.rust-lang.org/edition-guide/rust-2024/rpit-lifetime-capture.html)

---

[← Lição anterior](02-definindo-e-implementando-traits.md) · [Índice do módulo](README.md) · [Próxima lição →](04-traits-da-std-e-derive.md)

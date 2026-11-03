# 06 — Tipos associados avançados e GATs

**Objetivos.** Ao final, você vai saber:

- declarar constantes associadas em traits;
- restringir o tipo associado direto no bound (`impl Iterator<Item: Display>`);
- devolver `impl Trait` de um método de trait (RPITIT) e saber o que isso custa;
- usar *generic associated types* (GATs) para um iterador que empresta de si mesmo.

## Constantes associadas

Uma trait pode exigir, além de métodos e tipos, **constantes**. Cada implementação dá o seu valor:

```rust
trait Unidade {
    const SIMBOLO: &'static str;
    const POR_METRO: f64;

    fn formatar(metros: f64) -> String {
        format!("{:.2} {}", metros * Self::POR_METRO, Self::SIMBOLO)
    }
}

struct Quilometro;
struct Milha;

impl Unidade for Quilometro {
    const SIMBOLO: &'static str = "km";
    const POR_METRO: f64 = 0.001;
}

impl Unidade for Milha {
    const SIMBOLO: &'static str = "mi";
    const POR_METRO: f64 = 1.0 / 1609.344;
}

println!("{}", Quilometro::formatar(42_195.0));
println!("{}", Milha::formatar(42_195.0));
```

```text
42.20 km
26.22 mi
```

A diferença para um método `fn simbolo() -> &'static str` é que a constante é avaliada na compilação e pode ser usada onde só cabe constante (tamanho de array, padrão de `match`, outro `const`). A lição 03 usou o mesmo recurso para o nome de cada estado do pedido (`E::NOME`). A std tem vários: `i32::MAX`, `f64::EPSILON`, `char::MAX` são constantes associadas aos tipos.

> [!NOTE]
> Em C++, o equivalente é um `static constexpr` num tipo, lido por um template (`T::simbolo`), ou um *traits class* como `std::numeric_limits<T>::max()`. A diferença de sempre: em Rust, a trait declara que a constante existe, e o compilador confere na hora do `impl`, e não quando o template é instanciado.

## Bound direto no tipo associado

Uma função que junta itens de qualquer coleção, desde que eles tenham `Display`, se escrevia assim:

```rust
fn juntar<I>(itens: I, sep: &str) -> String
where
    I: IntoIterator,
    I::Item: Display,
{ /* ... */ }
```

Desde a 1.79, o bound do tipo associado pode ir **dentro** dos `<>`, o que permite escrever com `impl Trait`:

```rust
fn juntar(itens: impl IntoIterator<Item: Display>, sep: &str) -> String {
    itens
        .into_iter()
        .map(|i| i.to_string())
        .collect::<Vec<_>>()
        .join(sep)
}

println!("{}", juntar([1, 2, 3], " + "));
println!("{}", juntar(vec!['a', 'b'], ""));
```

```text
1 + 2 + 3
ab
```

Compare `Item = String` (o tipo associado **é** `String`) com `Item: Display` (o tipo associado é **qualquer** um que implemente `Display`). As duas formas convivem: `impl Iterator<Item = &str>` e `impl Iterator<Item: AsRef<str>>`.

## `impl Trait` no retorno de métodos de trait (RPITIT)

Desde a 1.75, um método de trait pode devolver `impl Trait`. É o que permite a uma trait devolver iteradores sem `Box`:

```rust
trait Colecao {
    fn itens(&self) -> impl Iterator<Item = &str>;

    fn contar_com(&self, prefixo: &str) -> usize {
        self.itens().filter(|i| i.starts_with(prefixo)).count()
    }
}

struct Lista(Vec<String>);

impl Colecao for Lista {
    fn itens(&self) -> impl Iterator<Item = &str> {
        self.0.iter().map(String::as_str)
    }
}

struct Csv(String);

impl Colecao for Csv {
    fn itens(&self) -> impl Iterator<Item = &str> {
        self.0.split(',').map(str::trim)
    }
}
```

```text
começam com 'ru': 2
itens do csv: ["rato", "roupa", "rei", "Roma"]
```

Cada implementação devolve um tipo de iterador diferente (`Map<slice::Iter<..>, ..>` e `Map<Split<..>, ..>`), e o compilador gera, por baixo, um tipo associado escondido para cada uma. O nome é feio, *return-position `impl Trait` in traits* (RPITIT), mas é o mesmo mecanismo do `async fn` em traits do [Módulo 12](../12-async/08-async-em-traits-e-closures.md): lá, o tipo escondido era uma future.

E tem a mesma limitação. Como cada implementação devolve um tipo de tamanho diferente, a trait não cabe numa *vtable*:

```rust
// ❌ NÃO COMPILA
let v: Vec<Box<dyn Colecao>> = vec![Box::new(Lista(vec![]))];
```

```text
error[E0038]: the trait `Colecao` is not dyn compatible
  --> src/main.rs:14:24
   |
14 |     let v: Vec<Box<dyn Colecao>> = vec![Box::new(Lista(vec![]))];
   |                        ^^^^^^^ `Colecao` is not dyn compatible
   |
note: for a trait to be dyn compatible it needs to allow building a vtable
      for more information, visit <https://doc.rust-lang.org/reference/items/traits.html#dyn-compatibility>
  --> src/main.rs:2:24
   |
 1 | trait Colecao {
   |       ------- this trait is not dyn compatible...
 2 |     fn itens(&self) -> impl Iterator<Item = &str>;
   |                        ^^^^^^^^^^^^^^^^^^^^^^^^^^ ...because method `itens` references an `impl Trait` type in its return type
   = help: consider moving `itens` to another trait
```

Se você precisa de `dyn`, a saída é a mesma do Módulo 12: devolver um tipo só, `Box<dyn Iterator<Item = &str> + '_>`, pagando uma alocação por chamada. Se precisa dos dois, mantenha a versão genérica e ofereça um método `itens_box` separado com `where Self: Sized` na versão `impl`.

## O limite do `Iterator`: emprestar de si mesmo

Um leitor de linhas que, em vez de alocar uma `String` nova por linha, reaproveita um **buffer** interno: limpa, preenche com a linha normalizada e devolve uma referência para ele. É o que você faria em C para não alocar num laço de milhões de linhas. Tentando com `Iterator`:

```rust
// ❌ NÃO COMPILA
struct Linhas<'t> {
    resto: &'t str,
    buf: String,
}

impl<'t> Iterator for Linhas<'t> {
    type Item = &str;

    fn next(&mut self) -> Option<Self::Item> {
        let (linha, resto) = self.resto.split_once('\n')?;
        self.resto = resto;
        self.buf.clear();
        self.buf.push_str(linha.trim());
        Some(&self.buf)
    }
}
```

```text
error: associated type `Iterator::Item` is declared without lifetime parameters, so using a borrowed type for them requires that lifetime to come from the implemented type
 --> src/main.rs:7:17
  |
7 |     type Item = &str;
  |                 ^
  |
note: you can't create an `Iterator` that borrows each `Item` from itself, but you can instead create a new type that borrows your existing type and implement `Iterator` for that new type
 --> src/main.rs:6:23
  |
6 | impl<'t> Iterator for Linhas<'t> {
  |                       ^^^^^^^^^^
```

A nota diz o essencial: *"you can't create an `Iterator` that borrows each `Item` from itself"*. Olhe a assinatura de `next`:

```rust
fn next(&mut self) -> Option<Self::Item>;
```

`Self::Item` é **um** tipo, fixado no `impl`. Ele não pode mencionar o lifetime do `&mut self` de cada chamada, porque esse lifetime nem existe quando o `impl` é escrito. Tentar inventar um parâmetro de lifetime solto também não funciona:

```rust
// ❌ NÃO COMPILA
impl<'t, 'a> Iterator for Linhas<'t> {
    type Item = &'a str;
    // ...
}
```

```text
error[E0207]: the lifetime parameter `'a` is not constrained by the impl trait, self type, or predicates
 --> src/main.rs:6:10
  |
6 | impl<'t, 'a> Iterator for Linhas<'t> {
  |          ^^ unconstrained lifetime parameter
```

E não é só um problema de sintaxe. O contrato do `Iterator` permite fazer `let a = it.next(); let b = it.next();` e usar os dois juntos (é o que `collect` faz). Se `a` apontasse para o buffer, o segundo `next` o sobrescreveria com `a` ainda em uso. O `Iterator` **precisa** proibir isso.

## GATs: um tipo associado com parâmetro

O que falta é um tipo associado que recebe um lifetime: `Item<'a>`, "o item emprestado por `'a`". Isso é um *generic associated type* (GAT), estável desde a 1.65:

```rust
trait Emprestador {
    type Item<'a>
    where
        Self: 'a;

    fn proximo(&mut self) -> Option<Self::Item<'_>>;
}
```

Leia `proximo` assim: "empresto `self` por um lifetime qualquer, e o item devolvido vive esse mesmo lifetime". O `'_` no retorno liga o `Item` ao `&mut self` daquela chamada (a elisão do Módulo 03 de sempre).

O `where Self: 'a` diz "o item emprestado por `'a` só existe se o próprio `Self` vive `'a`", o que é óbvio, mas o compilador exige que você escreva:

```rust
// ❌ NÃO COMPILA
trait Emprestador {
    type Item<'a>;
    fn proximo<'a>(&'a mut self) -> Option<Self::Item<'a>>;
}
```

```text
error: missing required bound on `Item`
 --> src/main.rs:2:5
  |
2 |     type Item<'a>;
  |     ^^^^^^^^^^^^^-
  |                  |
  |                  help: add the required where clause: `where Self: 'a`
  |
  = note: this bound is currently required to ensure that impls have maximum flexibility
  = note: we are soliciting feedback, see issue #87479 <https://github.com/rust-lang/rust/issues/87479> for more information
```

Agora o leitor de linhas:

```rust
impl Emprestador for Linhas<'_> {
    type Item<'a>
        = &'a str
    where
        Self: 'a;

    fn proximo(&mut self) -> Option<&str> {
        let (linha, resto) = self.resto.split_once('\n')?;
        self.resto = resto;
        self.buf.clear();
        self.buf
            .extend(linha.trim().chars().map(|c| c.to_ascii_uppercase()));
        Some(&self.buf)
    }
}

let texto = "  um\n dois  \ntres\n";
let mut linhas = Linhas::new(texto);
while let Some(l) = linhas.proximo() {
    println!("{l:?}");
}
```

```text
"UM"
"DOIS"
"TRES"
```

Uma alocação só, para o arquivo inteiro. E a garantia que o `Iterator` não conseguia dar aparece aqui como erro de compilação:

```rust
// ❌ NÃO COMPILA
let mut it = Linhas::new(texto);
let a = it.proximo();
let b = it.proximo();
println!("{a:?} {b:?}");
```

```text
error[E0499]: cannot borrow `it` as mutable more than once at a time
  --> src/main.rs:61:13
   |
60 |     let a = it.proximo();
   |             -- first mutable borrow occurs here
61 |     let b = it.proximo();
   |             ^^ second mutable borrow occurs here
62 |     println!("{a:?} {b:?}");
   |                - first borrow later used here
```

Enquanto `a` estiver vivo, o leitor está emprestado e ninguém chama `proximo` de novo. É a regra de *borrowing* do Módulo 03 fazendo o trabalho que, em C, seria um comentário dizendo "não guarde o ponteiro devolvido".

Esse padrão tem nome: *lending iterator* (iterador que empresta). Outro exemplo é o de janelas **mutáveis** e sobrepostas, que a std não tem (`windows` só existe para `&[T]`, e `chunks_mut` não sobrepõe), justamente porque duas janelas vivas ao mesmo tempo seriam dois `&mut` para o mesmo elemento:

```rust
struct JanelasMut<'d, T> {
    dados: &'d mut [T],
    pos: usize,
}

impl<T> Emprestador for JanelasMut<'_, T> {
    type Item<'a>
        = &'a mut [T]
    where
        Self: 'a;

    fn proximo(&mut self) -> Option<&mut [T]> {
        let janela = self.dados.get_mut(self.pos..self.pos + 2)?;
        self.pos += 1;
        Some(janela)
    }
}

/// Soma acumulada no lugar: `[1, 2, 3, 4]` vira `[1, 3, 6, 10]`.
fn acumular(v: &mut [i64]) {
    let mut j = JanelasMut { dados: v, pos: 0 };
    while let Some(par) = j.proximo() {
        par[1] += par[0];
    }
}
```

```text
[1, 3, 6, 10]
```

### O preço

GATs resolvem o problema, mas o ecossistema em volta do `Iterator` não vem junto:

- **Não há `for`.** O `for` usa `IntoIterator`; com `Emprestador`, é `while let Some(x) = it.proximo()`.
- **Não há `map`, `filter`, `collect`.** Você teria de escrever os adaptadores (e alguns, como `collect`, nem fazem sentido: guardar todos os itens é justamente o que é proibido).
- **Funções genéricas funcionam**, como `fn contar<E: Emprestador>(mut e: E) -> usize`, **até** você precisar de um bound sobre o item. Uma função que recebe uma closure para filtrar os itens precisa de um HRTB (lição 08), `F: for<'a> FnMut(E::Item<'a>) -> bool`, e esbarra numa limitação atual do compilador: o `where Self: 'a` do GAT, somado ao "para todo `'a`", vira a exigência de que o emprestador seja `'static`. Com um `JanelasMut` sobre um array local, o erro é um E0597 com a nota *"due to a current limitation of the type system, this implies a `'static` lifetime"*. Há contornos (com traits auxiliares), mas são avançados; na prática, escreva o laço `while let` direto onde precisar.

A crate [`lending-iterator`](https://docs.rs/lending-iterator) tem uma versão com adaptadores, e a std pode ganhar uma trait assim no futuro. Por ora, use um lending iterator quando a alocação por item realmente pesar; senão, devolver `String` com `Iterator` é mais simples.

> [!NOTE]
> GATs servem para mais que lifetimes. `type Ponteiro<T>` numa trait permite escrever código genérico sobre "`Rc` ou `Arc`", por exemplo: `trait Familia { type Ponteiro<T>: Deref<Target = T> + Clone; }`. Em C++, isso seria um *template template parameter*. O post de estabilização dos GATs (na leitura complementar) mostra esses usos.

Código completo, com testes, em [`codigo/gats`](codigo/gats/src/main.rs).

## Armadilhas comuns

- **`type Item = &str` num `Iterator` que empresta de `self`.** Não tem como. Ou devolva um valor com posse (`String`), ou crie um tipo separado que empresta a coleção (como `slice::Iter` empresta o `Vec`), ou use um GAT.
- **Esquecer o `where Self: 'a` do GAT.** O erro *"missing required bound"* diz exatamente o que escrever.
- **RPITIT e `dyn`.** Método que devolve `impl Trait` impede `dyn` (E0038). Se precisar de `dyn`, devolva `Box<dyn Trait + '_>`.
- **Confundir `Item = X` com `Item: X`.** O primeiro fixa o tipo; o segundo é um bound.

## Resumo

- Constantes associadas (`const SIMBOLO: &str;`) são valores por implementação, avaliados na compilação.
- `impl IntoIterator<Item: Display>` (1.79) põe o bound do tipo associado direto no `<>`.
- RPITIT (1.75): métodos de trait podem devolver `impl Trait`, sem custo, mas a trait deixa de ser dyn-compatível.
- `Iterator` não pode devolver itens que emprestam do próprio iterador, porque `Item` não tem acesso ao lifetime de `&mut self`.
- GATs (`type Item<'a> where Self: 'a`, 1.65) resolvem isso: o *lending iterator*. O custo é ficar sem `for` e sem adaptadores.

## Exercícios

Veja o exercício 5 em [`exercicios.md`](exercicios.md).

## Leitura complementar

- [The Rust Reference: Associated items](https://doc.rust-lang.org/reference/items/associated-items.html) (constantes, tipos e GATs)
- [Blog do Rust: Generic associated types to be stable in Rust 1.65](https://blog.rust-lang.org/2022/10/28/gats-stabilization/)
- [Blog do Rust: Announcing `async fn` and return-position `impl Trait` in traits](https://blog.rust-lang.org/2023/12/21/async-fn-rpit-in-traits/)
- [Blog do Rust: Rust 1.79.0](https://blog.rust-lang.org/2024/06/13/Rust-1.79.0/) (*associated type bounds*)
- [The Rust Programming Language, 20.2 Advanced Traits](https://doc.rust-lang.org/stable/book/ch20-02-advanced-traits.html) (tipos associados)

---

[← Lição anterior](05-blanket-extension-e-sealed.md) · [Índice do módulo](README.md) · [Próxima lição →](07-variancia-e-lifetimes-avancados.md)

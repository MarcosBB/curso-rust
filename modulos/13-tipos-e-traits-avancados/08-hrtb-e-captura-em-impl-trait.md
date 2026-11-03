# 08 — HRTB e a captura de lifetimes em `impl Trait`

**Objetivos.** Ao final, você vai saber:

- ler e escrever um *higher-ranked trait bound* (`for<'a> ...`) e reconhecer o que já fica escondido em `Fn(&str) -> &str`;
- resolver o caso da closure que devolve uma referência ao argumento;
- explicar a regra de captura de lifetimes em `-> impl Trait` da edition 2024;
- usar `use<..>` para declarar exatamente o que um `impl Trait` captura.

## O lifetime que quem chama não escolhe

Até aqui, todo parâmetro de lifetime era escolhido por **quem chama** a função. Em `fn maior<'a>(v: &[&'a str]) -> &'a str`, cada chamada fixa um `'a`. Isso não serve quando a função vai criar referências **por conta própria**, lá dentro, com lifetimes que o chamador nem consegue nomear.

Uma função que recebe uma coleção (por valor) e a percorre duas vezes **emprestada**:

```rust
// ❌ NÃO COMPILA
fn resumo<'a, C>(c: C) -> (i32, usize)
where
    &'a C: IntoIterator<Item = &'a i32>,
{
    let soma = (&c).into_iter().sum();
    let n = (&c).into_iter().count();
    (soma, n)
}
```

```text
error[E0597]: `c` does not live long enough
 --> src/main.rs:5:16
  |
1 | fn resumo<'a, C>(c: C) -> (i32, usize)
  |           --     - binding `c` declared here
  |           |
  |           lifetime `'a` defined here
...
5 |     let soma = (&c).into_iter().sum();
  |                ^^^^------------
  |                |
  |                borrowed value does not live long enough
  |                argument requires that `c` is borrowed for `'a`
...
8 | }
  | - `c` dropped here while still borrowed
...
error[E0309]: the parameter type `C` may not live long enough
  ...
help: consider adding an explicit lifetime bound
  |
1 | fn resumo<'a, C: 'a>(c: C) -> (i32, usize)
  |                ++++
```

(São sete erros ao todo, variações desses dois.) O bound diz "`&'a C` itera `&'a i32`" para **um** `'a`, escolhido pelo chamador, que pode ser enorme. Mas `&c` é um empréstimo de uma variável local, que morre no fim da função. Não há como ele durar um `'a` arbitrário. E o `help:` não resolve: com `C: 'a`, somem os E0309 e os E0597 continuam.

O que você quer dizer é outra coisa: "para **qualquer** lifetime `'a`, `&'a C` itera `&'a i32`". Assim, qualquer empréstimo que a função fizer, de qualquer duração, serve. Essa é a sintaxe `for<'a>`:

```rust
// ✅
fn resumo<C>(c: C) -> (i32, usize)
where
    for<'a> &'a C: IntoIterator<Item = &'a i32>,
{
    let soma = (&c).into_iter().sum();
    let n = (&c).into_iter().count();
    (soma, n)
}

println!("{:?}", resumo(vec![1, 2, 3]));
println!("{:?}", resumo([10, 20]));
println!("{:?}", resumo(BTreeSet::from([5, 5, 7])));
```

```text
(6, 3)
(30, 2)
(12, 2)
```

Isso é um *higher-ranked trait bound* (HRTB): um bound que vale para todos os lifetimes ao mesmo tempo, em vez de um escolhido de fora. O `'a` não é mais parâmetro da função (sumiu do `<C>`); ele vive só dentro do `for<'a>`. Leia `for<'a>` como "para todo `'a`".

## Você já usa HRTB sem saber

O caso mais comum de HRTB está escondido em todo bound de closure que recebe referência:

```rust
F: Fn(&str) -> usize
```

O compilador aplica a elisão de lifetimes aqui (Módulo 03), e o resultado é:

```rust
F: for<'a> Fn(&'a str) -> usize
```

"`f` aceita uma `&str` de **qualquer** duração". É o que permite chamar a mesma closure com um literal, com um `&String` local e com um pedaço de outro texto. Sem o `for<'a>`, a closure só aceitaria referências de um lifetime fixo. A sintaxe explícita só é necessária quando a elisão não consegue escrevê-la por você, como no `&'a C` acima, que não é um `Fn`.

### A closure do Módulo 07

O [Módulo 07](../07-colecoes-iteradores-e-closures/04-closures-e-captura.md) mostrou uma closure que não compila:

```rust
// ❌ NÃO COMPILA (Módulo 07)
let primeira = |s: &str| -> &str { s.split(' ').next().unwrap_or("") };
```

Agora dá para nomear o problema. Para uma closure guardada numa variável, o compilador dá a cada `&` anotado um lifetime **próprio** (`'1` para a entrada, `'2` para a saída) e não infere que um deve ser igual ao outro. O que você queria era a assinatura `for<'a> Fn(&'a str) -> &'a str`: "para qualquer `'a`, recebe `&'a str` e devolve `&'a str`".

Não existe sintaxe para anotar isso direto numa closure (`for<'a> |s: &'a str| -> &'a str` ainda é instável). Mas há um truque: passar a closure por uma função cujo bound **é** esse. O bound guia a inferência:

```rust
// ✅
fn com_hrtb<F>(f: F) -> F
where
    F: for<'a> Fn(&'a str) -> &'a str,
{
    f
}

let primeira = com_hrtb(|s| s.split(' ').next().unwrap_or(""));
let nome = String::from("Bia Lima");
println!("{} {}", primeira("Ana Souza"), primeira(&nome));
```

```text
Ana Bia
```

A função não faz nada em tempo de execução; ela existe só para dar o tipo. É o mesmo motivo pelo qual a closure escrita direto em `.map(...)` funciona: o `map` tem um bound que diz ao compilador o que esperar. Na prática, a recomendação do Módulo 07 continua valendo (escreva uma `fn`), mas agora você sabe o que ela evita.

> [!NOTE]
> Em C++, um template não tem esse problema, porque não verifica tempo de vida: `[](const std::string& s) -> std::string_view { ... }` compila, e um `string_view` pendurado num temporário é um bug clássico. O HRTB é o preço de o Rust verificar o caso genérico: ele precisa de uma forma de dizer "funciona para qualquer empréstimo".

Onde mais HRTB aparece:

- bounds sobre os itens de um GAT, como `F: for<'a> FnMut(E::Item<'a>) -> bool` (com a limitação que a lição 06 mostrou);
- bounds de traits de serialização: `for<'de> Deserialize<'de>` (a `serde` tem até um alias, `DeserializeOwned`, para não escrever isso);
- tipos de ponteiro de função e trait objects: `Box<dyn for<'a> Fn(&'a str) -> &'a str>` (que a elisão também deixa escrever `Box<dyn Fn(&str) -> &str>`).

## `impl Trait` no retorno: o que ele captura?

O [Módulo 06](../06-traits-e-generics/03-trait-bounds-where-e-impl-trait.md) avisou que `-> impl Trait` "captura" parâmetros, e que isso às vezes dá um erro inesperado. Eis o erro:

```rust
// ❌ NÃO COMPILA
fn indices_pares(v: &[i32]) -> impl Iterator<Item = usize> {
    (0..v.len()).step_by(2)
}

fn main() {
    let mut v = vec![10, 20, 30, 40];
    let it = indices_pares(&v);
    v.push(50);
    for i in it {
        println!("{i}");
    }
}
```

```text
error[E0502]: cannot borrow `v` as mutable because it is also borrowed as immutable
 --> src/main.rs:8:5
  |
7 |     let it = indices_pares(&v);
  |                            -- immutable borrow occurs here
8 |     v.push(50);
  |     ^^^^^^^^^^ mutable borrow occurs here
9 |     for i in it {
  |              -- immutable borrow later used here
  |
note: this call may capture more lifetimes than intended, because Rust 2024 has adjusted the `impl Trait` lifetime capture rules
 --> src/main.rs:7:14
  |
7 |     let it = indices_pares(&v);
  |              ^^^^^^^^^^^^^^^^^
help: use the precise capturing `use<...>` syntax to make the captures explicit
  |
1 | fn indices_pares(v: &[i32]) -> impl Iterator<Item = usize> + use<> {
  |                                                            +++++++
```

O iterador devolvido é um `StepBy<Range<usize>>`: ele **não** guarda nenhuma referência a `v`, só dois números. Mas quem chama não vê o tipo real, só `impl Iterator<Item = usize>`. E o compilador, olhando só a assinatura, precisa decidir: o valor escondido pode conter o `&[i32]` recebido, ou não?

Essa decisão é a **captura**. Um `impl Trait` que captura o lifetime do argumento é tratado, por quem chama, como se emprestasse o argumento: enquanto `it` estiver vivo, `v` está emprestado, e o `push` é recusado.

### A regra da edition 2024

| Onde | Captura parâmetros de tipo | Captura lifetimes |
|---|---|---|
| `-> impl Trait` em `fn` livre ou método inerente, **edition 2021** | todos | só os que aparecem nos bounds |
| `-> impl Trait` em `fn` livre ou método inerente, **edition 2024** | todos | **todos** em escopo |
| `async fn`, `impl Trait` em traits (RPITIT) e nas suas impls, qualquer edition | todos | todos |

Na 2021, o exemplo acima compilava, porque o `'_` do `&[i32]` não aparecia em `Iterator<Item = usize>`. Em compensação, a função `pares` abaixo, que **realmente** empresta `v`, não compilava sem um truque:

```rust
fn pares(v: &[i32]) -> impl Iterator<Item = i32> {
    v.iter().copied().filter(|x| x % 2 == 0)
}
```

Na 2021, você tinha de escrever `+ '_` no retorno (ou, em casos com vários lifetimes, uma trait auxiliar chamada `Captures`). Na prática, "capturar tudo" é o que se quer na maioria das vezes, e a 2024 fez disso o padrão, alinhado com `async fn` e com os métodos de trait, que sempre capturaram tudo. A `pares` compila na 2024 sem nada a mais.

O preço é o caso do começo: quando o valor devolvido **não** usa o empréstimo, a captura automática prende o argumento sem necessidade. Para isso existe `use<..>`.

### `use<..>`: captura precisa

Desde a 1.82 (em qualquer edition), um `impl Trait` pode listar **exatamente** o que captura, com `+ use<...>`:

```rust
// ✅ não captura nada: o iterador não prende `v`
fn indices_pares(v: &[i32]) -> impl Iterator<Item = usize> + use<> {
    (0..v.len()).step_by(2)
}

let mut v = vec![10, 20, 30, 40];
let it = indices_pares(&v);
v.push(50);
println!("{:?}", it.collect::<Vec<_>>());
```

```text
[0, 2]
```

`use<>` (lista vazia) quer dizer "não captura nenhum parâmetro". Com `use<'a>`, captura só `'a`; com `use<'a, T>`, captura `'a` e `T`. É uma promessa verificada: se você declarar `use<>` e o corpo devolver algo que empresta o argumento, o compilador recusa:

```rust
// ❌ NÃO COMPILA
fn pares(v: &[i32]) -> impl Iterator<Item = i32> + use<> {
    v.iter().copied().filter(|x| x % 2 == 0)
}
```

```text
error[E0700]: hidden type for `impl Iterator<Item = i32>` captures lifetime that does not appear in bounds
 --> src/main.rs:2:5
  |
1 | fn pares(v: &[i32]) -> impl Iterator<Item = i32> + use<> {
  |                        --------------------------------- opaque type defined here
2 |     v.iter().copied().filter(|x| x % 2 == 0)
  |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
  |
  = note: hidden type `Filter<Copied<Iter<'_, i32>>, {closure@...}>` captures lifetime `'_`
```

Uma restrição atual: **parâmetros de tipo** em escopo precisam constar da lista. Só os lifetimes podem ser deixados de fora.

```rust
// ❌ NÃO COMPILA
fn indices_pares<T>(v: &[T]) -> impl Iterator<Item = usize> + use<> {
    (0..v.len()).step_by(2)
}
```

```text
error: `impl Trait` must mention all type parameters in scope in `use<...>`
 --> src/main.rs:1:33
  |
1 | fn indices_pares<T>(v: &[T]) -> impl Iterator<Item = usize> + use<> {
  |                  -              ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
  |                  |
  |                  type parameter is implicitly captured by this `impl Trait`
  |
  = note: currently, all type parameters are required to be mentioned in the precise captures list
```

A correção é `use<T>`: captura o tipo, mas não o lifetime do empréstimo. Um exemplo que aproveita isso:

```rust
fn repetir<T: Clone>(x: &T, n: usize) -> impl Iterator<Item = T> + use<T> {
    std::iter::repeat_n(x.clone(), n)
}

let mut s = String::from("oi");
let rep = repetir(&s, 3);
s.push('!'); // permitido: `rep` não prende o empréstimo de `s`
println!("{:?} {s}", rep.collect::<Vec<_>>());
```

```text
["oi", "oi", "oi"] oi!
```

### Em traits

Desde a 1.87, `use<..>` também vale em métodos de trait que devolvem `impl Trait`. Ali, `Self` é um parâmetro de tipo implícito, e precisa estar na lista:

```rust
// ❌ NÃO COMPILA
trait Contador {
    fn indices(&self) -> impl Iterator<Item = usize> + use<>;
}
```

```text
error: `impl Trait` must mention the `Self` type of the trait in `use<...>`
 --> src/main.rs:2:26
  |
1 | trait Contador {
  | -------------- `Self` type parameter is implicitly captured by this `impl Trait`
2 |     fn indices(&self) -> impl Iterator<Item = usize> + use<>;
  |                          ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
```

```rust
// ✅ captura o tipo Self, mas não o empréstimo de &self
trait Contador {
    fn indices(&self) -> impl Iterator<Item = usize> + use<Self>;
}
```

### Quando escrever `use<..>`

Quase nunca de início. A regra da 2024 acerta na maioria das funções, e o compilador avisa quando não acerta, com a nota *"this call may capture more lifetimes than intended"* e a sugestão pronta. Escreva `use<..>` quando:

- o erro acima aparecer, e o valor devolvido de fato não usa o empréstimo;
- você publica uma biblioteca e quer **prometer** aos usuários que o iterador devolvido não prende o argumento (mudar isso depois seria uma quebra de API).

Ao migrar código da 2021 com `cargo fix --edition`, o lint `impl_trait_overcaptures` insere `use<..>` onde a captura mudaria. Os truques antigos (`+ 'a` e a trait `Captures`) ainda compilam, mas não são mais necessários: troque por `use<..>` ou apague.

Código completo, com testes, em [`codigo/variancia-e-hrtb`](codigo/variancia-e-hrtb/src/main.rs).

## Armadilhas comuns

- **Lifetime de função onde precisava de `for<'a>`.** Se a função empresta algo **local** e o bound fala do lifetime desse empréstimo, o lifetime não pode ser um parâmetro da função. O sintoma é um E0597 dizendo *"argument requires that `c` is borrowed for `'a`"*.
- **Seguir o `help:` de `C: 'a` num caso de HRTB.** Ele silencia os E0309, mas não resolve o E0597. A correção é `for<'a>`.
- **Closure guardada em variável que devolve referência ao argumento.** Escreva uma `fn`, ou passe a closure por uma função com bound `for<'a> Fn(&'a T) -> &'a U`.
- **`use<>` em função genérica.** Os parâmetros de tipo precisam aparecer: `use<T>`.
- **`use<>` quando o valor realmente empresta.** É E0700. A captura automática estava certa.
- **Copiar `+ '_` ou `Captures` de material antigo.** Na 2024 são desnecessários.

## Resumo

- `for<'a> Bound<'a>` (HRTB) vale para **todo** lifetime; serve quando a função cria empréstimos internos que o bound precisa cobrir.
- `F: Fn(&str) -> &str` já é um HRTB, escrito pela elisão. A forma explícita aparece em bounds que não são `Fn` (`for<'a> &'a C: IntoIterator<Item = &'a T>`).
- Na edition 2024, `-> impl Trait` captura todos os lifetimes e tipos em escopo; quem chama trata o valor como se emprestasse os argumentos.
- `+ use<..>` (1.82; em traits, 1.87) lista exatamente o que é capturado. Tipos (e `Self`, em traits) precisam estar na lista; lifetimes podem ficar de fora.
- O compilador avisa quando a captura atrapalha e sugere o `use<..>` certo.

## Exercícios

Veja o exercício 4 em [`exercicios.md`](exercicios.md).

## Leitura complementar

- [The Rust Reference: Higher-ranked trait bounds](https://doc.rust-lang.org/reference/trait-bounds.html#higher-ranked-trait-bounds) e [Use bounds](https://doc.rust-lang.org/reference/trait-bounds.html#use-bounds)
- [The Rustonomicon: Higher-Rank Trait Bounds](https://doc.rust-lang.org/nomicon/hrtb.html)
- [Edition Guide: RPIT lifetime capture rules](https://doc.rust-lang.org/edition-guide/rust-2024/rpit-lifetime-capture.html)
- [Blog do Rust: Rust 1.82.0](https://blog.rust-lang.org/2024/10/17/Rust-1.82.0/) (*precise capturing*) e [Rust 1.87.0](https://blog.rust-lang.org/2025/05/15/Rust-1.87.0/) (`use<..>` em traits)

---

[← Lição anterior](07-variancia-e-lifetimes-avancados.md) · [Índice do módulo](README.md) · [Próximo: exercícios →](exercicios.md)

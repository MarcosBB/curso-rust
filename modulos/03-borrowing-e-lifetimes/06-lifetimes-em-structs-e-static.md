# 06 — Lifetimes em structs e `'static`

**Objetivos.** Ao final, você vai saber:

- declarar uma struct que guarda referências (`struct Excerpt<'a>`) e escrever o `impl` dela;
- escolher entre devolver `&str` (preso a `self`) e `&'a str` (preso ao dado original);
- quando faz sentido guardar referências numa struct, e quando é melhor guardar valores com posse;
- o que `'static` significa de verdade, e os mitos mais comuns sobre ele.

## Uma struct que aponta para fora

Structs aparecem com detalhe no Módulo 04. Aqui basta a forma mais simples: `struct Nome { campo: Tipo }`. Se um campo for uma referência, o compilador exige um lifetime:

```rust
// ❌ NÃO COMPILA
struct Excerpt {
    part: &str,
}

fn main() {
    let novel = String::from("Chamai-me Ismael. Alguns anos atrás...");
    let e = Excerpt { part: &novel[..16] };
    println!("{}", e.part);
}
```

```text
error[E0106]: missing lifetime specifier
 --> src/main.rs:2:11
  |
2 |     part: &str,
  |           ^ expected named lifetime parameter
  |
help: consider introducing a named lifetime parameter
  |
1 ~ struct Excerpt<'a> {
2 ~     part: &'a str,
  |
```

Em funções, a elisão às vezes resolve. Em structs, nunca: o lifetime sempre vira um parâmetro do tipo.

```rust
struct Excerpt<'a> {
    part: &'a str,
}
```

Leia como: **"uma `Excerpt` não pode viver mais que o texto para o qual `part` aponta"**. O tipo completo agora é `Excerpt<'a>`, assim como `Vec<T>` tem um parâmetro de tipo. O compilador verifica a regra como verificaria uma referência solta:

```rust
// ❌ NÃO COMPILA
struct Excerpt<'a> {
    part: &'a str,
}

fn main() {
    let e;
    {
        let novel = String::from("Chamai-me Ismael. Alguns anos atrás...");
        e = Excerpt { part: &novel[..16] };
    }
    println!("{}", e.part);
}
```

```text
error[E0597]: `novel` does not live long enough
  --> src/main.rs:9:30
   |
 8 |         let novel = String::from("Chamai-me Ismael. Alguns anos atrás...");
   |             ----- binding `novel` declared here
 9 |         e = Excerpt { part: &novel[..16] };
   |                              ^^^^^ borrowed value does not live long enough
10 |     }
   |     - `novel` dropped here while still borrowed
11 |     println!("{}", e.part);
   |                    ------ borrow later used here
```

Em C++, um `struct` com um campo `std::string_view` ou `const T&` compila nessa mesma situação e fica pendurado. Em Rust, o parâmetro `'a` carrega a dependência junto com o tipo para onde quer que a struct vá.

## `impl<'a>`: métodos de uma struct com lifetime

No `impl`, o lifetime é declarado de novo, logo depois de `impl`, e usado no tipo:

```rust
struct Excerpt<'a> {
    part: &'a str,
}

impl<'a> Excerpt<'a> {
    fn part(&self) -> &str {
        self.part
    }

    fn part_long(&self) -> &'a str {
        self.part
    }
}

fn main() {
    let novel = String::from("Chamai-me Ismael. Alguns anos atrás...");
    let first = novel.split('.').next().unwrap_or("");
    let e = Excerpt { part: first };
    println!("{} | {}", e.part(), e.part_long());
}
```

```text
Chamai-me Ismael | Chamai-me Ismael
```

Os dois métodos devolvem a mesma coisa, mas com contratos diferentes:

- `part(&self) -> &str`: pela regra 3 da elisão, o retorno fica preso ao **empréstimo de `self`**, ou seja, à `Excerpt`.
- `part_long(&self) -> &'a str`: o retorno fica preso a `'a`, ou seja, ao **texto original**. Ele pode sobreviver à `Excerpt`.

A diferença aparece quando a struct é temporária:

```rust
// ❌ NÃO COMPILA
struct Excerpt<'a> {
    part: &'a str,
}

impl<'a> Excerpt<'a> {
    fn part(&self) -> &str {
        self.part
    }
}

fn first_part(text: &str) -> &str {
    let e = Excerpt { part: &text[..6] };
    e.part()
}

fn main() {
    println!("{}", first_part("Chamai-me Ismael."));
}
```

```text
error[E0515]: cannot return value referencing local variable `e`
  --> src/main.rs:13:5
   |
13 |     e.part()
   |     -^^^^^^^
   |     |
   |     returns a value referencing data owned by the current function
   |     `e` is borrowed here
```

Pelo contrato de `part`, o retorno empresta de `e`, que morre no fim da função. Na verdade ele aponta para `text`, que sobrevive, mas a assinatura não diz isso. Trocando a chamada para `e.part_long()`, compila: o contrato agora diz "vem de `'a`", e `'a` é o lifetime de `text`.

> [!TIP]
> Em structs que são "visões" de um texto (parsers, tokenizadores, iteradores), os métodos que devolvem pedaços do texto quase sempre devem devolver `&'a str`, e não `&str`. O exercício 4 é exatamente isso.

Quando o `impl` não precisa nomear o lifetime, use `'_`: `impl Excerpt<'_> { ... }`. É o mesmo lifetime anônimo da lição 05.

## Quando guardar referências numa struct

Uma struct com referência é uma **visão temporária** de dados que pertencem a outra pessoa. Isso é ótimo para:

- parsers e tokenizadores que percorrem um texto sem copiá-lo;
- resultados de busca que apontam para dentro do conteúdo (o `minigrep` do Projeto 1);
- iteradores sobre uma coleção.

Para dados de domínio, que vivem muito e circulam pelo programa (um `Usuario`, um `Pedido`, uma configuração), prefira **campos com posse**: `String`, `Vec<T>`. Pôr `&str` numa struct de domínio "para economizar uma cópia" espalha o `'a` por todo tipo e função que toca nela, e o ganho costuma ser irrelevante.

| Struct | Campo | Por quê |
|---|---|---|
| `Usuario` | `nome: String` | vive muito, é guardado, enviado e alterado |
| `Token<'a>` | `texto: &'a str` | existe só enquanto o código-fonte está sendo analisado |
| `Match<'a>` | `linha: &'a str` | resultado de busca, consumido logo em seguida |

## `'static`

`'static` é o único lifetime com nome reservado. Uma referência `&'static T` vale **até o fim do programa**. O caso mais comum são os literais de string:

```rust
fn main() {
    let s: &'static str = "estou no binário";
    static GREETING: &str = "olá";
    println!("{s} {GREETING}");
}
```

```text
estou no binário olá
```

O texto de um literal fica gravado no executável e nunca é liberado. Por isso todo `"..."` tem tipo `&'static str`. O mesmo vale para referências a um `static`. (Na declaração de `static` e `const`, o `'static` fica implícito.)

Na lição 04, o E0106 de `fn greeting() -> &str` sugeria `'static` e avisava: *"this is uncommon unless you're returning a borrowed value from a `const` or a `static`"*. Ou seja, `-> &'static str` é a resposta certa quando você devolve literais:

```rust
fn status(ok: bool) -> &'static str {
    if ok { "ok" } else { "falhou" }
}
```

Se o texto é montado em tempo de execução, `'static` não é a resposta. Devolva `String`.

### Mitos sobre `'static`

**"`'static` quer dizer que o valor vive o programa todo."** Vale para a **referência** `&'static T`. Mas em restrições como `T: 'static` (que você vai encontrar em threads, no Módulo 11), o significado é outro: o tipo `T` **não contém referências temporárias**. Um `String` satisfaz `T: 'static`, mesmo sendo liberado no fim do escopo, porque é dono de tudo que tem. Quem cobre esse detalhe é o artigo do pretzelhammer na leitura complementar.

**"Posso estender a vida de algo com `'static`."** Não. Lifetimes não existem em tempo de execução. São só a prova que o compilador faz. Escrever `'static` numa assinatura não faz nada viver mais. Só faz o compilador exigir que o valor já viva tanto.

**"Se o compilador sugeriu `'static`, é para usar."** Quase nunca. Na maioria dos casos, a correção é devolver um valor com posse.

## O que fica para depois

Este módulo cobre o que você precisa no dia a dia: anotações básicas, elisão, `'_`, `'static` e structs com referência. Ficam para o **Módulo 13** os tópicos avançados:

- restrições entre lifetimes e tipos (`T: 'a`, `'a: 'b`);
- *variância* (por que `&'static str` pode ser usado onde se espera `&'a str`);
- HRTB (`for<'a> Fn(&'a T)`);
- captura de lifetimes em `impl Trait` e a sintaxe `use<..>`.

## Armadilhas comuns

**Struct de domínio com `&str`.** Funciona até a primeira vez que você precisar guardar, devolver ou mandar a struct para outra thread. Use `String`.

**Método de visão devolvendo `&str` em vez de `&'a str`.** O resultado fica preso à struct, e não ao dado. O sintoma é um E0515 ou E0597 apontando para a struct temporária.

**Confundir `&'static T` com `T: 'static`.** O primeiro é uma referência que vale para sempre. O segundo é uma restrição: "não contém referências temporárias".

## Resumo

- Struct com campo referência precisa de parâmetro de lifetime: `struct Excerpt<'a> { part: &'a str }`. Ela não pode viver mais que o dado apontado.
- No `impl<'a> Excerpt<'a>`, `-> &str` prende o retorno a `self`. `-> &'a str` prende ao dado original e deixa o resultado sobreviver à struct.
- Use structs com referência para visões temporárias. Para dados de domínio, campos com posse.
- `&'static str` é o tipo dos literais. Para texto montado em runtime, devolva `String`.
- `T: 'static` quer dizer "sem referências temporárias", e não "vive para sempre". Variância, HRTB e `use<..>` ficam para o Módulo 13.

## Exercícios

Veja o exercício 4 em [`exercicios.md`](exercicios.md).

## Leitura complementar

- [The Rust Programming Language, 10.3, seções Lifetime Annotations in Struct Definitions e The Static Lifetime](https://doc.rust-lang.org/stable/book/ch10-03-lifetime-syntax.html#lifetime-annotations-in-struct-definitions)
- [Rust by Example: Static](https://doc.rust-lang.org/rust-by-example/scope/lifetime/static_lifetime.html)
- [pretzelhammer: Common Rust Lifetime Misconceptions](https://github.com/pretzelhammer/rust-blog/blob/master/posts/common-rust-lifetime-misconceptions.md) (mitos 2 e 8, sobre `'static`)
- Código desta lição: [`codigo/lifetimes`](codigo/lifetimes/)

---

[← Lição anterior](05-elisao-e-lifetime-anonimo.md) · [Índice do módulo](README.md) · [Próxima lição →](07-brigando-menos-com-o-borrow-checker.md)

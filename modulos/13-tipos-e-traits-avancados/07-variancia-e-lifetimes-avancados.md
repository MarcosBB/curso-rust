# 07 — Variância e lifetimes avançados

**Objetivos.** Ao final, você vai saber:

- explicar por que `&'static str` serve onde se espera `&'a str`, e por que o mesmo **não** vale atrás de um `&mut`;
- ler a tabela de variância e prever quando o compilador vai encolher um lifetime;
- usar as restrições `T: 'a` e `'a: 'b`;
- escolher o `PhantomData` certo para o seu tipo.

## Lifetimes têm "subtipos"

O [Módulo 03](../03-borrowing-e-lifetimes/06-lifetimes-em-structs-e-static.md) deixou uma pergunta: por que este código compila?

```rust
fn maior<'a>(v: &[&'a str]) -> &'a str {
    v.iter().copied().max_by_key(|s| s.len()).unwrap_or("")
}

let fixos: Vec<&'static str> = vec!["ana", "beatriz"];
let local = String::from("carolina");
let mistos: Vec<&str> = vec![fixos[0], &local];
println!("{} {}", maior(&fixos), maior(&mistos));
```

```text
beatriz carolina
```

`mistos` junta um `&'static str` com uma referência a `local`, que vive bem menos. E `maior(&fixos)` passa um `&[&'static str]` para uma função que pede `&[&'a str]`. Nos dois casos, o compilador aceita um lifetime **mais longo** onde se espera um mais curto.

A ideia por trás é intuitiva: uma referência que vale para sempre também vale por qualquer trecho menor. Em termos de tipos, `'static` é um *subtipo* de `'a` (escreve-se `'static: 'a`, "`'static` vive pelo menos tanto quanto `'a`"), e um valor de subtipo pode ser usado onde se espera o tipo maior, como um `Gato` onde se espera um `Animal` numa linguagem orientada a objetos.

```text
'static  ──────────────────────────────────────────────►  (o programa todo)
'a                  ────────────────►                       (um trecho)

&'static str  pode ser usado como  &'a str      (encolher é seguro)
&'a str       NÃO pode ser usado como  &'static str
```

Rust não tem herança, então essa é praticamente a **única** subtipagem da linguagem: entre lifetimes. E a pergunta interessante é como ela se propaga para tipos compostos. Se `&'static str` serve como `&'a str`, então `Vec<&'static str>` serve como `Vec<&'a str>`? E `&mut Vec<&'static str>` serve como `&mut Vec<&'a str>`? Isso é a *variância*.

## Covariante: encolher é seguro

Um tipo é **covariante** num parâmetro quando a subtipagem passa direto: se `'static` serve como `'a`, então `&'static str` serve como `&'a str`, `Vec<&'static str>` serve como `Vec<&'a str>`, e `&[&'static str]` serve como `&[&'a str]`. Foi o que aconteceu com `maior(&fixos)`.

Quase tudo é covariante: referências compartilhadas, `Box`, `Vec`, `Option`, tuplas, arrays, structs feitas desses campos. Faz sentido: se você só **lê** um valor, tratar uma referência longa como curta só faz você usá-la por menos tempo.

## Invariante: atrás de `&mut`, nada muda

Agora o caso que quebra:

```rust
// ❌ NÃO COMPILA
fn adicionar<'a>(v: &mut Vec<&'a str>, s: &'a str) {
    v.push(s);
}

fn main() {
    let mut nomes: Vec<&'static str> = vec!["ana"];
    {
        let local = String::from("bia");
        adicionar(&mut nomes, &local);
    }
    println!("{nomes:?}");
}
```

```text
error[E0597]: `local` does not live long enough
  --> src/main.rs:9:31
   |
 6 |     let mut nomes: Vec<&'static str> = vec!["ana"];
   |                    ----------------- type annotation requires that `local` is borrowed for `'static`
 7 |     {
 8 |         let local = String::from("bia");
   |             ----- binding `local` declared here
 9 |         adicionar(&mut nomes, &local);
   |                               ^^^^^^ borrowed value does not live long enough
10 |     }
   |     - `local` dropped here while still borrowed
```

Se `&mut Vec<&'static str>` pudesse virar `&mut Vec<&'a str>`, com `'a` sendo o bloco interno, `adicionar` empurraria `&local` para dentro de um vetor que **continua** dizendo que só tem `&'static str`. Fechado o bloco, `local` morre, e o `println!` leria memória liberada. É o *dangling pointer* clássico do C++, e o compilador o bloqueia proibindo o encolhimento.

Por isso `&mut T` é **invariante** em `T`: o `T` atrás de um `&mut` tem de ser **exatamente** o esperado, nem mais longo, nem mais curto. A razão é que por um `&mut` você pode **escrever**. Ler pede "pelo menos tão bom quanto"; escrever pede "exatamente igual".

O compilador então faz a única coisa possível: como `nomes` é `Vec<&'static str>` e não pode encolher, `'a` vira `'static`, e `&local` teria de viver para sempre. É o que diz *"type annotation requires that `local` is borrowed for `'static`"*.

O mesmo acontece dentro de uma função, com a nota um pouco diferente:

```rust
// ❌ NÃO COMPILA
fn registrar(nomes: &mut Vec<&'static str>) {
    let local = String::from("bia");
    adicionar(nomes, &local);
}
```

```text
error[E0597]: `local` does not live long enough
 --> src/main.rs:7:22
  |
6 |     let local = String::from("bia");
  |         ----- binding `local` declared here
7 |     adicionar(nomes, &local);
  |     -----------------^^^^^^-
  |     |                |
  |     |                borrowed value does not live long enough
  |     argument requires that `local` is borrowed for `'static`
8 | }
  | - `local` dropped here while still borrowed
```

As correções dependem do que você queria:

```rust
// ✅ 1. O vetor é dono dos textos: nada de lifetime
fn registrar(nomes: &mut Vec<String>) {
    nomes.push(String::from("bia"));
}

// ✅ 2. Não anote 'static se o vetor não precisa disso
let mut curtos: Vec<&str> = vec!["ana"];   // o compilador escolhe um 'a curto
adicionar(&mut curtos, &local);           // e "ana" (que é 'static) cabe nele
```

Na versão 2, o `"ana"` entra no vetor por covariância (um `&'static str` virando `&'a str` **antes** de ir para trás do `&mut`), e o vetor inteiro passa a ter o lifetime de `local`.

> [!NOTE]
> Em Java, arrays são covariantes **com escrita**: um `String[]` passa por `Object[]`, e guardar um `Integer` nele compila e explode em tempo de execução com `ArrayStoreException`. O problema é o mesmo deste erro, e o Rust resolve na compilação com a invariância.

O mesmo vale para tudo que permite escrever através de uma referência compartilhada: `Cell<T>`, `RefCell<T>` e `Mutex<T>` são invariantes em `T` (a mutabilidade interior do Módulo 10 é escrita, só que por `&`).

### A tabela

A [Reference](https://doc.rust-lang.org/reference/subtyping.html) lista a variância dos tipos básicos. As linhas que importam no dia a dia:

| Tipo | em `'a` | em `T` |
|---|---|---|
| `&'a T` | covariante | covariante |
| `&'a mut T` | covariante | **invariante** |
| `Box<T>`, `Vec<T>`, `[T]`, `Option<T>` | | covariante |
| `Cell<T>`, `RefCell<T>`, `UnsafeCell<T>` | | **invariante** |
| `fn() -> T` | | covariante |
| `fn(T)` | | **contravariante** |
| `*const T` | | covariante |
| `*mut T` | | **invariante** |
| `PhantomData<T>` | | covariante |
| `dyn Trait<T> + 'a` | covariante | invariante |

Repare que `&'a mut T` é covariante **em `'a`**: o empréstimo em si pode encolher (usar um `&mut` por menos tempo é seguro). O que não pode mudar é o `T` lá dentro.

*Contravariante* é o caso ao contrário: uma função que aceita `&'a str` (qualquer referência curta) serve onde se espera uma que aceita `&'static str`, porque quem aceita o mais geral aceita o particular. Isso quase nunca aparece fora de ponteiros de função e do `PhantomData`.

Para as **suas** structs, você não escreve a variância: o compilador a deduz dos campos. Se um parâmetro aparece num campo covariante e em outro invariante, ele fica invariante.

## `T: 'a`: "T não contém nada mais curto que `'a`"

O Módulo 03 mostrou `T: 'static` ("`T` não guarda referências temporárias"). A forma geral é `T: 'a`: todas as referências dentro de `T` vivem pelo menos `'a`. Ela aparece quando você guarda um `T` genérico num lugar que tem lifetime:

```rust
// ❌ NÃO COMPILA
use std::fmt::Debug;

fn empacotar<'a, T: Debug>(x: T) -> Box<dyn Debug + 'a> {
    Box::new(x)
}
```

```text
error[E0309]: the parameter type `T` may not live long enough
 --> src/main.rs:4:5
  |
3 | fn empacotar<'a, T: Debug>(x: T) -> Box<dyn Debug + 'a> {
  |              -- the parameter type `T` must be valid for the lifetime `'a` as defined here...
4 |     Box::new(x)
  |     ^^^^^^^^^^^ ...so that the type `T` will meet its required lifetime bounds
  |
help: consider adding an explicit lifetime bound
  |
3 | fn empacotar<'a, T: Debug + 'a>(x: T) -> Box<dyn Debug + 'a> {
  |                           ++++
```

`Box<dyn Debug + 'a>` promete "o que está aqui dentro vale por `'a`". Se `T` fosse `&'curto str`, a promessa seria falsa. O `help:` está certo: `T: Debug + 'a`. Com ele, `empacotar(&texto)` funciona e devolve uma caixa que vive o tanto que `texto` vive.

Sem o `+ 'a` no tipo de retorno, o erro é outro, porque `Box<dyn Trait>` tem um `'static` escondido (o [Módulo 06](../06-traits-e-generics/06-trait-objects-e-dyn.md) avisou):

```text
error[E0310]: the parameter type `T` may not live long enough
 --> src/main.rs:4:5
  |
3 | fn empacotar<T: Debug>(x: T) -> Box<dyn Debug> {
  |                                     --------- this `dyn Trait` has an implicit `'static` lifetime bound
4 |     Box::new(x)
  |     ^^^^^^^^^^^
  |     |
  |     the parameter type `T` must be valid for the static lifetime...
  |     ...so that the type `T` will meet its required lifetime bounds
```

Aí você escolhe: `T: 'static` (só aceita valores sem empréstimos, como exigem `thread::spawn` e `tokio::spawn`) ou um `'a` explícito no `dyn` (aceita empréstimos, e a caixa fica presa a eles).

Em structs, o `T: 'a` é deduzido. `struct Visao<'a, T> { itens: &'a [T] }` implica `T: 'a` sem você escrever (desde a edition 2018). Por isso você raramente vê esse bound em código novo, a não ser em funções com `dyn` ou em traits.

## `'a: 'b`: um lifetime que cobre outro

A mesma notação vale entre lifetimes: `'longo: 'curto` quer dizer "`'longo` vive pelo menos tanto quanto `'curto`". Serve quando uma função mistura dados de origens diferentes e devolve o mais curto:

```rust
/// `'longo: 'curto`: "'longo vive pelo menos tanto quanto 'curto".
fn primeiro_ou<'curto, 'longo: 'curto>(v: &'longo [String], padrao: &'curto str) -> &'curto str {
    v.first().map(String::as_str).unwrap_or(padrao)
}
```

Na prática, a covariância costuma resolver sozinha: com um lifetime só (`fn primeiro_ou<'a>(v: &'a [String], padrao: &'a str) -> &'a str`), o compilador encolhe os dois argumentos para o menor. O `'a: 'b` explícito fica para quando os dois lifetimes precisam continuar distintos, por exemplo numa struct com dois parâmetros de lifetime ou atrás de um `&mut` (onde não dá para encolher).

## `PhantomData` e variância

A lição 03 disse que o compilador recusa um parâmetro sem uso porque não saberia a variância. O `PhantomData` dá a resposta, e **qual** `PhantomData` você escreve escolhe a variância e outros efeitos:

| Campo | Variância em `T` | `Send`/`Sync` do seu tipo | Quando usar |
|---|---|---|---|
| `PhantomData<T>` | covariante | só se `T` for | o tipo é, logicamente, **dono** de um `T` (uma coleção que guarda `T` por ponteiro cru) |
| `PhantomData<fn() -> T>` | covariante | sempre | o tipo só **produz** ou **marca** `T`: ids, typestate, unidades |
| `PhantomData<&'a T>` | covariante | só se `T: Sync` | o tipo empresta um `T` por `'a` (um iterador sobre ponteiros crus) |
| `PhantomData<fn(T)>` | contravariante | sempre | o tipo **consome** `T` (raro) |
| `PhantomData<fn(T) -> T>` ou `PhantomData<Cell<T>>` | invariante | depende | o tipo permite escrever `T` |
| `PhantomData<*const T>` | covariante | nunca | quer explicitamente **não** ser `Send`/`Sync` |

A coluna do meio é a que mais morde. Com `PhantomData<T>`, o id herda o `Send` de `T`:

```rust
// ❌ NÃO COMPILA
struct Id<T> {
    valor: u64,
    _marca: PhantomData<T>,
}

type Sessao = Rc<String>;

fn main() {
    let id: Id<Sessao> = Id { valor: 1, _marca: PhantomData };
    thread::spawn(move || mostrar(id)).join().unwrap();
}
```

```text
error[E0277]: `Rc<String>` cannot be sent between threads safely
   --> src/main.rs:14:19
    |
 14 |     thread::spawn(move || mostrar(id)).join().unwrap();
    |     ------------- -------^^^^^^^^^^^^
    |     |             |
    |     |             `Rc<String>` cannot be sent between threads safely
    |     |             within this `{closure@src/main.rs:14:19: 14:26}`
    |     required by a bound introduced by this call
    |
    = help: within `{closure@src/main.rs:14:19: 14:26}`, the trait `Send` is not implemented for `Rc<String>`
note: required because it appears within the type `PhantomData<Rc<String>>`
    ...
note: required because it appears within the type `Id<Rc<String>>`
```

O id é só um `u64`, mandar para outra thread é perfeitamente seguro, mas o `PhantomData<Rc<String>>` diz ao compilador que o `Id` "tem" um `Rc`. Trocando para `PhantomData<fn() -> T>`, o mesmo código compila: um ponteiro de função é sempre `Send` e `Sync`, qualquer que seja `T`. É por isso que o `Id` do projeto [`codigo/typestate-e-builder`](codigo/typestate-e-builder/src/lib.rs) usa essa forma.

> [!NOTE]
> `PhantomData<T>` também avisa o compilador de que, ao descartar o seu tipo, um `T` pode ser descartado junto (a análise chamada *drop check*). Isso só importa para quem escreve coleções com ponteiros crus e `unsafe`, assunto do [Módulo 15](../15-unsafe-e-ffi/README.md) e do capítulo sobre `PhantomData` do Nomicon.

Código completo em [`codigo/variancia-e-hrtb`](codigo/variancia-e-hrtb/src/main.rs).

## Armadilhas comuns

- **Anotar `'static` num `Vec<&str>` "para garantir".** Atrás de um `&mut`, ele não encolhe mais, e qualquer empréstimo local vira E0597. Deixe a inferência escolher, ou guarde `String`.
- **Seguir o `help:` de `'static` sem pensar.** Num E0310, o `T: 'static` sugerido proíbe passar referências. Se você quer aceitá-las, ponha um `'a` no `dyn`.
- **`PhantomData<T>` num tipo que só marca `T`.** Ele herda `Send`, `Sync` e o *drop check* de `T`. Para ids e estados, use `PhantomData<fn() -> T>`.
- **Achar que `Cell<&'static str>` encolhe como `&'static str`.** Mutabilidade interior é escrita: `Cell`, `RefCell` e `Mutex` são invariantes.

## Resumo

- Lifetimes têm subtipagem: `'static` serve como qualquer `'a`. É a única subtipagem do Rust.
- Covariância (`&T`, `Box`, `Vec`, `Option`): o lifetime de dentro pode encolher. Invariância (`&mut T`, `Cell`, `RefCell`): tem de ser exatamente igual, porque dá para escrever.
- O erro típico de invariância é um E0597 dizendo que algo precisa ser emprestado por `'static`.
- `T: 'a` diz que `T` não tem referências mais curtas que `'a`; aparece com `Box<dyn Trait + 'a>`. Sem o `'a`, o `dyn` exige `'static` (E0310).
- `'a: 'b` liga dois lifetimes; quase sempre a covariância dispensa.
- O tipo dentro do `PhantomData` escolhe variância e `Send`/`Sync`: `fn() -> T` para marcas.

## Exercícios

Veja o exercício 4 em [`exercicios.md`](exercicios.md).

## Leitura complementar

- [The Rust Reference: Subtyping and Variance](https://doc.rust-lang.org/reference/subtyping.html)
- [The Rustonomicon: Subtyping and Variance](https://doc.rust-lang.org/nomicon/subtyping.html) e [PhantomData](https://doc.rust-lang.org/nomicon/phantom-data.html)
- [pretzelhammer: Common Rust Lifetime Misconceptions](https://github.com/pretzelhammer/rust-blog/blob/master/posts/common-rust-lifetime-misconceptions.md)
- *Rust for Rustaceans*, de Jon Gjengset, cap. 1 (seção sobre variância)

---

[← Lição anterior](06-tipos-associados-avancados-e-gats.md) · [Índice do módulo](README.md) · [Próxima lição →](08-hrtb-e-captura-em-impl-trait.md)

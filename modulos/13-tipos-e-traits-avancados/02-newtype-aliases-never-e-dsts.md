# 02 — Newtype, aliases, o tipo `!` e tipos sem tamanho

**Objetivos.** Ao final, você vai saber:

- usar newtypes para que o compilador impeça misturar valores de mesmo formato (metros e pés, ids diferentes);
- diferenciar um newtype de um *alias* de tipo, e quando usar cada um;
- ler e escrever funções que devolvem `!` (o tipo "nunca");
- explicar o que são tipos sem tamanho (`str`, `[T]`, `dyn Trait`), o *fat pointer* e o `?Sized`.

## Newtype: o mesmo `f64`, tipos diferentes

Em 1999, a sonda Mars Climate Orbiter se perdeu porque um módulo produzia impulso em libra-força·segundo e outro esperava newton·segundo. Os dois eram números de ponto flutuante, e nenhum compilador tinha como notar.

O *newtype* (Módulo 04) resolve isso em Rust sem custo nenhum: uma struct de um campo só, que na memória é exatamente o campo, mas para o compilador é outro tipo.

```rust
// ❌ NÃO COMPILA
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
struct Metros(f64);
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
struct Pes(f64);

fn altitude_segura(a: Metros) -> bool {
    a.0 > 300.0
}

fn main() {
    let leitura = Pes(1000.0);
    println!("{}", altitude_segura(leitura));
}
```

```text
error[E0308]: mismatched types
  --> src/main.rs:12:36
   |
12 |     println!("{}", altitude_segura(leitura));
   |                    --------------- ^^^^^^^ expected `Metros`, found `Pes`
   |                    |
   |                    arguments to this function are incorrect
   |
note: function defined here
  --> src/main.rs:6:4
   |
 6 | fn altitude_segura(a: Metros) -> bool {
   |    ^^^^^^^^^^^^^^^ ---------
```

A conversão passa a ser explícita, num lugar só:

```rust
// ✅
impl From<Pes> for Metros {
    fn from(p: Pes) -> Metros {
        Metros(p.0 * 0.3048)
    }
}

let m = Metros::from(Pes(1000.0));
println!("{m:?}, segura? {}", altitude_segura(m));
```

```text
Pes(1000.0) = Metros(304.8), segura? true
```

Você já viu dois outros usos do newtype: contornar a *orphan rule* ([Módulo 06](../06-traits-e-generics/02-definindo-e-implementando-traits.md)) e garantir uma invariante com um construtor que valida, como a `Port` do [Módulo 04](../04-structs-enums-e-patterns/08-estados-invalidos-irrepresentaveis.md). Os três se resumem a uma ideia: um tipo novo carrega uma informação que o tipo de dentro não tem.

O preço é que o newtype não herda nada: `Metros` não tem `+`, nem `Display`, nem os métodos de `f64`. Você implementa o que fizer sentido (a lição 01 mostrou como fazer o `+`) e deixa de fora o que não faz. `Metros * Metros` daria metros quadrados, e não metros. Não implementar é uma decisão de projeto, não um descuido.

> [!NOTE]
> Em C++, o equivalente é uma `struct Metros { double valor; };` com `explicit` no construtor. Funciona, mas cada operador precisa ser escrito à mão, e é por isso que existem bibliotecas como a `mp-units`. Em Python, `typing.NewType("Metros", float)` cria um tipo distinto **só para o verificador** (mypy, pyright). Em tempo de execução, `Metros(3.0)` é um `float` qualquer.

## Alias: só um nome mais curto

Um *alias* de tipo dá outro nome a um tipo que já existe. Não cria tipo novo:

```rust
type Indice = HashMap<String, Vec<usize>>;
type Resultado<T> = Result<T, String>;

fn indexar(texto: &str) -> Indice { /* ... */ }
fn porta(texto: &str) -> Resultado<u16> { /* ... */ }
```

`Indice` **é** `HashMap<String, Vec<usize>>`: você pode passar um no lugar do outro, chamar todos os métodos do `HashMap`, e o compilador mostra o nome original nas mensagens de erro. Um alias serve para encurtar nomes longos e repetidos, e não para proteger.

Se você escrevesse `type Metros = f64; type Pes = f64;`, o exemplo do começo **compilaria**, e a sonda cairia do mesmo jeito.

A biblioteca padrão usa alias para fixar o tipo de erro de um módulo: `std::io::Result<T>` é `Result<T, std::io::Error>`, e `std::fmt::Result` é `Result<(), std::fmt::Error>`. É o uso mais comum que você vai escrever também: `pub type Result<T> = std::result::Result<T, MeuErro>;` no módulo de erros do seu crate.

| | Newtype `struct Metros(f64);` | Alias `type Metros = f64;` |
|---|---|---|
| Tipo novo para o compilador? | sim | não |
| Herda métodos e traits? | não | sim (é o mesmo tipo) |
| Pode ter `impl` próprio, inclusive de trait de fora? | sim | não (seria `impl` em `f64`) |
| Custo em runtime | zero | zero |
| Para que serve | segurança e invariantes | encurtar nomes |

## O tipo `!`: expressões que nunca terminam

Algumas expressões não produzem valor nenhum, porque o fluxo nunca passa delas: `panic!`, `return`, `break`, `continue`, um `loop` sem `break`, `std::process::exit`. O tipo delas é `!`, o *never type* ("nunca").

O truque é que `!` se converte em **qualquer** tipo. É isso que deixa os braços de um `match` terem "tipos diferentes":

```rust
fn soma_validos(entradas: &[&str]) -> i64 {
    let mut total = 0;
    for e in entradas {
        let n: i64 = match e.parse() {
            Ok(n) => n,          // i64
            Err(_) => continue,  // !, que vira i64
        };
        total += n;
    }
    total
}
```

```text
válidos: 12
```

(Para `["10", "x", "5", "", "-3"]`.) O mesmo vale para `let Some(x) = opcao else { return; }` (Módulo 04): o `else` precisa ter tipo `!`, ou seja, precisa sair do bloco.

Você pode declarar uma função que nunca volta:

```rust
fn abortar(motivo: &str) -> ! {
    panic!("abortado: {motivo}")
}

let porta: u16 = match texto.parse() {
    Ok(p) => p,
    Err(_) => abortar("porta inválida"),
};
```

Se o corpo pudesse terminar normalmente, a função não compilaria. É o equivalente ao `[[noreturn]]` do C++ e ao `typing.NoReturn` (ou `Never`) do Python, com a diferença de que o Rust **verifica** a promessa.

> [!NOTE]
> Usar `!` como tipo em outras posições, como `Result<T, !>`, ainda é instável. Na versão estável, use [`std::convert::Infallible`](https://doc.rust-lang.org/std/convert/enum.Infallible.html), um enum sem variantes: como não existe valor dele, um `Result<T, Infallible>` é sempre `Ok`. É o tipo de erro de `String::from_str`, por exemplo.

## Tipos sem tamanho (DSTs)

Quase todo tipo tem um tamanho conhecido na compilação: `i32` tem 4 bytes, `Metros` tem 8, `String` tem 24 (ponteiro, capacidade, tamanho). Três famílias não têm:

- `str`: uma sequência de bytes UTF-8 de **qualquer** tamanho;
- `[T]`: uma sequência de `T` de qualquer tamanho (diferente de `[T; 3]`, que tem tamanho fixo);
- `dyn Trait`: um valor de qualquer tipo que implemente a trait (Módulo 06).

Esses são os *dynamically sized types* (DSTs). Você não consegue ter um deles numa variável, porque o compilador precisa saber quanto espaço reservar na stack:

```rust
// ❌ NÃO COMPILA
let s: str = *"oi";
```

```text
error[E0277]: the size for values of type `str` cannot be known at compilation time
 --> src/main.rs:2:9
  |
2 |     let s: str = *"oi";
  |         ^ doesn't have a size known at compile-time
  |
  = help: the trait `Sized` is not implemented for `str`
  = note: all local variables must have a statically known size
```

Por isso um DST sempre vive **atrás de um ponteiro**: `&str`, `&[T]`, `Box<dyn Trait>`, `Rc<str>`. E esse ponteiro é "gordo": guarda o endereço **e** a informação que falta. Para `str` e `[T]`, o tamanho; para `dyn Trait`, o endereço da *vtable*. Dá para medir:

```rust
use std::mem::size_of;

println!("&u8           {}", size_of::<&u8>());
println!("&str          {}", size_of::<&str>());
println!("&[u8]         {}", size_of::<&[u8]>());
println!("&dyn Display  {}", size_of::<&dyn Display>());
println!("Box<dyn Fn()> {}", size_of::<Box<dyn Fn()>>());
println!("Option<&u8>   {}", size_of::<Option<&u8>>());
println!("Metros        {}", size_of::<Metros>());
println!("PhantomData   {}", size_of::<PhantomData<String>>());
println!("()            {}", size_of::<()>());
```

```text
&u8           8
&str          16
&[u8]         16
&dyn Display  16
Box<dyn Fn()> 16
Option<&u8>   8
Metros        8
PhantomData   0
()            0
```

(Num sistema de 64 bits.) Os ponteiros para DSTs têm o dobro do tamanho de um ponteiro comum. `Option<&u8>` não gasta nada a mais, porque uma referência nunca é nula e o `None` usa o valor zero (é o *nicho* do [Módulo 04](../04-structs-enums-e-patterns/04-option.md)). `Metros` tem o tamanho do `f64`, e `PhantomData` não ocupa nada, o que vai importar na próxima lição.

### `Sized` e `?Sized` nas suas funções

A marca de "tem tamanho conhecido" é a trait `Sized`, de [`std::marker`](https://doc.rust-lang.org/std/marker/index.html). O [Módulo 06](../06-traits-e-generics/06-trait-objects-e-dyn.md) contou que todo parâmetro genérico tem um `T: Sized` implícito. Isso morde quando você escreve uma função genérica que recebe `&T`:

```rust
// ❌ NÃO COMPILA
use std::fmt::Display;

fn mostrar<T: Display>(x: &T) {
    println!("[{x}]");
}

fn main() {
    let s: &str = "oi";
    mostrar(s);
}
```

```text
error[E0277]: the size for values of type `str` cannot be known at compilation time
 --> src/main.rs:9:13
  |
9 |     mostrar(s);
  |     ------- ^ doesn't have a size known at compile-time
  |     |
  |     required by a bound introduced by this call
  |
  = help: the trait `Sized` is not implemented for `str`
note: required by an implicit `Sized` bound in `mostrar`
 --> src/main.rs:3:12
  |
3 | fn mostrar<T: Display>(x: &T) {
  |            ^ required by the implicit `Sized` requirement on this type parameter in `mostrar`
help: consider relaxing the implicit `Sized` restriction
  |
3 | fn mostrar<T: Display + ?Sized>(x: &T) {
  |                       ++++++++
```

O argumento é `&str`, então `T` é `str`, que não é `Sized`. Como a função só usa `x` através da referência, ela não precisa do tamanho de `T`. O `help:` está certo:

```rust
// ✅
fn mostrar<T: Display + ?Sized>(x: &T) -> String {
    format!("[{x}]")
}

println!("{} {} {}", mostrar("oi"), mostrar(&42), mostrar(&'z'));
```

```text
[oi] [42] [z]
```

A regra prática: se a função só usa `T` atrás de `&`, `&mut`, `Box` ou outro ponteiro, `?Sized` deixa ela aceitar também `str`, `[T]` e `dyn Trait`. É o que a std faz em `impl<T: Display + ?Sized> ToString for T`, em `Box<T: ?Sized>` e em `AsRef<T: ?Sized>`.

Uma struct sua também pode ter um DST, desde que seja o **último** campo. Ela vira um DST também, e você a cria com tamanho fixo e deixa o compilador "esquecer" o tamanho ao pôr atrás de um ponteiro:

```rust
struct Rotulado<T: ?Sized> {
    rotulo: &'static str,
    valor: T,
}

// Box<Rotulado<[i32; 3]>> vira Box<Rotulado<[i32]>>
let notas: Box<Rotulado<[i32]>> = Box::new(Rotulado {
    rotulo: "notas",
    valor: [7, 9, 10],
});
println!("{notas} ({} notas)", notas.valor.len());
```

```text
notas: [7, 9, 10] (3 notas)
```

### As coerções que você já usa

Esse "esquecer o tamanho" é uma *coerção*: uma conversão de tipo que o compilador faz sozinho, em pontos específicos (argumentos, `let` com tipo anotado, retorno). As que aparecem no dia a dia:

| De | Para | Nome |
|---|---|---|
| `&[T; N]`, `Box<[T; N]>` | `&[T]`, `Box<[T]>` | *unsizing* (array → slice) |
| `&Tipo`, `Box<Tipo>` | `&dyn Trait`, `Box<dyn Trait>` | *unsizing* (tipo → trait object) |
| `&String`, `&Vec<T>`, `&Box<T>` | `&str`, `&[T]`, `&T` | *deref coercion* (Módulo 06/10) |
| `&mut T` | `&T` | enfraquecer a referência |
| `&'static str` | `&'a str` | subtipagem de lifetimes (lição 07) |
| `!` | qualquer tipo | never |
| closure sem captura | `fn(...)` | ponteiro de função |

Coerção nunca chama código seu além de `Deref`, e nunca acontece entre tipos numéricos: `let x: f64 = 1_i32;` é erro, e a conversão é `f64::from(1_i32)` ou `1_i32 as f64`. Em C++, um construtor de um argumento sem `explicit` vira conversão implícita; em Rust, não existe esse mecanismo.

Código completo, com testes, em [`codigo/tipos-e-tamanhos`](codigo/tipos-e-tamanhos/src/main.rs).

## Armadilhas comuns

- **Alias quando você queria newtype.** `type UsuarioId = u64;` não impede passar um `PedidoId`. Se misturar seria um bug, use `struct UsuarioId(u64);` (ou o `Id<T>` da próxima lição).
- **Newtype com `pub` no campo e sem construtor.** `pub struct Email(pub String)` deixa qualquer um criar um `Email("abc")`. Se o newtype tem uma invariante, o campo fica privado (Módulo 04).
- **Brigar com o `Sized` implícito.** A mensagem *"the size for values of type `str` cannot be known"* numa função **sua** quase sempre se resolve com `?Sized`, desde que o parâmetro só seja usado atrás de um ponteiro.
- **Achar que `&str` é "um ponteiro".** São dois `usize`: endereço e tamanho. Isso importa quando você mede memória ou passa para C (Módulo 15).

## Resumo

- Newtype (`struct Metros(f64)`) cria um tipo novo com custo zero; serve para não misturar unidades e ids, impor invariantes e contornar a orphan rule.
- Alias (`type Resultado<T> = ...`) só dá outro nome; não protege nada.
- `!` é o tipo de `panic!`, `continue`, `return`, `loop` sem `break`; ele vira qualquer tipo. `fn f() -> !` promete nunca voltar. Fora disso, use `Infallible`.
- `str`, `[T]` e `dyn Trait` não têm tamanho fixo; vivem atrás de *fat pointers* (16 bytes em 64 bits).
- Parâmetros genéricos são `Sized` por padrão; `T: ?Sized` aceita DSTs quando `T` só é usado atrás de ponteiro.

## Exercícios

Veja o exercício 1 em [`exercicios.md`](exercicios.md) (o `Dinheiro` é um newtype).

## Leitura complementar

- [The Rust Programming Language, 20.3 Advanced Types](https://doc.rust-lang.org/stable/book/ch20-03-advanced-types.html)
- [The Rust Reference: Never type](https://doc.rust-lang.org/reference/types/never.html), [Dynamically Sized Types](https://doc.rust-lang.org/reference/dynamically-sized-types.html) e [Type coercions](https://doc.rust-lang.org/reference/type-coercions.html)
- [Rust API Guidelines: C-NEWTYPE](https://rust-lang.github.io/api-guidelines/type-safety.html#c-newtype)
- [Documentação de `std::marker::Sized`](https://doc.rust-lang.org/std/marker/trait.Sized.html)

---

[← Lição anterior](01-sobrecarga-de-operadores.md) · [Índice do módulo](README.md) · [Próxima lição →](03-phantomdata-e-typestate.md)

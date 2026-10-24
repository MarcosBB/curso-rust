# 02 — Definindo e implementando traits

**Objetivos.** Ao final, você vai saber:

- definir uma *trait*, implementá-la para os seus tipos e usar métodos padrão;
- implementar `Display` à mão, entendendo cada linha da receita do Módulo 05;
- por que uma trait precisa estar em escopo para os seus métodos funcionarem;
- o que a *orphan rule* proíbe e como contorná-la com um *newtype*.

## O problema: o mesmo comportamento em tipos diferentes

Um agregador de notícias tem posts e tweets, e quer mostrar um resumo de cada um. Em Python, bastaria que os dois tivessem um método `resumir`: quem chama não pergunta o tipo, só chama o método (*duck typing*). Se o objeto não tiver o método, o erro aparece na hora da execução.

Em Rust, você declara o comportamento comum numa *trait*: um conjunto de métodos que um tipo promete ter.

```rust
trait Resumo {
    fn autor(&self) -> String;

    fn resumir(&self) -> String {
        format!("(Leia mais de {}...)", self.autor())
    }
}
```

A trait tem dois métodos. `autor` termina em `;`, sem corpo: é **obrigatório**, e cada tipo precisa escrever o seu. `resumir` tem corpo: é um **método padrão**, que vale para quem não escrever um próprio. Repare que o método padrão chama o obrigatório: ele não sabe nada sobre o tipo concreto, só que `autor` existe.

Para dizer que um tipo cumpre a promessa, você escreve um bloco `impl Trait for Tipo`:

```rust
struct Post {
    titulo: String,
    autor: String,
}

struct Tweet {
    usuario: String,
}

impl Resumo for Post {
    fn autor(&self) -> String {
        self.autor.clone()
    }

    // Substitui o método padrão.
    fn resumir(&self) -> String {
        format!("{}, por {}", self.titulo, self.autor)
    }
}

impl Resumo for Tweet {
    fn autor(&self) -> String {
        format!("@{}", self.usuario)
    }
    // `resumir` vem do método padrão.
}
```

```rust
println!("{}", post.resumir());
println!("{}", tweet.resumir());
```

```text
Traits em Rust, por Ana
(Leia mais de @rustlang...)
```

O código completo está em [`codigo/resumo`](codigo/resumo/).

Um tipo pode ter quantos blocos `impl` quiser: um bloco "inerente" (`impl Post { ... }`, com os métodos próprios do tipo, como no Módulo 04) e um bloco para cada trait que implementa. Uma *unit struct* (`struct Saida;`, sem campos), que o Módulo 04 apresentou, é útil justamente aqui: um tipo que existe só para implementar uma trait.

Se faltar um método obrigatório, o compilador lista o que falta:

```rust
// ❌ NÃO COMPILA
impl Resumo for Tweet {}
```

```text
error[E0046]: not all trait items implemented, missing: `autor`
  --> src/main.rs:13:1
   |
 2 |     fn autor(&self) -> String;
   |     -------------------------- `autor` from trait
...
13 | impl Resumo for Tweet {}
   | ^^^^^^^^^^^^^^^^^^^^^ missing `autor` in implementation
```

O bloco vazio `{}` é válido quando **todos** os métodos têm padrão. É o caso de `std::error::Error`, e é por isso que, no Módulo 05, `impl Error for ArgsError {}` bastava: o `source()` padrão devolve `None`.

## Ter o método não basta: é preciso o `impl`

Esta é a diferença principal para o *duck typing* do Python e para os *concepts* do C++. Um tipo com um método chamado `resumir` não implementa `Resumo` por acaso:

```rust
// ❌ NÃO COMPILA
trait Resumo {
    fn resumir(&self) -> String;
}

struct Nota {
    texto: String,
}

impl Nota {
    fn resumir(&self) -> String {
        self.texto.clone()
    }
}

fn publicar(item: &impl Resumo) {
    println!("Publicando: {}", item.resumir());
}

fn main() {
    let n = Nota { texto: String::from("lembrete") };
    publicar(&n);
}
```

```text
error[E0277]: the trait bound `Nota: Resumo` is not satisfied
  --> src/main.rs:21:14
   |
21 |     publicar(&n);
   |     -------- ^^ unsatisfied trait bound
   |     |
   |     required by a bound introduced by this call
   |
help: the trait `Resumo` is not implemented for `Nota`
...
note: required by a bound in `publicar`
  --> src/main.rs:15:25
   |
15 | fn publicar(item: &impl Resumo) {
   |                         ^^^^^^ required by this bound in `publicar`
```

(`&impl Resumo` quer dizer "uma referência para qualquer tipo que implemente `Resumo`". É a próxima lição.)

Esse sistema se chama **nominal**: vale o que foi declarado, e não a forma do tipo. O `typing.Protocol` do Python e os concepts do C++20 são **estruturais**: basta ter os métodos certos. A vantagem do nominal é que implementar a trait é uma decisão explícita, que garante que `resumir` quer dizer a mesma coisa em todo tipo, e não só que o nome coincide. A desvantagem é ter que escrever o `impl`.

| | Python `Protocol` | Python `ABC` | C++ classe abstrata | C++20 concept | Rust trait |
|---|---|---|---|---|---|
| tipo precisa declarar? | não (estrutural) | sim (herda) | sim (herda) | não (estrutural) | sim (`impl`) |
| métodos com corpo | sim | sim | sim (`virtual` não puro) | não se aplica | sim (padrão) |
| quem verifica | mypy | runtime (ao instanciar) | compilador | compilador | compilador |
| custo em runtime | não se aplica | não se aplica | vptr em todo objeto | zero | zero (ou vtable, se pedir `dyn`) |

A última linha é a [lição 06](06-trait-objects-e-dyn.md). Por ora, basta saber que uma trait **não** acrescenta nada ao tipo na memória: `Post` continua com os mesmos campos, sem ponteiro escondido.

## `Display` à mão: a receita do Módulo 05, linha por linha

No Módulo 05 você copiou esta receita para o tipo de erro, e o Módulo 01 avisou que `{}` só funciona com tipos que implementam `Display`. Agora dá para ler cada parte:

```rust
use std::fmt;

struct Ponto {
    x: i32,
    y: i32,
}

impl fmt::Display for Ponto {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}
```

- `impl fmt::Display for Ponto`: implementa a trait `Display`, que mora no módulo `std::fmt`.
- `fn fmt(&self, ...)`: o único método obrigatório de `Display`. A assinatura precisa ser **idêntica** à da trait, por isso você copia.
- `f: &mut fmt::Formatter<'_>`: o destino da escrita. O `Formatter` guarda uma referência para onde o texto vai (uma `String`, o stdout...) e as opções de formatação (largura, alinhamento, precisão). Como guarda uma referência, ele tem um parâmetro de lifetime, e o `'_` é o *lifetime anônimo* da [lição 05 do Módulo 03](../03-borrowing-e-lifetimes/05-elisao-e-lifetime-anonimo.md): "tem um lifetime aqui, e o compilador cuida dele".
- `-> fmt::Result`: um apelido para `Result<(), fmt::Error>`. A escrita pode falhar (o destino pode recusar), e o erro sobe por aqui.
- `write!(f, ...)`: igual a `format!`, mas escreve no `Formatter` e devolve o `fmt::Result`. Por isso ele é a última expressão do corpo, sem `;`.

```rust
let p = Ponto { x: 3, y: -1 };
println!("p = {p}");
let s: String = p.to_string();
println!("{s:?}");
```

```text
p = (3, -1)
"(3, -1)"
```

O `to_string()` veio de graça: a biblioteca padrão implementa `ToString` para todo tipo que tem `Display`. A [lição 03](03-trait-bounds-where-e-impl-trait.md) mostra como (é um *blanket impl*).

> [!NOTE]
> `write!` ignora largura e alinhamento: `format!("[{p:>12}]")` imprime `[(3, -1)]`, sem os espaços. Se você quer que `{:>12}` funcione com o seu tipo, monte o texto e passe por `f.pad(...)`, que aplica as opções. Com `struct Sigla(&'static str)` e `f.pad(self.0)`, `format!("[{:>6}]", Sigla("BR"))` dá `[    BR]`. Para tipos de erro e mensagens, `write!` basta.

`Display` é a saída **para o usuário**, e por isso não tem `derive`: só você sabe como o seu tipo deve aparecer. `Debug`, a saída para quem programa, tem. As duas comparações óbvias:

```cpp
// C++: sobrecarga de operator<< para ostream
std::ostream& operator<<(std::ostream& os, const Ponto& p) {
    return os << "(" << p.x << ", " << p.y << ")";
}
```

```python
# Python: o método especial __str__
class Ponto:
    def __str__(self) -> str:
        return f"({self.x}, {self.y})"
```

A diferença para o Python é pequena: em vez de devolver uma `str` nova, o `fmt` do Rust escreve direto no destino, sem montar uma `String` intermediária.

## A trait precisa estar em escopo

Métodos de trait só podem ser chamados se a trait estiver em escopo, ou seja, importada com `use` (ou definida no próprio módulo). Este código implementa uma trait num módulo e tenta usá-la em outro:

```rust
// ❌ NÃO COMPILA
mod saida {
    pub trait Gritar {
        fn gritar(&self) -> String;
    }
    impl Gritar for str {
        fn gritar(&self) -> String { self.to_uppercase() }
    }
}

fn main() {
    println!("{}", "oi".gritar());
}
```

```text
error[E0599]: no method named `gritar` found for reference `&'static str` in the current scope
  --> src/main.rs:11:25
   |
11 |     println!("{}", "oi".gritar());
   |                         ^^^^^^ method not found in `&'static str`
   |
   = help: items from traits can only be used if the trait is in scope
help: trait `Gritar` which provides `gritar` is implemented but not in scope; perhaps you want to import it
   |
 1 + use crate::saida::Gritar;
   |
```

O motivo: dois crates diferentes podem definir traits diferentes com um método `gritar`, e as duas podem estar implementadas para `str`. Exigir o `use` deixa claro qual você quis.

É exatamente o `use std::io::Write` do minigrep. O `writeln!(stdout, ...)` chama o método `write_fmt`, que pertence à trait `io::Write`. Sem o import, o compilador dá esse mesmo erro. Acontece também com `write!` numa `String`, que usa outra trait, `std::fmt::Write`:

```rust
use std::fmt::Write; // sem isto: E0599, "cannot write into `String`"

let mut s = String::new();
write!(s, "{} + {} = {}", 1, 2, 3).unwrap();
```

Algumas traits você nunca importa (`Clone`, `Iterator`, `ToString`, `From`...): elas estão no *prelude*, a lista de nomes que todo arquivo Rust já recebe importada.

O exemplo acima mostrou outra coisa: `impl Gritar for str` implementa uma trait **sua** para um tipo **da biblioteca padrão**. Isso é permitido, e é útil para acrescentar métodos a tipos que você não controla.

## A *orphan rule*

O contrário também seria útil: implementar `Display` para `Vec<String>`, para imprimir uma lista com `{}`. Não dá:

```rust
// ❌ NÃO COMPILA
use std::fmt;

impl fmt::Display for Vec<String> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}]", self.join(", "))
    }
}
```

```text
error[E0117]: only traits defined in the current crate can be implemented for types defined outside of the crate
 --> src/main.rs:3:1
  |
3 | impl fmt::Display for Vec<String> {
  | ^^^^^^^^^^^^^^^^^^^^^^-----------
  |                       |
  |                       `Vec` is not defined in the current crate
  |
  = note: impl doesn't have any local type before any uncovered type parameters
  = note: for more information see https://doc.rust-lang.org/reference/items/implementations.html#orphan-rules
  = note: define and implement a trait or new type instead
```

A regra, chamada *orphan rule* (regra do órfão) ou *coerência*: **você pode implementar uma trait para um tipo se a trait ou o tipo for do seu crate.** `Display` e `Vec` são da std, então a implementação seria "órfã", sem um dono óbvio.

| | tipo seu | tipo de fora |
|---|---|---|
| **trait sua** | ✅ | ✅ (`impl Gritar for str`) |
| **trait de fora** | ✅ (`impl Display for Ponto`) | ❌ E0117 (`impl Display for Vec<String>`) |

O motivo é evitar conflitos. Se o seu crate e outro crate que você usa pudessem implementar `Display` para `Vec<String>`, cada um do seu jeito, qual valeria? A regra garante que existe no máximo uma implementação de cada trait para cada tipo no programa inteiro, e que acrescentar uma dependência nunca quebra o seu código por causa disso.

A mesma regra vale para implementações genéricas. Tentar "todo tipo que implementa a minha trait ganha `Display`" também é recusado, porque `T` pode ser um tipo de fora:

```text
error[E0210]: type parameter `T` must be used as an argument to some local type (e.g., `MyStruct<T>`)
 --> src/main.rs:5:6
  |
5 | impl<T> fmt::Display for T where T: Resumo {
  |      ^ uncovered type parameter
```

A regra completa tem mais detalhes (a Reference fala em "tipo local antes de qualquer parâmetro descoberto"), mas a versão "a trait ou o tipo precisa ser seu" resolve quase todos os casos.

### A saída: um *newtype*

A nota do compilador já diz o caminho: *define a new type*. Embrulhe o tipo de fora numa struct sua, de um campo só (o padrão *newtype* do Módulo 04):

```rust
struct Lista(Vec<String>);

impl fmt::Display for Lista {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}]", self.0.join(", "))
    }
}

println!("{}", Lista(vec![String::from("a"), String::from("b")])); // [a, b]
```

`Lista` é do seu crate, então a implementação é permitida. O custo em tempo de execução é zero: na memória, `Lista` é só o `Vec`. O custo está no código: `Lista` não tem os métodos de `Vec` (`push`, `len`...), e você acessa o vetor por `.0`. Há um jeito de fazer o newtype "se passar" pelo tipo de dentro, a trait `Deref`, mas ela tem ressalvas e é assunto do Módulo 10.

Python não tem nada parecido com a orphan rule: qualquer módulo pode acrescentar métodos a uma classe de outro (*monkey patching*), e o último a escrever ganha. Rust permite a versão segura disso, acrescentar **traits suas** a tipos de fora, e proíbe a perigosa.

## Armadilhas comuns

**Esperar que um método com o nome certo baste.** Rust é nominal: sem `impl Trait for Tipo`, o tipo não implementa a trait, mesmo com todos os métodos (E0277).

**Esquecer o `use` da trait.** O E0599 "method not found" com `help: ... is implemented but not in scope` quase sempre se resolve com o `use` que ele sugere. Acontece muito com `io::Write`, `fmt::Write`, `io::Read` e `io::BufRead`.

**Mudar a assinatura do método ao implementar.** O `fn fmt` (e qualquer outro método de trait) precisa ter a assinatura da trait. Trocar `&self` por `self`, por exemplo, dá `error[E0053]: method `fmt` has an incompatible type for trait`, com a assinatura esperada e a encontrada lado a lado. Na dúvida, copie a assinatura da documentação.

**Brigar com a orphan rule.** Se você precisa de uma trait de fora num tipo de fora, use um newtype. Se é uma trait sua, implemente direto.

## Resumo

- Uma trait declara métodos obrigatórios (sem corpo) e métodos padrão (com corpo, que podem chamar os obrigatórios).
- `impl Trait for Tipo` precisa escrever todos os obrigatórios (E0046). O sistema é nominal: sem o `impl`, não vale.
- `Display` à mão: `fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result` com `write!`. Ganha `to_string()` de graça.
- Para chamar métodos de uma trait, ela precisa estar em escopo (`use std::io::Write`).
- *Orphan rule*: a trait ou o tipo precisa ser do seu crate. Para traits de fora em tipos de fora, use um newtype.

## Exercícios

Veja o exercício 2 em [`exercicios.md`](exercicios.md). O exercício 6 aplica o `Display` no minigrep.

## Leitura complementar

- [The Rust Programming Language, 10.2 Traits: Defining Shared Behavior](https://doc.rust-lang.org/stable/book/ch10-02-traits.html)
- [The Rust Programming Language, 20.2 Advanced Traits](https://doc.rust-lang.org/stable/book/ch20-02-advanced-traits.html) (seção sobre o newtype para implementar traits externas)
- [Documentação de `std::fmt`](https://doc.rust-lang.org/std/fmt/index.html), em especial a seção "Formatting traits"
- [The Rust Reference: orphan rules](https://doc.rust-lang.org/reference/items/implementations.html#orphan-rules)
- [Comprehensive Rust: Traits](https://google.github.io/comprehensive-rust/methods-and-traits/traits.html)

---

[← Lição anterior](01-generics.md) · [Índice do módulo](README.md) · [Próxima lição →](03-trait-bounds-where-e-impl-trait.md)

# 03 — `PhantomData` e o padrão *typestate*

**Objetivos.** Ao final, você vai saber:

- usar um parâmetro de tipo que não aparece em nenhum campo, com `PhantomData`;
- criar ids e unidades "marcados" por tipo (`Id<Usuario>` x `Id<Pedido>`);
- modelar estados como tipos, para que uma transição inválida seja erro de compilação;
- testar que um uso errado da sua API **não** compila.

## Um id que lembra de onde veio

O newtype da lição anterior resolve `Metros` x `Pes`. Mas e quando há dezenas de entidades, cada uma com seu id? Escrever `UsuarioId`, `PedidoId`, `ProdutoId`... todos iguais, cansa. A ideia natural é um tipo genérico, com a entidade como parâmetro:

```rust
// ❌ NÃO COMPILA
struct Id<T> {
    valor: u64,
}
```

```text
error[E0392]: type parameter `T` is never used
 --> src/main.rs:1:11
  |
1 | struct Id<T> {
  |           ^ unused type parameter
  |
  = help: consider removing `T`, referring to it in a field, or using a marker such as `PhantomData`
  = help: if you intended `T` to be a const parameter, use `const T: /* Type */` instead
```

O compilador recusa um parâmetro que nenhum campo usa. Não é implicância: ele precisa saber como `Id<T>` se relaciona com `T` para decidir coisas como "`Id<T>` é `Send`?" e "um `Id<&'static str>` serve onde se espera `Id<&'a str>`?" (a *variância*, lição 07). Sem um campo, não há de onde tirar a resposta.

O `help:` dá a saída: [`PhantomData<T>`](https://doc.rust-lang.org/std/marker/struct.PhantomData.html), um tipo de `std::marker` que **ocupa zero bytes** e diz ao compilador "trate esta struct como se ela tivesse um `T`":

```rust
use std::marker::PhantomData;

struct Usuario;
struct Pedido;

struct Id<T> {
    valor: u64,
    _marca: PhantomData<T>,
}

impl<T> Id<T> {
    fn new(valor: u64) -> Self {
        Id { valor, _marca: PhantomData }
    }
}

fn buscar_pedido(id: Id<Pedido>) -> String {
    format!("pedido {}", id.valor)
}
```

O valor do campo também se escreve `PhantomData`. Agora `Id<Usuario>` e `Id<Pedido>` são tipos diferentes, embora na memória os dois sejam só um `u64`:

```rust
// ❌ NÃO COMPILA
let u: Id<Usuario> = Id::new(7);
println!("{}", buscar_pedido(u));
```

```text
error[E0308]: mismatched types
  --> src/main.rs:25:34
   |
25 |     println!("{}", buscar_pedido(u));
   |                    ------------- ^ expected `Id<Pedido>`, found `Id<Usuario>`
   |                    |
   |                    arguments to this function are incorrect
   |
   = note: expected struct `Id<Pedido>`
              found struct `Id<Usuario>`
```

`Usuario` e `Pedido` nem precisam ter campos: são só **marcas** (as structs unitárias do [Módulo 04](../04-structs-enums-e-patterns/01-structs.md)). Num programa real, elas seriam as structs de domínio que você já tem.

### A armadilha do `derive`

Um id deveria ser `Copy`, comparável e usável como chave de `HashMap`. O reflexo é derivar tudo:

```rust
// ❌ NÃO COMPILA
#[derive(Debug, Clone, Copy, PartialEq)]
struct Id<T> {
    valor: u64,
    _marca: PhantomData<T>,
}

fn main() {
    let a: Id<Usuario> = Id { valor: 7, _marca: PhantomData };
    let b = a;
    println!("{} {}", a.valor, b == a);
}
```

```text
error[E0369]: binary operation `==` cannot be applied to type `Id<Usuario>`
  --> src/main.rs:14:34
   |
14 |     println!("{} {}", a.valor, b == a);
   |                                - ^^ - Id<Usuario>
   |                                |
   |                                Id<Usuario>
   |
note: an implementation of `PartialEq` might be missing for `Usuario`
  --> src/main.rs:3:1
   |
 3 | struct Usuario;
   | ^^^^^^^^^^^^^^ must implement `PartialEq`
help: consider annotating `Usuario` with `#[derive(PartialEq)]`
```

O `derive` gera `impl<T: PartialEq> PartialEq for Id<T>`: ele põe o bound em **cada parâmetro de tipo**, sem olhar se o parâmetro é usado de verdade. O mesmo acontece com `Clone` e `Copy`. Para um id, isso é errado: comparar dois `Id<Usuario>` não depende de comparar usuários.

O `help:` (derivar `PartialEq` em `Usuario`) faz compilar, mas obriga toda entidade a ser comparável só para o id funcionar. A correção certa é escrever as implementações sem o bound, o que é curto:

```rust
// ✅
impl<T> Clone for Id<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for Id<T> {}

impl<T> PartialEq for Id<T> {
    fn eq(&self, outro: &Self) -> bool {
        self.valor == outro.valor
    }
}

impl<T> Eq for Id<T> {}

impl<T> Hash for Id<T> {
    fn hash<H: Hasher>(&self, h: &mut H) {
        self.valor.hash(h);
    }
}
```

O projeto [`codigo/typestate-e-builder`](codigo/typestate-e-builder/src/lib.rs) tem essas impls, um `Debug` que imprime `Id<Usuario>(7)` e um teste que guarda `Id<Usuario>` num `HashSet` sem `Usuario` implementar nada. Lá, o campo é `PhantomData<fn() -> T>` em vez de `PhantomData<T>`; a lição 07 explica a diferença (em resumo: o id não é **dono** de um `T`).

> [!NOTE]
> Em C++, o mesmo truque se chama *tag type* ou *phantom type*: `template <typename Tag> struct Id { uint64_t valor; };`. O C++ aceita parâmetros de template que não são usados, então não há um `PhantomData`. Em Python, o mais próximo é `class Id(Generic[T])` com o verificador de tipos; em tempo de execução, `Id[Usuario]` e `Id[Pedido]` são a mesma classe.

## Typestate: o estado no tipo

No [Módulo 04](../04-structs-enums-e-patterns/08-estados-invalidos-irrepresentaveis.md), o pedido virou um `enum` com estados (`Cart`, `Paid`, `Shipped`), e cada transição era um método que consome `self`. Um problema ficou pendente: `ship` num carrinho compilava, e o método só ignorava a chamada em tempo de execução.

O padrão *typestate* sobe o estado do valor para o **tipo**. Em vez de um `Pedido` que pode estar em três estados, há três tipos: `Pedido<Carrinho>`, `Pedido<Pago>` e `Pedido<Enviado>`. Cada um tem só os métodos que fazem sentido nele.

```mermaid
stateDiagram-v2
    direction LR
    [*] --> Carrinho: Pedido::new()
    Carrinho --> Carrinho: adicionar(&mut self)
    Carrinho --> Pago: pagar(self)
    Pago --> Enviado: enviar(self, rastreio)
    Enviado --> [*]
```

Os estados são structs vazias, e o pedido carrega o estado num `PhantomData`:

```rust
use std::marker::PhantomData;

pub struct Carrinho;
pub struct Pago;
pub struct Enviado;

pub struct Pedido<E> {
    itens: Vec<Item>,
    total_centavos: u64,
    rastreio: Option<String>,
    _estado: PhantomData<E>,
}
```

Cada estado ganha o seu bloco `impl`. As transições consomem `self` e devolvem o pedido no estado seguinte:

```rust
impl Pedido<Carrinho> {
    pub fn new() -> Self {
        Pedido { itens: Vec::new(), total_centavos: 0, rastreio: None, _estado: PhantomData }
    }

    pub fn adicionar(&mut self, nome: &str, preco_centavos: u64) {
        self.itens.push(Item { nome: nome.to_string(), preco_centavos });
    }

    /// Carrinho vazio não pode ser pago: devolve o carrinho de volta no `Err`.
    pub fn pagar(self) -> Result<Pedido<Pago>, Pedido<Carrinho>> {
        if self.itens.is_empty() {
            return Err(self);
        }
        let total = self.itens.iter().map(|i| i.preco_centavos).sum();
        Ok(Pedido { itens: self.itens, total_centavos: total, rastreio: None, _estado: PhantomData })
    }
}

impl Pedido<Pago> {
    pub fn total_centavos(&self) -> u64 {
        self.total_centavos
    }

    pub fn enviar(self, rastreio: &str) -> Pedido<Enviado> {
        Pedido {
            itens: self.itens,
            total_centavos: self.total_centavos,
            rastreio: Some(rastreio.to_string()),
            _estado: PhantomData,
        }
    }
}

impl Pedido<Enviado> {
    pub fn rastreio(&self) -> &str {
        self.rastreio.as_deref().unwrap_or_default()
    }
}
```

Agora, a chamada que no Módulo 04 era ignorada em tempo de execução nem compila:

```rust
// ❌ NÃO COMPILA
let p = Pedido::new();
println!("{}", p.enviar("BR123"));
```

```text
error[E0599]: no method named `enviar` found for struct `Pedido<Carrinho>` in the current scope
 --> src/main.rs:5:22
  |
5 |     println!("{}", p.enviar("BR123"));
  |                      ^^^^^^ method not found in `Pedido<Carrinho>`
  |
  = note: the method was found for `Pedido<Pago>`
```

A nota final é exatamente o que você queria que alguém dissesse: "esse método existe, mas para um pedido pago".

O *ownership* faz a outra metade do trabalho. Como `enviar` consome `self`, não sobra um `Pedido<Pago>` para enviar de novo:

```rust
// ❌ NÃO COMPILA
let mut carrinho = Pedido::new();
carrinho.adicionar("livro", 4_990);
let pago = carrinho.pagar().unwrap();
let _ = pago.enviar("BR1");
let _ = pago.enviar("BR2");
```

```text
error[E0382]: use of moved value: `pago`
 --> src/main.rs:8:13
  |
6 |     let pago = carrinho.pagar().unwrap();
  |         ---- move occurs because `pago` has type `Pedido<Pago>`, which does not implement the `Copy` trait
7 |     let _ = pago.enviar("BR1");
  |                  ------------- `pago` moved due to this method call
8 |     let _ = pago.enviar("BR2");
  |             ^^^^ value used here after move
  |
note: `typestate_e_builder::pedido::Pedido::<typestate_e_builder::pedido::Pago>::enviar` takes ownership of the receiver `self`, which moves `pago`
```

Em C++, `std::move(pago)` deixaria `pago` num estado "válido, mas não especificado", e a segunda chamada compilaria. Em Rust, o valor antigo deixa de existir.

### Métodos que valem em qualquer estado

Um `impl<E>` genérico serve para o que todo estado tem. Para que cada estado tenha um nome, uma trait com uma *constante associada* (lição 06) resolve:

```rust
pub trait Estado {
    const NOME: &'static str;
}

impl Estado for Carrinho { const NOME: &'static str = "carrinho"; }
impl Estado for Pago     { const NOME: &'static str = "pago"; }
impl Estado for Enviado  { const NOME: &'static str = "enviado"; }

impl<E: Estado> Pedido<E> {
    pub fn itens(&self) -> &[Item] {
        &self.itens
    }

    pub fn status(&self) -> &'static str {
        E::NOME
    }
}
```

O fluxo completo:

```rust
let mut pedido = Pedido::new();
pedido.adicionar("livro", 4_990);
pedido.adicionar("caneta", 350);
println!("status: {}", pedido.status());

let pago = match pedido.pagar() {
    Ok(pago) => pago,
    Err(carrinho) => {
        println!("carrinho vazio: {} itens", carrinho.itens().len());
        return;
    }
};
println!("status: {}, total: {} centavos", pago.status(), pago.total_centavos());

let enviado = pago.enviar("BR123");
println!("status: {}, rastreio: {}", enviado.status(), enviado.rastreio());
```

```text
status: carrinho
status: pago, total: 5340 centavos
status: enviado, rastreio: BR123
```

Repare em `pagar`: uma transição que pode falhar devolve `Result<Pedido<Pago>, Pedido<Carrinho>>`. Se falhar, o carrinho volta inteiro no `Err`, porque `self` foi consumido e, sem isso, os itens se perderiam.

### Typestate ou enum?

Os dois têm lugar. A pergunta é: **quem sabe o estado, o compilador ou só o programa rodando?**

- Typestate funciona quando o estado segue a estrutura do código: "primeiro conecta, depois envia", "primeiro configura, depois constrói". O compilador acompanha cada variável e sabe em que estado ela está.
- Um `enum` é o certo quando o estado vem de fora, em tempo de execução: pedidos lidos de um banco de dados, uma conexão que pode cair a qualquer momento. Você não consegue escrever `Vec<Pedido<?>>` para pedidos em estados diferentes (a não ser com um `enum` por cima).

O preço do typestate é um tipo novo por estado e um pouco de repetição nas transições (o `Pedido { itens: self.itens, ... }` em cada uma). Use quando um erro de ordem for caro e fácil de cometer. A biblioteca `embedded-hal` e os *drivers* de hardware usam muito (um pino configurado como entrada não tem `set_high`), e os builders da próxima lição também.

## Testando que algo **não** compila

Como garantir, num teste, que `enviar` continua indisponível para o carrinho? Um `#[test]` não serve, porque o arquivo de testes também precisa compilar. A ferramenta é o *doctest* `compile_fail` do [Módulo 08](../08-modulos-e-crates/09-documentacao-e-doctests.md): um exemplo na documentação que o `cargo test` exige que **falhe** ao compilar.

````rust
/// Enviar um pedido que ainda é carrinho não compila:
///
/// ```compile_fail,E0599
/// use typestate_e_builder::pedido::Pedido;
/// let p = Pedido::new();
/// p.enviar("BR123"); // erro: `enviar` não existe em `Pedido<Carrinho>`
/// ```
pub struct Pedido<E> { /* ... */ }
````

O `,E0599` depois do `compile_fail` exige o código de erro certo. Sem ele, o teste passaria por qualquer motivo, até por um erro de digitação no exemplo. Doctests só rodam em crates de biblioteca (`src/lib.rs`), e é por isso que o projeto desta lição é uma lib com um `main.rs` de demonstração.

Código completo, com testes e doctests, em [`codigo/typestate-e-builder`](codigo/typestate-e-builder/src/lib.rs).

## Armadilhas comuns

- **`derive` em tipo com `PhantomData`.** `#[derive(Clone, PartialEq, Hash)]` exige o mesmo de `T`. Se `T` é só marca, implemente à mão (são poucas linhas).
- **Esquecer de devolver o valor numa transição que falha.** `fn pagar(self) -> Option<Pedido<Pago>>` perde o carrinho no `None`. Devolva `Result<Novo, Antigo>`.
- **Typestate para estado que vem de runtime.** Se você precisa guardar valores em estados diferentes na mesma coleção, ou o estado muda por um evento externo, use um `enum`.
- **Doctest `compile_fail` sem código de erro.** Ele passa por qualquer erro. Escreva `compile_fail,E0599`.

## Resumo

- `PhantomData<T>` é um campo de tamanho zero que "usa" um parâmetro de tipo, satisfazendo o E0392.
- Tipos marcados (`Id<Usuario>`, `Id<Pedido>`) impedem misturar valores de mesmo formato.
- O `derive` põe bounds em todo parâmetro; para marcas, escreva as impls sem bound.
- Typestate: um tipo por estado, transições que consomem `self`, métodos só no estado certo. Erro de ordem vira E0599, e reuso de estado vira E0382.
- `impl<E> Tipo<E>` guarda os métodos comuns. Estado vindo de runtime continua sendo `enum`.
- Doctests `compile_fail,EXXXX` testam que um uso errado não compila.

## Exercícios

Veja o exercício 2 em [`exercicios.md`](exercicios.md).

## Leitura complementar

- [Documentação de `std::marker::PhantomData`](https://doc.rust-lang.org/std/marker/struct.PhantomData.html)
- [The Rustonomicon: PhantomData](https://doc.rust-lang.org/nomicon/phantom-data.html)
- [Cliffle: The Typestate Pattern in Rust](https://cliffle.com/blog/rust-typestate/)
- [The Rust Programming Language, 18.3 Implementing an Object-Oriented Design Pattern](https://doc.rust-lang.org/stable/book/ch18-03-oo-design-patterns.html) (o mesmo blog post como `Box<dyn State>` e, no fim, como tipos)
- [rustdoc: atributos de doctests (`compile_fail`)](https://doc.rust-lang.org/rustdoc/write-documentation/documentation-tests.html#attributes)

---

[← Lição anterior](02-newtype-aliases-never-e-dsts.md) · [Índice do módulo](README.md) · [Próxima lição →](04-builder-e-idiomas.md)

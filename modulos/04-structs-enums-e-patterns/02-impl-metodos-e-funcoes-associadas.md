# 02 — `impl`: métodos e funções associadas

**Objetivos.** Ao final, você vai saber:

- escrever métodos com `&self`, `&mut self` e `self`, e o que cada receptor promete;
- criar funções associadas, como os construtores `new`;
- por que o Rust não tem `->` nem getters automáticos;
- reconhecer os erros de chamar uma função associada como método e de esquecer os parênteses.

## Dados de um lado, comportamento do outro

Em Rust, a struct declara só os dados. Os métodos ficam num bloco `impl` separado:

```rust
#[derive(Debug, Clone, PartialEq)]
struct Rect {
    width: u32,
    height: u32,
}

impl Rect {
    fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }

    fn square(side: u32) -> Self {
        Self::new(side, side)
    }

    fn area(&self) -> u32 {
        self.width * self.height
    }

    fn can_hold(&self, other: &Rect) -> bool {
        self.width > other.width && self.height > other.height
    }

    fn scale(&mut self, factor: u32) {
        self.width *= factor;
        self.height *= factor;
    }

    fn into_square(self) -> Rect {
        let side = self.width.max(self.height);
        Rect::square(side)
    }
}

fn main() {
    let mut r = Rect::new(3, 4);
    println!("área: {}", r.area());
    println!("cabe 2x2? {}", r.can_hold(&Rect::square(2)));
    r.scale(2);
    println!("{r:?}");
    let q = r.into_square();
    println!("{q:?}");
}
```

```text
área: 12
cabe 2x2? true
Rect { width: 6, height: 8 }
Rect { width: 8, height: 8 }
```

`Self` (com S maiúsculo) é um apelido para o tipo do bloco `impl`, aqui `Rect`. Código completo, com testes, em [`codigo/retangulo`](codigo/retangulo/).

## O receptor é um contrato de ownership

O primeiro parâmetro de um método diz o que ele faz com o valor. É a mesma escolha do Módulo 03 entre emprestar e mover, só que escrita de um jeito curto:

| Receptor | Abreviação de | O método... | Exemplo |
|---|---|---|---|
| `&self` | `self: &Self` | só lê | `area`, `can_hold` |
| `&mut self` | `self: &mut Self` | altera o valor no lugar | `scale` |
| `self` | `self: Self` | **consome** o valor | `into_square` |

Isso é mais informativo que o `this` do C++ ou o `self` do Python. Só de ler a assinatura, você sabe se o método pode mudar o objeto e se o objeto ainda existe depois da chamada. O C++ chega perto com métodos `const`, mas não tem como dizer "este método consome o objeto".

Os erros seguem as regras que você já conhece. Chamar `scale` numa variável sem `mut`:

```rust
// ❌ NÃO COMPILA
struct Counter {
    value: u32,
}

impl Counter {
    fn inc(&mut self) {
        self.value += 1;
    }
}

fn main() {
    let c = Counter { value: 0 };
    c.inc();
    println!("{}", c.value);
}
```

```text
error[E0596]: cannot borrow `c` as mutable, as it is not declared as mutable
  --> src/main.rs:13:5
   |
13 |     c.inc();
   |     ^ cannot borrow as mutable
   |
help: consider changing this to be mutable
   |
12 |     let mut c = Counter { value: 0 };
   |         +++
```

E usar um valor depois de um método que recebe `self`:

```rust
// ❌ NÃO COMPILA
let r = Rect { width: 3, height: 4 };
let q = r.into_square();
println!("{q:?} {r:?}");
```

```text
error[E0382]: borrow of moved value: `r`
  --> src/main.rs:17:22
   |
15 |     let r = Rect { width: 3, height: 4 };
   |         - move occurs because `r` has type `Rect`, which does not implement the `Copy` trait
16 |     let q = r.into_square();
   |               ------------- `r` moved due to this method call
17 |     println!("{q:?} {r:?}");
   |                      ^ value borrowed here after move
   |
note: `Rect::into_square` takes ownership of the receiver `self`, which moves `r`
  --> src/main.rs:8:20
   |
 8 |     fn into_square(self) -> Rect {
   |                    ^^^^
```

A nota aponta direto para o `self` na assinatura. Métodos que consomem costumam ter nomes com `into_` (`into_square`, `String::into_bytes`). Use `self` quando o método transforma o valor em outro, ou quando o valor não deve mais ser usado depois (fechar uma conexão, por exemplo).

## Sem `->`: o ponto faz o empréstimo para você

Em C++, você escolhe entre `obj.metodo()` e `ptr->metodo()`. Em Rust só existe o ponto. O compilador acrescenta `&`, `&mut` ou `*` conforme o receptor do método (*auto-referencing*). Estas duas linhas são equivalentes:

```rust
println!("área: {}", r.area());
println!("área (forma explícita): {}", Rect::area(&r));
```

A forma explícita ajuda a entender alguns erros de empréstimo: `r.scale(2)` é, na verdade, `Rect::scale(&mut r, 2)`, e é por isso que ele exige `let mut r`.

## Funções associadas e construtores

Funções dentro do `impl` que **não** recebem `self` são *funções associadas*. São o equivalente dos métodos estáticos do C++ ou de um `@classmethod` do Python, e são chamadas com `Tipo::funcao()`:

```rust
let r = Rect::new(3, 4);
let s = Rect::square(5);
```

`new` é só uma convenção, não uma palavra reservada. Rust não tem construtores: tem funções comuns que devolvem `Self`. Por isso, é normal um tipo ter vários "construtores" com nomes que explicam o que fazem (`Rect::square`, `String::with_capacity`, `Vec::from`). A sobrecarga de construtores do C++ vira nomes diferentes.

Uma convenção que vale seguir desde já: **`new` não deve falhar**. Se a criação pode dar errado (um valor fora da faixa, um arquivo que não existe), dê outro nome, como `build`, `try_new` ou `parse`, e devolva um `Option` ou um `Result`. A lição 08 faz isso com `Option`, e o Módulo 05 com `Result`.

Chamar uma função associada como se fosse método é um erro comum, e o compilador explica bem:

```rust
// ❌ NÃO COMPILA
let r = Rect { width: 1, height: 2 };
let s = r.square(3);
```

```text
error[E0599]: no method named `square` found for struct `Rect` in the current scope
  --> src/main.rs:14:15
   |
 1 | struct Rect {
   | ----------- method `square` not found for this struct
...
14 |     let s = r.square(3);
   |               ^^^^^^ this is an associated function, not a method
   |
   = note: found the following associated functions; to be used as methods, functions must have a `self` parameter
...
help: use associated function syntax instead
   |
14 -     let s = r.square(3);
14 +     let s = Rect::square(3);
   |
```

## Vários blocos `impl`

Um tipo pode ter quantos blocos `impl` você quiser:

```rust
impl Rect {
    fn is_square(&self) -> bool {
        self.width == self.height
    }
}
```

Por enquanto isso serve só para organizar. Com generics e traits (Módulo 06), cada bloco pode ter condições diferentes, e aí a separação vira essencial.

## Getters: só quando precisa

Rust não gera getters nem setters. Se o campo é `pub`, quem está fora do módulo lê e escreve direto. O getter aparece quando o campo é **privado** e você quer deixar leitura, mas não escrita:

```rust
mod net {
    pub struct Endpoint {
        pub host: String,
        port: u16,
    }

    impl Endpoint {
        pub fn new(host: &str, port: u16) -> Endpoint {
            Endpoint { host: host.to_string(), port }
        }

        // getter: mesmo nome do campo, sem prefixo `get_`
        pub fn port(&self) -> u16 {
            self.port
        }
    }
}
```

A convenção da API Guidelines é chamar o getter pelo nome do campo (`port()`, e não `get_port()`). Getters de campos que não são `Copy` devolvem uma referência, como `fn name(&self) -> &str`.

## Armadilhas comuns

**Esquecer os parênteses.** `r.area` (sem `()`) tenta ler um campo:

```text
error[E0615]: attempted to take value of method `area` on type `Rect`
  --> src/main.rs:14:22
   |
14 |     println!("{}", r.area);
   |                      ^^^^ method, not a field
   |
help: use parentheses to call the method
   |
14 |     println!("{}", r.area());
   |                          ++
```

**Escolher `self` por engano.** Se o método só precisa ler, use `&self`. Um `self` desnecessário obriga quem chama a clonar ou a perder o valor.

**Esperar herança.** Não existe `class Square : public Rect`. Rust usa composição (uma struct dentro da outra) e traits para comportamento compartilhado. O Módulo 06 discute isso em detalhe.

## Resumo

- Os dados ficam na `struct`; o comportamento, em um ou mais blocos `impl`.
- `&self` lê, `&mut self` altera, `self` consome. A assinatura documenta o contrato.
- Funções associadas não têm `self` e são chamadas com `Tipo::f()`. `new` é convenção e não deve falhar.
- O ponto faz o empréstimo automaticamente: não existe `->`.
- Getters só para campos privados, com o nome do campo e sem `get_`.

## Exercícios

Os exercícios 2 e 6 em [`exercicios.md`](exercicios.md) pedem métodos e funções associadas.

## Leitura complementar

- [The Rust Programming Language, 5.3 Method Syntax](https://doc.rust-lang.org/stable/book/ch05-03-method-syntax.html)
- [Rust API Guidelines: getters (C-GETTER)](https://rust-lang.github.io/api-guidelines/naming.html#getter-names-follow-rust-convention-c-getter)
- [Microsoft RustTraining (C/C++), cap. 5](https://microsoft.github.io/RustTraining/c-cpp-book/ch05-data-structures.html#rust-structs)

---

[← Lição anterior](01-structs.md) · [Índice do módulo](README.md) · [Próxima lição →](03-enums-algebricos.md)

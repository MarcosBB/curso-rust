# 01 — Structs

**Objetivos.** Ao final, você vai saber:

- definir e criar structs nomeadas, *tuple structs* e *unit structs*;
- usar o atalho de inicialização de campos e a sintaxe de atualização (e o *move* escondido nela);
- pedir ao compilador `Debug`, `Clone` e `PartialEq` com `#[derive]`;
- por que a mutabilidade é da variável, e não do campo.

## Agrupando dados

Uma *struct* junta valores relacionados sob um nome, com um campo para cada valor:

```rust
#[derive(Debug, Clone, PartialEq)]
struct User {
    name: String,
    email: String,
    active: bool,
    logins: u32,
}

fn main() {
    let u = User {
        name: String::from("Ana"),
        email: String::from("ana@exemplo.com"),
        active: true,
        logins: 0,
    };
    println!("{}", u.name);
    println!("{u:?}");
}
```

```text
Ana
User { name: "Ana", email: "ana@exemplo.com", active: true, logins: 0 }
```

A linha `#[derive(Debug, Clone, PartialEq)]` pede ao compilador que gere três capacidades: imprimir com `{:?}`, copiar com `.clone()` e comparar com `==`. Por enquanto, trate isso como uma receita. O que o `derive` gera são implementações de *traits*, assunto do [Módulo 06](../06-traits-e-generics/README.md).

Para quem vem de outras linguagens:

```cpp
// C++: struct com construtor implícito, herança possível, campos públicos por padrão
struct User {
    std::string name;
    std::string email;
    bool active = true;   // valor padrão no próprio campo
    unsigned logins = 0;
};
```

```python
# Python: dataclass gera __init__, __repr__ e __eq__
@dataclass
class User:
    name: str
    email: str
    active: bool = True
    logins: int = 0
```

A struct do Rust é mais enxuta que as duas: não tem herança, não tem construtor implícito, não tem valor padrão por campo e não gera nada que você não peça. O comportamento (métodos) vem separado, num bloco `impl`, assunto da [próxima lição](02-impl-metodos-e-funcoes-associadas.md).

## Todo campo precisa de um valor

Não existe construtor implícito nem campo "não inicializado". Se faltar um campo, o código não compila:

```rust
// ❌ NÃO COMPILA
struct Point {
    x: f64,
    y: f64,
}

fn main() {
    let p = Point { x: 1.0 };
    println!("{}", p.x);
}
```

```text
error[E0063]: missing field `y` in initializer of `Point`
 --> src/main.rs:7:13
  |
7 |     let p = Point { x: 1.0 };
  |             ^^^^^ missing `y`
```

Em C++, um `int` sem inicializador num `struct` fica com lixo de memória. Em Rust, isso não tem como acontecer. Quando você quiser valores padrão, a solução é uma função construtora (próxima lição) ou `#[derive(Default)]` com `..Default::default()`, que veremos no Módulo 06.

## Mutabilidade é da variável, não do campo

Não existe campo `mut`. Quem decide se a struct pode mudar é a variável que a guarda:

```rust
// ❌ NÃO COMPILA
struct Point {
    x: f64,
    y: f64,
}

fn main() {
    let p = Point { x: 1.0, y: 2.0 };
    p.x = 5.0;
    println!("{} {}", p.x, p.y);
}
```

```text
error[E0594]: cannot assign to `p.x`, as `p` is not declared as mutable
 --> src/main.rs:8:5
  |
8 |     p.x = 5.0;
  |     ^^^^^^^^^ cannot assign
  |
help: consider changing this to be mutable
  |
7 |     let mut p = Point { x: 1.0, y: 2.0 };
  |         +++
```

Com `let mut p`, todos os campos ficam alteráveis. Parece menos flexível que o `const` por membro do C++, mas combina com as regras de *borrowing* do Módulo 03: o que importa é quem tem acesso mutável ao valor inteiro naquele momento. Para deixar um campo "somente leitura" para o resto do programa, a ferramenta é a visibilidade (campo privado com um *getter*), e não a mutabilidade.

## Atalho de campos e sintaxe de atualização

Quando a variável tem o mesmo nome do campo, basta escrever o nome uma vez (*field init shorthand*):

```rust
fn new_user(name: String, email: String) -> User {
    User {
        name,  // o mesmo que name: name
        email,
        active: true,
    }
}
```

Para criar uma struct a partir de outra, mudando só alguns campos, use a *struct update syntax* (`..outra`):

```rust
let u2 = User {
    email: String::from("bia@exemplo.com"),
    ..u1
};
```

Os campos que você não escreveu vêm de `u1`. Parece o `dataclasses.replace(u1, email=...)` do Python, mas com uma diferença importante: **os campos são movidos, não copiados**. Volte ao Módulo 02: `String` não é `Copy`, então `u1.name` passa a pertencer a `u2`:

```rust
// ❌ NÃO COMPILA
let u2 = User {
    email: String::from("bia@exemplo.com"),
    ..u1
};
println!("{} {}", u1.name, u2.active);
```

```text
error[E0382]: borrow of moved value: `u1.name`
  --> src/main.rs:18:23
   |
14 |       let u2 = User {
   |  ______________-
15 | |         email: String::from("bia@exemplo.com"),
16 | |         ..u1
17 | |     };
   | |_____- value moved here
18 |       println!("{} {}", u1.name, u2.active);
   |                         ^^^^^^^ value borrowed here after move
   |
   = note: move occurs because `u1.name` has type `String`, which does not implement the `Copy` trait
```

Repare na precisão: o erro fala de `u1.name`, e não de `u1`. O *move* foi **parcial**. `u1.email` (que você substituiu) e `u1.active` (um `bool`, que é `Copy`) continuam válidos. Só o que saiu de `..u1` e não é `Copy` foi movido.

Duas correções, conforme a intenção:

```rust
// ✅ copia tudo de u1 (u1 continua intacto)
let u2 = User {
    email: String::from("bia@exemplo.com"),
    ..u1.clone()
};

// ✅ ou fornece os campos que não são Copy: nada de String sai de u1
let u3 = User {
    name: String::from("Caio"),
    email: String::from("caio@exemplo.com"),
    ..u1
};
println!("{u1:?}");  // ok: só `active`, que é Copy, veio de u1
```

## Tuple structs e o padrão *newtype*

Uma *tuple struct* tem campos sem nome, acessados por posição (`.0`, `.1`):

```rust
#[derive(Debug, Clone, Copy, PartialEq)]
struct Meters(f64);

#[derive(Debug, Clone, Copy, PartialEq)]
struct Feet(f64);

fn to_feet(m: Meters) -> Feet {
    Feet(m.0 * 3.28084)
}

fn main() {
    let altura = Meters(1.80);
    let Meters(valor) = altura; // desestruturação, como numa tupla
    println!("{valor} m = {:?}", to_feet(altura));
}
```

```text
1.8 m = Feet(5.905512)
```

Com um campo só, ela é chamada de *newtype*: um tipo novo que embrulha outro. O ganho é que `Meters` e `Feet` são tipos **diferentes**, mesmo que os dois guardem um `f64`:

```rust
// ❌ NÃO COMPILA
let f = Feet(10.0);
let _ = to_feet(f);
```

```text
error[E0308]: mismatched types
  --> src/main.rs:10:21
   |
10 |     let _ = to_feet(f);
   |             ------- ^ expected `Meters`, found `Feet`
   |             |
   |             arguments to this function are incorrect
```

Em C++, um `using Meters = double;` não impediria essa troca, porque o alias é o mesmo tipo. O newtype do Rust é um tipo de verdade e não custa nada em tempo de execução: `Meters` ocupa exatamente o espaço de um `f64`. Na lição 08, ele vai ganhar um construtor que valida o valor.

Há ainda a *unit struct*, sem campo nenhum:

```rust
#[derive(Debug)]
struct Origin;
```

Ela não guarda dados e ocupa zero bytes. Parece inútil agora, mas é comum quando o que interessa é o **tipo** e não o valor, por exemplo para implementar uma trait (Módulo 06) ou marcar estados (Módulo 13).

## Imprimindo e comparando: `derive` como receita

Uma struct recém-criada não sabe se imprimir nem se comparar. Tente:

```rust
// ❌ NÃO COMPILA
struct Point {
    x: i32,
    y: i32,
}

fn main() {
    let a = Point { x: 1, y: 2 };
    let b = Point { x: 1, y: 2 };
    println!("{a:?}");
    println!("{}", a == b);
}
```

```text
error[E0277]: `Point` doesn't implement `Debug`
 --> src/main.rs:9:15
  |
9 |     println!("{a:?}");
  |               ^^^^^ `Point` cannot be formatted using `{:?}` because it doesn't implement `Debug`
  |
  = help: the trait `Debug` is not implemented for `Point`
help: consider annotating `Point` with `#[derive(Debug)]`
  |
1 + #[derive(Debug)]
2 | struct Point {
  |

error[E0369]: binary operation `==` cannot be applied to type `Point`
  --> src/main.rs:10:22
   |
10 |     println!("{}", a == b);
   |                    - ^^ - Point
   |                    |
   |                    Point
   |
...
help: consider annotating `Point` with `#[derive(PartialEq)]`
```

O próprio compilador dá a receita. Neste módulo, use estas três:

| `derive` | O que você ganha | Equivalente em Python |
|---|---|---|
| `Debug` | `{:?}`, `{:#?}` (formatado em várias linhas) e `dbg!` | `__repr__` do `dataclass` |
| `Clone` | `.clone()`, uma cópia profunda e explícita | `copy.deepcopy` |
| `PartialEq` | `==` e `!=`, comparando campo a campo | `__eq__` do `dataclass` |

A diferença para o `@dataclass` é que o Rust não gera nada por conta própria: o tipo só ganha o que você pede. O `{}` (sem `?`) exige outra trait, `Display`, que você implementa à mão no Módulo 06.

A macro `dbg!` é ótima para depurar: imprime arquivo, linha, a expressão e o valor, e devolve o valor:

```rust
let p = Point { x: 1, y: 2 };
let soma = dbg!(p.x + p.y);
dbg!(&p);
```

```text
[src/main.rs:9:16] p.x + p.y = 3
[src/main.rs:10:5] &p = Point {
    x: 1,
    y: 2,
}
```

Passe `&p` e não `p`, senão o `dbg!` toma posse do valor.

## Visibilidade dos campos

Por padrão, os campos de uma struct são **privados ao módulo** onde ela foi definida. Fora dele, só os campos marcados com `pub` podem ser lidos ou escritos:

```rust
mod net {
    pub struct Endpoint {
        pub host: String, // visível fora do módulo
        port: u16,        // só o código dentro de `net` enxerga
    }
}
```

Enquanto todo o seu código estiver num arquivo só, isso não faz diferença. Mas é a base de uma técnica importante, que aparece na lição 08: garantir que um valor sempre foi validado, porque a única forma de criá-lo é por uma função que valida. Módulos em detalhe ficam para o Módulo 08.

## Armadilhas comuns

**Struct update move.** `..u1` move os campos que não são `Copy`. Se você ainda precisa de `u1`, use `..u1.clone()` ou forneça esses campos explicitamente.

**Esquecer o `derive`.** E0277 em `{:?}` e E0369 em `==` quase sempre se resolvem com `#[derive(Debug)]` e `#[derive(PartialEq)]`. Não tente usar `{}`: ele exige `Display`.

**"Fields are never read".** Se um campo só aparece no `{:?}`, o compilador avisa:

```text
warning: fields `name`, `email`, and `active` are never read
...
  = note: `User` has derived impls for the traits `Clone` and `Debug`, but these are intentionally ignored during dead code analysis
```

É só um aviso: o código gerado pelo `derive` não conta como "uso". Em código de verdade, os campos são lidos em algum lugar e o aviso some.

**`mut` no campo.** `struct Point { mut x: f64 }` é erro de sintaxe (`expected '(', found 'x'`). A mutabilidade vai em `let mut p`.

## Resumo

- Struct nomeada (`struct User { name: String }`), tuple struct (`struct Meters(f64);`) e unit struct (`struct Origin;`).
- Todo campo precisa ser inicializado; não há construtor nem valor padrão implícitos.
- A mutabilidade é da variável (`let mut p`), não do campo.
- `..outra` preenche o resto a partir de outra struct e **move** os campos que não são `Copy`.
- Newtypes criam tipos distintos sem custo em tempo de execução.
- `#[derive(Debug, Clone, PartialEq)]` é a receita para imprimir, copiar e comparar (traits no Módulo 06).

## Exercícios

Esta lição é base para todos os exercícios em [`exercicios.md`](exercicios.md). O exercício 2 usa tuple structs e campos privados.

## Leitura complementar

- [The Rust Programming Language, 5.1 Defining and Instantiating Structs](https://doc.rust-lang.org/stable/book/ch05-01-defining-structs.html)
- [The Rust Programming Language, 5.2 An Example Program Using Structs](https://doc.rust-lang.org/stable/book/ch05-02-example-structs.html)
- [Rust by Example: Structures](https://doc.rust-lang.org/rust-by-example/custom_types/structs.html)
- [Microsoft RustTraining (Python), structs x classes](https://microsoft.github.io/RustTraining/python-book/ch05-data-structures-and-collections.html#structs-vs-classes)

---

[Índice do módulo](README.md) · [Próxima lição →](02-impl-metodos-e-funcoes-associadas.md)

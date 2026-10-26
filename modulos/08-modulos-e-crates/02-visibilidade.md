# 02 — Visibilidade

**Objetivos.** Ao final, você vai saber:

- a regra de quem enxerga um item privado (e por que a fronteira é o módulo, não o tipo);
- usar `pub(crate)`, `pub(super)` e `pub(in caminho)`;
- como a visibilidade funciona em campos de structs e em enums;
- por que começar tudo privado.

## A regra: o módulo e os descendentes

Um item privado é visível no módulo em que foi definido **e em todos os módulos dentro dele**. O contrário não vale: o pai não enxerga o privado do filho.

```rust
// ❌ NÃO COMPILA
mod loja {
    pub mod estoque {
        fn contar_interno() -> u32 {
            3
        }

        pub fn total() -> u32 {
            contar_interno() + super::taxa() // ✅ o filho vê o privado do pai
        }
    }

    fn taxa() -> u32 {
        1
    }

    pub fn resumo() -> u32 {
        estoque::contar_interno() // ❌ o pai não vê o privado do filho
    }
}

fn main() {
    println!("{} {}", loja::estoque::total(), loja::resumo());
}
```

```text
error[E0603]: function `contar_interno` is private
  --> src/main.rs:17:18
   |
17 |         estoque::contar_interno() // ❌ o pai não vê o privado do filho
   |                  ^^^^^^^^^^^^^^ private function
   |
note: the function `contar_interno` is defined here
  --> src/main.rs:3:9
   |
 3 |         fn contar_interno() -> u32 {
   |         ^^^^^^^^^^^^^^^^^^^^^^^^^^
```

Pense numa casa: quem está num cômodo interno enxerga os objetos da sala, mas quem está na sala não vê o que está guardado nos quartos. É por isso que o `mod tests` com `use super::*` consegue testar funções privadas ([M01-10](../01-tour-da-sintaxe/10-kit-testes.md)): ele é filho do módulo testado. E é por isso que `rede::tcp`, na lição anterior, chama a função privada `rede::host_valido`.

## A fronteira é o módulo, não o tipo

Em C++ e Java, `private` é por classe. Em Rust, é por módulo: qualquer código do módulo onde a struct foi definida enxerga os campos privados, inclusive funções livres.

```rust
mod estoque {
    pub struct Item {
        pub nome: String,
        preco: u32,
    }

    impl Item {
        pub fn novo(nome: &str, preco: u32) -> Item {
            Item { nome: nome.to_string(), preco }
        }
    }

    /// Função livre do mesmo módulo: enxerga o campo privado.
    pub fn com_desconto(item: &Item) -> u32 {
        item.preco * 9 / 10
    }
}
```

Consequências para quem vem de C++:

- **Não existe `friend`** porque não é preciso: o que precisa de acesso especial fica no mesmo módulo.
- **Não existe `protected`** porque não existe herança (o [Módulo 06](../06-traits-e-generics/README.md) mostrou a composição no lugar dela). `pub(super)`, visto abaixo, **não** é a mesma coisa: ele abre para o módulo pai, não para "subtipos".

E para quem vem de Python: o `_nome` é uma convenção que qualquer um pode ignorar. O privado do Rust é verificado pelo compilador.

## Os níveis de visibilidade

| Visibilidade | Visível em |
|---|---|
| (nada) | o módulo atual e os descendentes |
| `pub(self)` | o mesmo que nada (útil em macros, Módulo 14) |
| `pub(super)` | o módulo pai e os descendentes dele |
| `pub(in crate::a::b)` | o módulo `crate::a::b` e os descendentes; precisa ser um ancestral |
| `pub(crate)` | a crate inteira, mas não quem usa a crate |
| `pub` | todo mundo que alcança o item |

Na prática, quase todo código usa só três: nada, `pub(crate)` e `pub`.

```rust
// ❌ NÃO COMPILA
mod loja {
    pub mod estoque {
        pub(crate) fn contar() -> u32 {
            1
        }
        pub(super) fn repor() {}
        pub struct Id(u32); // construtor privado (veja abaixo)
    }

    pub fn abrir() {
        estoque::repor(); // ✅ `loja` é o pai de `estoque`
    }
}

fn main() {
    loja::abrir();
    loja::estoque::contar(); // ✅ pub(crate): qualquer lugar da crate
    loja::estoque::repor(); // ❌ pub(super): só dentro de `loja`
}
```

```text
error[E0603]: function `repor` is private
  --> src/main.rs:18:20
   |
18 |     loja::estoque::repor(); // ❌ pub(super): só dentro de `loja`
   |                    ^^^^^ private function
   |
note: the function `repor` is defined here
  --> src/main.rs:6:9
   |
 6 |         pub(super) fn repor() {}
   |         ^^^^^^^^^^^^^^^^^^^^^
```

Repare que a mensagem diz "private function", mesmo para um item `pub(super)`: do ponto de vista de `main`, ele é privado.

`pub(in ...)` só aceita ancestrais. Restringir a um módulo irmão não faz sentido e é recusado:

```text
error[E0742]: visibilities can only be restricted to ancestor modules
 --> src/main.rs:4:16
  |
4 |         pub(in crate::caixa) fn cobrar() {}
  |                ^^^^^^^^^^^^
```

### `pub(crate)`: o "interno" de uma biblioteca

Numa biblioteca, `pub(crate)` é o nível certo para utilitários que vários módulos usam, mas que não fazem parte da API. No [`chat`](codigo/chat/src/lib.rs):

```rust
// src/lib.rs
/// Registra uma mensagem de diagnóstico. Visível em toda a crate, mas fora da API pública.
pub(crate) fn registrar(msg: &str) -> String {
    format!("[chat] {msg}")
}
```

O módulo `cliente` chama `crate::registrar(...)`. O `main.rs`, que é outra crate, não consegue.

## Structs e enums

Numa `pub struct`, cada campo tem a própria visibilidade, e o padrão é privado. Fora do módulo, você não constrói com literal nem lê o campo:

```rust
// ❌ NÃO COMPILA (usando o módulo `estoque` acima)
fn main() {
    let b = estoque::Item { nome: "lápis".into(), preco: 100 }; // ❌ E0451
}
```

```text
error[E0451]: field `preco` of struct `Item` is private
  --> src/main.rs:22:51
   |
22 |     let b = estoque::Item { nome: "lápis".into(), preco: 100 }; // ❌ E0451
   |                                                   ^^^^^ private field
```

Ler `b.preco` de fora dá o erro irmão, `E0616: field preco of struct Item is private`.

Numa *tuple struct*, um campo privado torna o **construtor** privado (`Id(3)` dá E0603). É a base do *newtype* validado da [M04-08](../04-structs-enums-e-patterns/08-estados-invalidos-irrepresentaveis.md). Lá você viu que o `help` do compilador sugere `pub u32`: resolve o erro e destrói a garantia. O compilador conhece as regras de visibilidade, não o seu design.

Já num `pub enum`, **todas as variantes e seus campos são públicos**. Não dá para esconder uma variante.

> [!NOTE]
> `#[non_exhaustive]` numa `pub struct` ou num `pub enum` impede quem está fora da crate de construir com literal ou de fazer um `match` sem `_`. Serve para poder acrescentar campos ou variantes depois sem quebrar ninguém (lição 05).

## Tipo privado numa assinatura pública

Uma função `pub` que devolve um tipo privado gera um aviso:

```rust
pub mod cofre {
    struct Segredo;

    pub fn abrir() -> Segredo {
        Segredo
    }
}
```

```text
warning: type `Segredo` is more private than the item `abrir`
 --> src/lib.rs:4:5
  |
4 |     pub fn abrir() -> Segredo {
  |     ^^^^^^^^^^^^^^^^^^^^^^^^^ function `abrir` is reachable at visibility `pub`
  |
note: but type `Segredo` is only usable at visibility `pub(self)`
 --> src/lib.rs:2:5
  |
2 |     struct Segredo;
  |     ^^^^^^^^^^^^^^
  = note: `#[warn(private_interfaces)]` on by default
```

Quem está fora recebe um valor de um tipo que não consegue nomear. Quase sempre é um esquecimento: torne o tipo `pub` ou a função menos visível.

## Comece privado

Tornar público um item privado é uma mudança compatível. O contrário quebra quem já usava (lição 05). Por isso a regra prática é: comece tudo privado, use `pub(crate)` para o que circula dentro da crate, e só marque `pub` o que você aceita manter como API.

| | C++ | Python | Rust |
|---|---|---|---|
| unidade de privacidade | a classe | (convenção `_nome`) | o módulo |
| acesso especial | `friend` | — | mesmo módulo; `pub(crate)`, `pub(super)` |
| para herdeiros | `protected` | — | não existe |
| interno à biblioteca | símbolos não exportados, `static` | `_modulo` | `pub(crate)` |

## Armadilhas comuns

- **Esperar que o pai veja o privado do filho.** É ao contrário: o filho vê o do pai.
- **Tratar `pub(super)` como `protected`.** Ele abre para o módulo pai, não para tipos "derivados".
- **`pub struct` com campos privados e construção por literal fora do módulo** (E0451). Ofereça um construtor.
- **Aceitar o `help` que torna o campo `pub`** num newtype validado. Mantenha o campo privado e crie a partir de uma função.
- **Tipo privado em função pública** (`private_interfaces`).

## Resumo

- Um item privado é visível no módulo onde foi definido e nos descendentes.
- A fronteira de encapsulamento é o módulo: funções livres do mesmo módulo veem campos privados.
- Os níveis úteis: privado, `pub(crate)` e `pub`; `pub(super)` e `pub(in ...)` para casos pontuais.
- Campos de struct são privados por padrão; variantes de enum são sempre públicas.
- Comece privado: abrir depois é compatível, fechar é quebra.

## Exercícios

Veja o exercício 2 em [`exercicios.md`](exercicios.md).

## Leitura complementar

- [The Rust Programming Language, 7.3 Paths (visibilidade)](https://doc.rust-lang.org/stable/book/ch07-03-paths-for-referring-to-an-item-in-the-module-tree.html)
- [The Rust Reference: Visibility and Privacy](https://doc.rust-lang.org/reference/visibility-and-privacy.html)
- [Comprehensive Rust: Visibility](https://google.github.io/comprehensive-rust/modules/visibility.html) e [Encapsulation](https://google.github.io/comprehensive-rust/modules/encapsulation.html)
- [Effective Rust, Item 22: Minimize visibility](https://effective-rust.com/visibility.html)

---

[← Lição anterior](01-a-arvore-de-modulos.md) · [Índice do módulo](README.md) · [Próxima lição →](03-caminhos-use-e-reexports.md)

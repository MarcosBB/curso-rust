# Exercícios — Módulo 14

Soluções em [`solucoes/14-macros`](../../solucoes/14-macros/).

Nos exercícios 1 a 4, crie o projeto com `cargo new --lib nome`, acrescente `rust-version = "1.96"` no `Cargo.toml` e cole o código inicial em `src/lib.rs`. O exercício 5 monta um workspace pequeno; os arquivos estão no enunciado.

O objetivo final é sempre o mesmo: `cargo test` passando e `CARGO_BUILD_WARNINGS=deny cargo clippy --all-targets` sem avisos. Quando travar, `cargo expand` (`cargo install cargo-expand`) mostra o que a sua macro gerou.

## 1. `media!` (fácil, faça compilar e passe nos testes)

Lições: [02](02-macro-rules.md) e [03](03-repeticoes-e-recursao.md).

`cargo new --lib media`. A macro só aceita um valor. Faça `media!` aceitar um ou mais números (inteiros ou `f64`, devolvendo `f64`), com vírgula final opcional, avaliando cada expressão **uma vez**. Extra: `media!()` deve dar um erro de compilação com a mensagem `media! precisa de pelo menos um valor` (confira à mão, numa função qualquer; não dá para testar com `#[test]`).

```rust
/// Média aritmética de um ou mais números, como `f64`.
#[macro_export]
macro_rules! media {
    ($x:expr) => {
        $x
    };
}

#[cfg(test)]
mod tests {
    #[test]
    fn um_valor() {
        assert_eq!(media!(4.0), 4.0);
    }

    #[test]
    fn varios_valores() {
        assert_eq!(media!(1.0, 2.0, 3.0), 2.0);
        assert_eq!(media!(1, 2), 1.5);
    }

    #[test]
    fn virgula_final() {
        assert_eq!(media!(10.0, 20.0,), 15.0);
    }

    #[test]
    fn expressoes_e_avaliacao_unica() {
        let mut chamadas = 0;
        let mut proximo = || {
            chamadas += 1;
            chamadas as f64
        };
        // 1.0 e 2.0: cada chamada roda uma vez.
        assert_eq!(media!(proximo(), proximo()), 1.5);
        assert_eq!(chamadas, 2);
        assert_eq!(media!(2.0 * 3.0, 1.0 + 1.0), 4.0);
    }
}
```

<details>
<summary>Dica</summary>

Uma repetição `$($x:expr),+ $(,)?` captura a lista. Para somar e contar avaliando cada expressão uma vez, ponha todas num array: `[$($x as f64),+]`. Para o extra, uma regra `() => { compile_error!("...") }`.

</details>

## 2. `maximo!` recursiva (fácil, faça compilar e passe nos testes)

Lição: [03](03-repeticoes-e-recursao.md).

`cargo new --lib maximo`. A versão inicial só aceita dois argumentos, e avalia o vencedor duas vezes. Reescreva `maximo!` para aceitar um ou mais valores de qualquer tipo `Ord`, por recursão, avaliando cada argumento uma vez só.

```rust
/// O maior de um ou mais valores que implementam `Ord`.
#[macro_export]
macro_rules! maximo {
    ($a:expr, $b:expr) => {
        if $a > $b { $a } else { $b }
    };
}

#[cfg(test)]
mod tests {
    #[test]
    fn um_valor() {
        assert_eq!(maximo!(7), 7);
    }

    #[test]
    fn varios_valores() {
        assert_eq!(maximo!(1, 5, 2), 5);
        assert_eq!(maximo!(3, 9, 4, 9, 1,), 9);
        assert_eq!(maximo!("pera", "uva", "banana"), "uva");
    }

    #[test]
    fn avalia_cada_argumento_uma_vez() {
        let mut chamadas = 0;
        let mut proximo = || {
            chamadas += 1;
            chamadas
        };
        assert_eq!(maximo!(proximo(), proximo(), proximo()), 3);
        assert_eq!(chamadas, 3);
    }
}
```

<details>
<summary>Dica</summary>

Duas regras: o caso base `($x:expr $(,)?)` e o passo `($x:expr, $($resto:expr),+ $(,)?)`, que expande para `::std::cmp::max($x, $crate::maximo!($($resto),+))`. Uma chamada de função avalia cada argumento uma vez.

</details>

## 3. Uma macro para outras crates (médio, faça compilar)

Lição: [04](04-higiene-e-escopo.md).

`cargo new --lib relatorios`. A macro `linha!` deve funcionar dentro da crate (em `resumo`) **e** num teste de integração, que é outra crate. Cole o código em `src/lib.rs` e o teste em `tests/integracao.rs`. São três problemas; o compilador mostra um de cada vez. Não mude o teste.

```rust
pub mod formato {
    pub fn moeda(valor: f64) -> String {
        format!("R$ {valor:.2}")
    }
}

pub fn resumo(total: f64) -> String {
    linha!("total", total)
}

macro_rules! linha {
    ($rotulo:expr, $valor:expr) => {
        format!("{}: {}", $rotulo, moeda($valor))
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resumo_formata() {
        assert_eq!(resumo(99.9), "total: R$ 99.90");
    }
}
```

```rust
// tests/integracao.rs
use relatorios::linha;

#[test]
fn linha_fora_da_crate() {
    assert_eq!(linha!("frete", 12.5), "frete: R$ 12.50");
}
```

<details>
<summary>Dica</summary>

Ordem textual, `#[macro_export]` e `$crate`. Cuidado com a sugestão do compilador `use crate::formato::moeda;`: ela conserta `resumo`, mas não o teste de integração.

</details>

## 4. `enum_com_nomes!` (médio, passe nos testes)

Lição: [03](03-repeticoes-e-recursao.md).

`cargo new --lib enum-com-nomes`. Escreva uma macro que recebe uma declaração de enum (com atributos, visibilidade e variantes sem dados) e gera o enum exatamente como escrito, mais um `impl` com:

- `pub const TODOS: &'static [Self]`: as variantes, na ordem;
- `pub fn nome(&self) -> &'static str`: o nome da variante;
- `pub fn de_nome(texto: &str) -> Option<Self>`: o caminho inverso.

```rust
#[macro_export]
macro_rules! enum_com_nomes {
    ($nome:ident { $($variante:ident),* }) => {
        enum $nome {
            $($variante),*
        }
    };
}

#[cfg(test)]
mod tests {
    enum_com_nomes! {
        #[derive(Debug, Clone, Copy, PartialEq)]
        pub enum Dia {
            Segunda,
            Terca,
            Quarta,
        }
    }

    enum_com_nomes! {
        #[derive(Debug, PartialEq)]
        enum Nivel { Baixo, Alto }
    }

    #[test]
    fn todos_na_ordem() {
        assert_eq!(Dia::TODOS, [Dia::Segunda, Dia::Terca, Dia::Quarta]);
        assert_eq!(Nivel::TODOS.len(), 2);
    }

    #[test]
    fn nome_e_de_nome() {
        assert_eq!(Dia::Terca.nome(), "Terca");
        assert_eq!(Dia::de_nome("Quarta"), Some(Dia::Quarta));
        assert_eq!(Dia::de_nome("Domingo"), None);
        assert_eq!(Nivel::de_nome("Alto"), Some(Nivel::Alto));
        assert_eq!(Nivel::Baixo.nome(), "Baixo");
    }

    #[test]
    fn ida_e_volta() {
        for d in Dia::TODOS {
            assert_eq!(Dia::de_nome(d.nome()), Some(*d));
        }
    }
}
```

<details>
<summary>Dica</summary>

Os atributos são `$(#[$meta:meta])*` e a visibilidade é `$vis:vis`. Para o nome, `stringify!($variante)`; para `de_nome`, um `match texto { $(stringify!($variante) => Some(...),)+ _ => None }`.

</details>

## 5. `#[derive(Construtor)]` (difícil, passe nos testes)

Lições: [05](05-macros-procedurais-e-derive.md) e [06](06-erros-e-testes.md).

Escreva um derive que gera `pub fn new(...) -> Self` com um parâmetro por campo, na ordem. Campos marcados com `#[construtor(padrao)]` não viram parâmetro e recebem `Default::default()`. O derive precisa funcionar com genéricos, e um enum ou uma struct de tupla deve dar um erro de compilação (com `syn::Error`) apontando o problema.

Monte o workspace:

```text
construtor/
├── Cargo.toml
├── construtor-derive/   (cargo new --lib construtor-derive)
└── uso/                 (cargo new --lib uso)
```

```toml
# construtor/Cargo.toml
[workspace]
resolver = "3"
members = ["construtor-derive", "uso"]
```

```toml
# construtor-derive/Cargo.toml
[package]
name = "construtor-derive"
version = "0.1.0"
edition = "2024"
rust-version = "1.96"

[lib]
proc-macro = true

[dependencies]
proc-macro2 = "1.0.107"
quote = "1.0.47"
syn = "3.0.9"
```

```toml
# uso/Cargo.toml
[package]
name = "uso"
version = "0.1.0"
edition = "2024"
rust-version = "1.96"

[dependencies]
construtor-derive = { path = "../construtor-derive" }
```

O código inicial do derive compila, mas não gera nada:

```rust
// construtor-derive/src/lib.rs
use proc_macro::TokenStream;
use syn::{DeriveInput, parse_macro_input};

#[proc_macro_derive(Construtor, attributes(construtor))]
pub fn derive_construtor(entrada: TokenStream) -> TokenStream {
    let entrada = parse_macro_input!(entrada as DeriveInput);
    let _ = entrada;
    TokenStream::new() // não gera nada (ainda)
}
```

E os testes ficam na crate `uso`:

```rust
// uso/src/lib.rs
//! Tipos que usam `#[derive(Construtor)]`, e os testes do exercício 5.

use construtor_derive::Construtor;

#[derive(Debug, PartialEq, Construtor)]
pub struct Usuario {
    pub nome: String,
    pub idade: u32,
    #[construtor(padrao)]
    pub amigos: Vec<String>,
}

#[derive(Debug, PartialEq, Construtor)]
pub struct Par<T: Clone> {
    pub esquerda: T,
    pub direita: T,
}

#[derive(Debug, PartialEq, Construtor)]
pub struct Vazio {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn um_parametro_por_campo() {
        let u = Usuario::new("Ana".to_string(), 30);
        assert_eq!(u.nome, "Ana");
        assert_eq!(u.idade, 30);
        assert!(u.amigos.is_empty());
    }

    #[test]
    fn funciona_com_genericos() {
        let p = Par::new(1, 2);
        assert_eq!(
            p,
            Par {
                esquerda: 1,
                direita: 2
            }
        );
        let q = Par::new("a", "b");
        assert_eq!(q.direita, "b");
    }

    #[test]
    fn struct_sem_campos() {
        assert_eq!(Vazio::new(), Vazio {});
    }
}
```

Com o código inicial, `cargo test` (na raiz do workspace) falha com `error[E0599]: no associated function or constant named new found for struct Usuario`. Depois, confira à mão que `#[derive(Construtor)] enum E { A }` dá um erro apontando o `E`.

<details>
<summary>Dica</summary>

Siga o `LinhaCsv` da lição 05: separe uma função `expandir(DeriveInput) -> syn::Result<proc_macro2::TokenStream>`, percorra `Fields::Named`, leia `#[construtor(padrao)]` com `parse_nested_meta` e monte dois `Vec`s: os parâmetros (`#ident: #tipo`) e as inicializações (`#ident` ou `#ident: ::std::default::Default::default()`). No `quote!`, `pub fn new(#(#parametros),*) -> Self { Self { #(#inicializacoes),* } }`, dentro de um `impl` com `split_for_impl`.

</details>

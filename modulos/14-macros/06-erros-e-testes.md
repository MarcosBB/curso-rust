# 06 — Erros e testes de macros procedurais

**Objetivos.** Ao final, você vai saber:

- gerar erros de compilação que apontam para o lugar certo, com `syn::Error`;
- usar `quote_spanned!` para que erros no código gerado apontem para o código do usuário;
- testar uma macro procedural em três níveis: a função interna, o uso real e os casos que não devem compilar.

## `panic!` x `syn::Error`

Uma macro procedural é código Rust: se ela der *panic*, o compilador mostra a mensagem. O Rust Book faz isso, com `unwrap`, para manter o exemplo curto. O resultado (com uma versão do `LinhaCsv` que só faz `panic!`):

```text
error: proc-macro derive panicked
 --> src/main.rs:3:10
  |
3 | #[derive(Panica)]
  |          ^^^^^^
  |
  = help: message: `LinhaCsv` só pode ser derivada para structs
```

Funciona, mas o erro aponta sempre para o nome do derive, nunca para o pedaço do código que está errado. E um *panic* para tudo: a macro não consegue informar dois problemas de uma vez.

O jeito idiomático é devolver um **erro de compilação** como resultado da macro. `syn::Error` guarda uma mensagem e um *span* (a posição no código-fonte), e `into_compile_error()` o transforma em tokens `::core::compile_error! { "mensagem" }` com aquela posição. O compilador então mostra a mensagem exatamente ali.

Foi o que a lição 05 montou:

```rust
#[proc_macro_derive(LinhaCsv, attributes(csv))]
pub fn derive_linha_csv(entrada: TokenStream) -> TokenStream {
    let entrada = parse_macro_input!(entrada as DeriveInput);
    linha_csv::expandir(entrada)                         // syn::Result<TokenStream2>
        .unwrap_or_else(syn::Error::into_compile_error)  // Err vira compile_error!
        .into()
}
```

Dentro de `expandir`, cada problema é um `Err` com um span escolhido a dedo.

## Escolhendo o span

`syn::Error::new_spanned(tokens, mensagem)` usa a posição de qualquer pedaço da árvore. Na lição 05, o enum é rejeitado apontando o **nome** dele:

```rust
// ❌ NÃO COMPILA
use linha_csv::LinhaCsv;

#[derive(LinhaCsv)]
enum Status {
    Ativo,
    Inativo,
}

fn main() {}
```

```text
error: `LinhaCsv` só pode ser derivada para structs
 --> src/main.rs:4:6
  |
4 | enum Status {
  |      ^^^^^^
```

Uma struct de tupla é rejeitada apontando os **campos**, que é onde está o problema:

```text
error: `LinhaCsv` precisa de campos nomeados
 --> src/main.rs:4:13
  |
4 | struct Ponto(i32, i32);
  |             ^^^^^^^^^^
```

E dentro de `parse_nested_meta`, `meta.error(...)` aponta para a opção que foi lida:

```text
error: opção desconhecida; use `renomear = "..."` ou `pular`
 --> src/main.rs:5:11
  |
5 |     #[csv(renomar = "código")]
  |           ^^^^^^^
```

Essas três mensagens parecem ter saído do próprio compilador. É o padrão que o `serde` e o `thiserror` seguem, e que o usuário espera.

> [!TIP]
> Para relatar **vários** erros de uma vez (por exemplo, dois campos com atributos inválidos), junte-os com `erro.combine(outro)`. O `into_compile_error` gera um `compile_error!` para cada um.

Quando a entrada nem é do tipo esperado, o `parse_macro_input!` cuida sozinho. A macro de atributo da lição 07 espera uma função; aplicada a uma struct:

```text
error: expected `fn`
 --> src/main.rs:4:1
  |
4 | struct Ponto;
  | ^^^^^^
```

## Erros no código gerado: `quote_spanned!`

Há um tipo de erro que a macro não detecta: os que só aparecem quando o compilador verifica o código gerado. `LinhaCsv` chama `ToString::to_string` em cada campo, e um campo sem `Display` não tem `to_string`:

```rust
// ❌ NÃO COMPILA
use linha_csv::LinhaCsv;

#[derive(LinhaCsv)]
struct Item {
    nome: String,
    tags: Vec<String>,
}
```

Com a versão da lição 05 (tokens gerados por `quote!`), o erro aponta para o derive:

```text
error[E0277]: the trait bound `Vec<String>: ToString` is not satisfied
 --> src/main.rs:3:10
  |
3 | #[derive(LinhaCsv)]
  |          ^^^^^^^^ the trait `std::fmt::Display` is not implemented for `Vec<String>`
  |
  = note: required for `Vec<String>` to implement `ToString`
  = note: this error originates in the derive macro `LinhaCsv` (in Nightly builds, run with -Z macro-backtrace for more info)
```

Com uma struct de vinte campos, o usuário fica procurando qual deles é o culpado. A correção é dar aos tokens gerados para cada campo o span **do campo**. `quote_spanned!` é o `quote!` com uma posição:

```rust
use quote::{quote, quote_spanned};
use syn::spanned::Spanned;

// para cada campo:
valores.push(quote_spanned! {campo.ty.span()=>
    ::std::string::ToString::to_string(&self.#ident)
});

// e, no impl:
// ::std::vec![#(#valores),*]
```

O mesmo código, agora:

```text
error[E0277]: the trait bound `Vec<String>: ToString` is not satisfied
 --> src/main.rs:6:5
  |
6 |     tags: Vec<String>,
  |     ^^^^^^---
  |     |     |
  |     |     required by a bound introduced by this call
  |     the trait `std::fmt::Display` is not implemented for `Vec<String>`
  |
  = note: required for `Vec<String>` to implement `ToString`
```

O erro foi para a linha do campo. Essa é a versão que está em [`minhas-macros/src/linha_csv.rs`](codigo/macros-procedurais/minhas-macros/src/linha_csv.rs).

## Testar no nível da função: `parse_quote!` e `proc_macro2`

Uma função marcada com `#[proc_macro_derive]` não pode ser chamada num teste (o tipo `proc_macro::TokenStream` só existe dentro do compilador). Mas a função `expandir` usa `proc_macro2`, que funciona em qualquer lugar. E o `syn` tem `parse_quote!`, que monta uma árvore de sintaxe a partir de código escrito no teste:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_quote;

    #[test]
    fn gera_o_cabecalho_com_renomear_e_pular() {
        let entrada: DeriveInput = parse_quote! {
            struct Pessoa {
                #[csv(renomear = "nome completo")]
                nome: String,
                idade: u32,
                #[csv(pular)]
                senha: String,
            }
        };
        let gerado = expandir(entrada).unwrap().to_string();
        assert!(gerado.contains(r#"vec ! ["nome completo" , "idade"]"#), "{gerado}");
        assert!(!gerado.contains("senha"), "{gerado}");
    }

    #[test]
    fn rejeita_enum() {
        let entrada: DeriveInput = parse_quote! {
            enum Cor { Vermelho, Verde }
        };
        let erro = expandir(entrada).unwrap_err();
        assert_eq!(erro.to_string(), "`LinhaCsv` só pode ser derivada para structs");
    }
}
```

Esses testes rodam com `cargo test` na crate `minhas-macros`, sem compilar nenhum usuário. Repare na forma de `to_string()` de um `TokenStream`: os tokens saem separados por espaço (`vec ! [...]`). Comparar a saída inteira como texto é frágil, porque um detalhe de espaçamento muda o resultado. Teste **o que importa** (contém o título, não contém o campo pulado, qual é o erro) e deixe o comportamento real para o próximo nível.

## Testar o uso real: testes de integração

O teste mais valioso é usar o derive como um usuário usaria. Os [testes de integração](../09-testes-e-qualidade/04-testes-de-integracao.md) (Módulo 09) da crate da trait são o lugar natural: cada arquivo em `tests/` é outra crate, que depende de `linha_csv`, então o caminho `::linha_csv::LinhaCsv` resolve.

```rust
// linha-csv/tests/derive.rs
use linha_csv::{LinhaCsv, para_csv};
use std::fmt::Display;

#[derive(LinhaCsv)]
struct Produto {
    #[csv(renomear = "código")]
    codigo: u32,
    nome: String,
    preco: f64,
    #[csv(pular)]
    _rascunho: bool,
}

#[derive(LinhaCsv)]
struct Par<T: Display> {
    chave: &'static str,
    valor: T,
}

#[test]
fn cabecalho_respeita_renomear_e_pular() {
    assert_eq!(Produto::cabecalho(), ["código", "nome", "preco"]);
}

#[test]
fn funciona_com_genericos() {
    let p = Par { chave: "x", valor: 1.5 };
    assert_eq!(Par::<f64>::cabecalho(), ["chave", "valor"]);
    assert_eq!(p.linha(), ["x", "1.5"]);
}
```

Se o derive gerar código inválido, o teste nem compila, e o erro aparece na hora. Inclua sempre um caso com genéricos.

## Testar o que não deve compilar: `trybuild`

Falta um nível: garantir que os **erros** continuam bons. Não dá para escrever um `#[test]` que "não compila", mas a crate [`trybuild`](https://docs.rs/trybuild) (também de David Tolnay) faz isso: compila cada arquivo de uma pasta como um programa separado e compara a saída do compilador com um arquivo `.stderr` gravado.

```rust
// tests/ui.rs (com `trybuild` em [dev-dependencies])
#[test]
fn erros_de_compilacao() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui/*.rs");
}
```

Na primeira vez, `TRYBUILD=overwrite cargo test` grava o `.stderr` de cada caso (aqui, o `enum Status` de antes):

```text
error: `LinhaCsv` só pode ser derivada para structs
 --> tests/ui/enum.rs:4:6
  |
4 | enum Status {
  |      ^^^^^^
```

Depois disso, o teste falha se a mensagem mudar. É assim que o `serde`, o `thiserror` e o proc-macro-workshop testam as mensagens. O custo: a `trybuild` traz algumas dependências, e o `.stderr` precisa ser regravado quando uma versão nova do compilador muda a formatação. Por isso o código deste módulo não a usa; para uma biblioteca publicada, vale a pena.

## Armadilhas comuns

- **`unwrap` e `panic!` para erros de usuário.** O erro aponta para o derive e não para o problema. Devolva `syn::Error`.
- **Span genérico.** `syn::Error::new(Span::call_site(), ...)` aponta para a chamada inteira. Use `new_spanned` com o pedaço exato.
- **Testes que comparam o `TokenStream` inteiro como texto.** Quebram a cada mudança de espaçamento ou ordem. Teste propriedades e o uso real.
- **Testar só structs sem genéricos.** O primeiro usuário com `<T>` encontra o bug.

## Resumo

- Devolva `syn::Result` e transforme o erro com `into_compile_error()`: a mensagem aparece no lugar certo, como um erro do compilador.
- `new_spanned` escolhe o pedaço apontado; `meta.error` aponta a opção de um atributo; `combine` junta vários erros.
- `quote_spanned!` faz os erros do código gerado apontarem para o código do usuário.
- Teste em três níveis: a função interna com `parse_quote!`, o uso real em `tests/` e, para bibliotecas, as mensagens de erro com `trybuild`.

## Exercícios

Veja o exercício 5 em [`exercicios.md`](exercicios.md).

## Leitura complementar

- [Documentação de `syn::Error`](https://docs.rs/syn/3/syn/struct.Error.html) e de [`quote_spanned!`](https://docs.rs/quote/latest/quote/macro.quote_spanned.html)
- [Documentação de `parse_quote!`](https://docs.rs/syn/3/syn/macro.parse_quote.html)
- [Documentação da `trybuild`](https://docs.rs/trybuild)
- [proc-macro-workshop](https://github.com/dtolnay/proc-macro-workshop): cada etapa é um teste trybuild

---

[← Lição anterior](05-macros-procedurais-e-derive.md) · [Índice do módulo](README.md) · [Próxima lição →](07-atributos-e-function-like.md)

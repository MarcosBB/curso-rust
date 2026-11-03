# Soluções — Módulo 14

Enunciados em [`modulos/14-macros/exercicios.md`](../../modulos/14-macros/exercicios.md).

Cada solução é um projeto Cargo com os testes do enunciado. O exercício 5 é um workspace com duas crates. Para conferir:

```bash
cd nome-do-projeto
cargo test
```

## 1. `media!`

Projeto: [`ex1-media/`](ex1-media/src/lib.rs).

Com o código inicial, cada chamada com mais de um valor para no primeiro `,`:

```text
error: no rules expected `,`
  --> src/lib.rs:20:30
   |
 5 | macro_rules! media {
   | ------------------ when calling this macro
...
20 |         assert_eq!(media!(1.0, 2.0, 3.0), 2.0);
   |                              ^ no rules expected this token in macro call
   |
note: while trying to match meta-variable `$x:expr`
```

A solução:

```rust
#[macro_export]
macro_rules! media {
    () => {
        compile_error!("media! precisa de pelo menos um valor")
    };
    ($($x:expr),+ $(,)?) => {{
        // O array avalia cada expressão uma vez só, na ordem.
        let valores = [$($x as f64),+];
        valores.iter().sum::<f64>() / valores.len() as f64
    }};
}
```

- `$($x:expr),+` exige pelo menos um valor; `$(,)?` aceita a vírgula final.
- O array resolve dois problemas de uma vez: cada expressão aparece **uma** vez na expansão (então `proximo()` roda uma vez por argumento), e `len()` conta os valores sem truques de recursão.
- `$x as f64` converte inteiros e mantém `f64`. Como `$x` chega como uma expressão já agrupada, `media!(1 + 1)` vira `(1 + 1) as f64`, e não `1 + (1 as f64)`.
- A regra vazia vem primeiro e dá a mensagem pedida:

```text
error: media! precisa de pelo menos um valor
  --> src/lib.rs:7:9
   |
 7 |         compile_error!("media! precisa de pelo menos um valor")
   |         ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
...
47 | pub fn vazio() -> f64 { media!() }
   |                         -------- in this macro invocation
```

Sem ela, `media!()` também não compilaria, mas com o genérico `unexpected end of macro invocation`.

## 2. `maximo!` recursiva

Projeto: [`ex2-maximo/`](ex2-maximo/src/lib.rs).

O código inicial exige exatamente dois argumentos:

```text
error: unexpected end of macro invocation
  --> src/lib.rs:15:29
   |
 5 | macro_rules! maximo {
   | ------------------- when calling this macro
...
15 |         assert_eq!(maximo!(7), 7);
   |                             ^ missing tokens in macro arguments
   |
note: while trying to match `,`
```

A solução:

```rust
#[macro_export]
macro_rules! maximo {
    // Caso base: um valor só (com vírgula final opcional).
    ($x:expr $(,)?) => {
        $x
    };
    // Passo: o primeiro contra o máximo do resto.
    ($x:expr, $($resto:expr),+ $(,)?) => {
        ::std::cmp::max($x, $crate::maximo!($($resto),+))
    };
}
```

`maximo!(1, 5, 2)` vira `max(1, maximo!(5, 2))`, depois `max(1, max(5, maximo!(2)))` e, por fim, `max(1, max(5, 2))`. Como cada valor vira **argumento de uma função**, cada um é avaliado uma vez, da esquerda para a direita: o teste `avalia_cada_argumento_uma_vez` confere que `proximo()` roda três vezes e devolve 1, 2 e 3.

Consertar só a recursão e manter o `if $a > $b { $a } else { $b }` passaria nos dois primeiros testes, mas reprovaria no terceiro: o vencedor seria avaliado duas vezes (uma na comparação e outra no braço do `if`).

A chamada recursiva usa `$crate::maximo!` para funcionar também de fora da crate; dentro dela, `maximo!` sozinho bastaria. `::std::cmp::max` exige `Ord`, por isso `f64` não funciona; para isso seria preciso um `if` com um `match` guardando os valores, ou `f64::max`.

## 3. Uma macro para outras crates

Projeto: [`ex3-relatorios/`](ex3-relatorios/src/lib.rs), com o teste em [`tests/integracao.rs`](ex3-relatorios/tests/integracao.rs).

Os três erros aparecem um de cada vez. Primeiro, a ordem textual:

```text
error: cannot find macro `linha` in this scope
  --> src/lib.rs:8:5
   |
 8 |     linha!("total", total)
   |     ^^^^^ consider moving the definition of `linha` before this call
   |
note: a macro with the same name exists, but it appears later
```

Com `#[macro_export]` (ou movendo a macro para cima), surge a função auxiliar, procurada no escopo de quem chama:

```text
error[E0425]: cannot find function `moeda` in this scope
  --> src/lib.rs:14:36
   |
 8 |     linha!("total", total)
   |     ---------------------- in this macro invocation
...
14 |         format!("{}: {}", $rotulo, moeda($valor))
   |                                    ^^^^^ not found in this scope
   |
help: consider importing this function
   |
 1 + use crate::formato::moeda;
   |
```

Aceitar a sugestão conserta `resumo`, mas o teste de integração (outra crate) quebra do mesmo jeito:

```text
error[E0425]: cannot find function `moeda` in this scope
 --> tests/integracao.rs:5:16
  |
5 |     assert_eq!(linha!("frete", 12.5), "frete: R$ 12.50");
  |                ^^^^^^^^^^^^^^^^^^^^^ not found in this scope
```

E, sem `#[macro_export]`, o teste nem acha a macro: `error[E0432]: unresolved import relatorios::linha ... no linha in the root`.

A solução:

```rust
#[macro_export]
macro_rules! linha {
    ($rotulo:expr, $valor:expr) => {
        format!("{}: {}", $rotulo, $crate::formato::moeda($valor))
    };
}
```

- `#[macro_export]` publica a macro na raiz (`relatorios::linha`). Com ele, a chamada em `resumo` funciona mesmo **antes** da definição, porque a macro passa a ter um caminho, e não só escopo textual. A solução a move para cima mesmo assim, para ler na ordem.
- `$crate::formato::moeda` vira `crate::formato::moeda` dentro de `relatorios` e `::relatorios::formato::moeda` no teste.
- `moeda` já era `pub`; se não fosse, o teste de integração não poderia chamá-la, nem pela macro.

## 4. `enum_com_nomes!`

Projeto: [`ex4-enum-com-nomes/`](ex4-enum-com-nomes/src/lib.rs).

O código inicial não aceita atributos (`no rules expected #`). A solução:

```rust
#[macro_export]
macro_rules! enum_com_nomes {
    (
        $(#[$meta:meta])*
        $vis:vis enum $nome:ident {
            $($variante:ident),+ $(,)?
        }
    ) => {
        $(#[$meta])*
        $vis enum $nome {
            $($variante),+
        }

        impl $nome {
            pub const TODOS: &'static [$nome] = &[$($nome::$variante),+];

            pub fn nome(&self) -> &'static str {
                match self {
                    $($nome::$variante => stringify!($variante)),+
                }
            }

            pub fn de_nome(texto: &str) -> ::std::option::Option<$nome> {
                match texto {
                    $(stringify!($variante) => ::std::option::Option::Some($nome::$variante),)+
                    _ => ::std::option::Option::None,
                }
            }
        }
    };
}
```

- O padrão reproduz a sintaxe de um enum: `$(#[$meta:meta])*` captura zero ou mais atributos (o `#[derive(...)]`), `$vis:vis` captura `pub` ou nada, e a palavra `enum` é literal.
- `$($variante),+` aparece **três** vezes na expansão (no enum, em `TODOS` e nos dois `match`). Cada repetição percorre a mesma lista.
- `stringify!(Terca)` vira `"Terca"`, um literal de string, e por isso pode ser usado como **padrão** no `match` de `de_nome`.
- Usar uma fatia (`&'static [$nome]`) em vez de um array `[$nome; N]` evita ter de contar as variantes. Para um array, seria preciso o `contar!` da lição 03.
- Os caminhos `::std::option::Option` protegem de quem tenha um tipo chamado `Option` (raro, mas é o hábito de macros exportadas).

A crate `strum` faz isso (e muito mais) com um derive: `#[derive(EnumIter, IntoStaticStr, EnumString)]`.

## 5. `#[derive(Construtor)]`

Projeto: [`ex5-construtor/`](ex5-construtor/), com a macro em [`construtor-derive/src/lib.rs`](ex5-construtor/construtor-derive/src/lib.rs) e os testes em [`uso/src/lib.rs`](ex5-construtor/uso/src/lib.rs).

Com o código inicial, o derive não gera nada, e os testes não acham o `new`:

```text
error[E0599]: no associated function or constant named `new` found for struct `Usuario` in the current scope
   --> uso/src/lib.rs:28:26
    |
  6 | pub struct Usuario {
    | ------------------ associated function or constant `new` not found for this struct
...
 28 |         let u = Usuario::new("Ana".to_string(), 30);
    |                          ^^^ associated function or constant not found in `Usuario`
```

O núcleo da solução:

```rust
let mut parametros = Vec::new(); // `nome: Tipo`, na lista de argumentos
let mut inicializacoes = Vec::new(); // `nome` ou `nome: Default::default()`
for campo in campos {
    let ident = campo.ident.as_ref().expect("campo nomeado");
    let tipo = &campo.ty;

    let mut padrao = false;
    for attr in &campo.attrs {
        if attr.path().is_ident("construtor") {
            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("padrao") {
                    padrao = true;
                    Ok(())
                } else {
                    Err(meta.error("opção desconhecida; a única é `padrao`"))
                }
            })?;
        }
    }

    if padrao {
        inicializacoes.push(quote! { #ident: ::std::default::Default::default() });
    } else {
        parametros.push(quote! { #ident: #tipo });
        inicializacoes.push(quote! { #ident });
    }
}

let (impl_generics, ty_generics, where_clause) = entrada.generics.split_for_impl();
Ok(quote! {
    impl #impl_generics #nome #ty_generics #where_clause {
        pub fn new(#(#parametros),*) -> Self {
            Self { #(#inicializacoes),* }
        }
    }
})
```

- Os parâmetros usam **os nomes dos campos**, que vieram do usuário. Por isso `Self { nome, idade, amigos: ... }` (a forma abreviada de `nome: nome`) funciona: o identificador do parâmetro e o do campo são o mesmo token.
- `split_for_impl` faz `Par<T: Clone>` funcionar: `impl<T: Clone> Par<T> { pub fn new(esquerda: T, direita: T) -> Self ... }`.
- Uma struct sem campos (`Vazio {}`) gera `pub fn new() -> Self { Self {} }`, que também é válido.
- O enum e a struct de tupla são rejeitados com `syn::Error::new_spanned`, como no `LinhaCsv`. Os testes unitários da crate `construtor-derive` conferem isso com `parse_quote!`, e conferem que o campo `padrao` não vira parâmetro (`pub fn new (a : i32)` no texto gerado).
- Um campo `padrao` cujo tipo não implementa `Default` dá um erro no código gerado; para apontá-lo no campo, a melhoria seria a mesma do `quote_spanned!` da lição 06.

A crate `derive-new` faz exatamente isso (com mais opções), e o primeiro projeto do [proc-macro-workshop](https://github.com/dtolnay/proc-macro-workshop) é o próximo passo natural: um `derive(Builder)`.

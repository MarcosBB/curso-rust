# 07 — Macros de atributo e *function-like*

**Objetivos.** Ao final, você vai saber:

- escrever uma macro de atributo que transforma uma função (`#[cronometrar]`);
- evitar colisões de nomes em macros procedurais com `Span::mixed_site()`;
- escrever uma macro *function-like* que valida a entrada na compilação (`cor!("#f97316")`);
- reconhecer essas macros no ecossistema: `#[tokio::main]`, `#[async_trait]`, `sqlx::query!`.

## Macro de atributo: `#[cronometrar]`

Um derive só **acrescenta** código depois do item. Uma macro de atributo **substitui** o item: recebe o item inteiro e devolve o que vai ficar no lugar dele. É o equivalente em Rust a um decorator do Python:

```python
import functools
import time


def cronometrar(funcao):
    @functools.wraps(funcao)
    def embrulho(*args, **kwargs):
        inicio = time.perf_counter()
        resultado = funcao(*args, **kwargs)
        print(f"[cronometrar] {funcao.__name__} levou {time.perf_counter() - inicio:.4f}s")
        return resultado
    return embrulho


@cronometrar
def soma_dos_quadrados(n):
    return sum(x * x for x in range(1, n + 1))
```

```text
[cronometrar] soma_dos_quadrados levou 0.0457s
```

O decorator do Python roda quando o módulo é importado e troca a função por outra, que embrulha a original. A macro do Rust faz a troca na compilação, reescrevendo o **código** da função:

```rust
use minhas_macros::cronometrar;

#[cronometrar]
fn soma_dos_quadrados(n: u64) -> u64 {
    (1..=n).map(|x| x * x).sum()
}

println!("soma = {}", soma_dos_quadrados(1_000_000));
```

```text
[cronometrar] soma_dos_quadrados levou 19.755583ms
soma = 333333833333500000
```

(O tempo muda a cada execução.) A definição:

```rust
/// Mede o tempo de uma função e imprime no stderr ao terminar.
#[proc_macro_attribute]
pub fn cronometrar(args: TokenStream, item: TokenStream) -> TokenStream {
    let funcao = parse_macro_input!(item as ItemFn);
    cronometrar::expandir(args.into(), funcao)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}
```

Uma macro de atributo recebe **dois** `TokenStream`s: `args` é o que está entre os parênteses do atributo (em `#[cronometrar(ms)]`, seria `ms`), e `item` é o item marcado. `ItemFn` é a função inteira, que o `syn` só entende com a feature `"full"`.

```rust
use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{Ident, ItemFn};

pub fn expandir(args: TokenStream, funcao: ItemFn) -> syn::Result<TokenStream> {
    if !args.is_empty() {
        return Err(syn::Error::new_spanned(args, "`cronometrar` não recebe argumentos"));
    }
    if let Some(token) = &funcao.sig.asyncness {
        return Err(syn::Error::new_spanned(token, "`cronometrar` não funciona com `async fn`"));
    }

    let ItemFn { attrs, vis, sig, block, .. } = funcao;
    let nome = sig.ident.to_string();

    // Nomes com `Span::mixed_site()` se comportam como os de `macro_rules!`:
    // não colidem com um parâmetro do usuário que se chame `inicio`.
    let inicio = Ident::new("inicio", Span::mixed_site());
    let resultado = Ident::new("resultado", Span::mixed_site());

    // O corpo original vira uma closure chamada na hora: um `return` dentro
    // dele sai da closure, e o tempo é impresso do mesmo jeito.
    Ok(quote! {
        #(#attrs)*
        #vis #sig {
            let #inicio = ::std::time::Instant::now();
            let #resultado = (|| #block)();
            ::std::eprintln!("[cronometrar] {} levou {:?}", #nome, #inicio.elapsed());
            #resultado
        }
    })
}
```

Os pontos de projeto:

- **A assinatura (`#sig`), a visibilidade e os outros atributos (`#(#attrs)*`, inclusive o comentário de documentação) são devolvidos intactos.** Só o corpo muda. Quem chama não percebe a diferença.
- **O corpo vira uma closure chamada na hora.** Se o corpo tiver um `return`, ele sai da closure, e a linha do tempo ainda roda. Colar o bloco direto antes do `eprintln!` faria um `return` pular a medição.
- **`async fn` é rejeitada com uma mensagem clara.** Medir uma future exigiria embrulhar a future, não o corpo; é o tipo de caso que uma macro precisa recusar em vez de gerar algo errado.
- **O `..` no padrão** ignora o campo `modifiers`, que o `syn` 3 acrescentou e que uma função comum não usa. Sem o `..`, a desestruturação deixaria de compilar na próxima vez que o `syn` acrescentasse um campo.

Os erros, aplicando a macro onde não deve:

```rust
// ❌ NÃO COMPILA
use minhas_macros::cronometrar;

#[cronometrar]
struct Ponto;

#[cronometrar(ms)]
fn f() {}
```

```text
error: expected `fn`
 --> src/main.rs:4:1
  |
4 | struct Ponto;
  | ^^^^^^

error: `cronometrar` não recebe argumentos
 --> src/main.rs:6:15
  |
6 | #[cronometrar(ms)]
  |               ^^
```

## Higiene em macros procedurais

Por que os nomes `inicio` e `resultado` foram criados com `Span::mixed_site()`? Porque, ao contrário do `macro_rules!`, **os tokens do `quote!` não são higiênicos**. Eles recebem o span `Span::call_site()`, que se comporta como se o usuário tivesse escrito o código ali. Com um `let inicio = ...` escrito direto no `quote!`, uma função com um parâmetro chamado `inicio` quebra:

```rust
// ❌ NÃO COMPILA (com a versão sem `mixed_site`)
use minhas_macros::cronometrar;

#[cronometrar]
fn espera(inicio: u64) -> u64 {
    inicio * 2
}
```

```text
error[E0369]: cannot multiply `Instant` by `{integer}`
   --> src/main.rs:5:12
    |
  5 |     inicio * 2
    |     ------ ^ - {integer}
    |     |
    |     Instant
```

O `let inicio = Instant::now()` gerado pela macro **sombreou** o parâmetro, e o corpo do usuário passou a ver um `Instant`. É exatamente o problema das macros do C que a lição 04 mostrou que o `macro_rules!` evita.

`Span::mixed_site()` dá ao identificador a mesma higiene do `macro_rules!`: variáveis locais criadas assim não se misturam com as do usuário. Com ela, `espera(21)` devolve `42`. A regra prática:

| Identificador | Span |
|---|---|
| nomes que vieram do usuário (`#ident`, `#nome`) | o deles: deixe como está |
| variáveis locais que a macro cria | `Span::mixed_site()` |
| itens que o usuário deve enxergar (uma função gerada, por exemplo) | `Span::call_site()` (o padrão do `quote!`) |

Muitas macros antigas usam nomes como `__inicio` para "reduzir a chance" de colisão. Funciona quase sempre; `mixed_site` funciona sempre.

## Function-like: `cor!("#f97316")`

A terceira forma parece uma `macro_rules!` na chamada (`nome!(...)`), mas é uma função Rust, com toda a liberdade das procedurais. O uso típico é **validar ou calcular algo na compilação** que uma regra de `macro_rules!` não consegue. Por exemplo, converter uma cor em hexadecimal:

```rust
use minhas_macros::cor;

const LARANJA: [u8; 3] = cor!("#f97316");
println!("laranja = {LARANJA:?}"); // laranja = [249, 115, 22]
```

```rust
/// Converte `"#rrggbb"` em `[u8; 3]` durante a compilação.
#[proc_macro]
pub fn cor(entrada: TokenStream) -> TokenStream {
    let texto = parse_macro_input!(entrada as LitStr);
    cor::expandir(&texto)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

// cor.rs
pub fn expandir(texto: &LitStr) -> syn::Result<TokenStream> {
    let valor = texto.value();
    let erro = || syn::Error::new(texto.span(), "cor inválida: use o formato \"#rrggbb\"");

    let hex = valor.strip_prefix('#').ok_or_else(erro)?;
    if hex.len() != 6 {
        return Err(erro());
    }
    let canal = |i: usize| u8::from_str_radix(&hex[i..i + 2], 16).map_err(|_| erro());
    let (r, g, b) = (canal(0)?, canal(2)?, canal(4)?);

    Ok(quote! { [#r, #g, #b] })
}
```

A macro lê um literal de string, faz a conversão **enquanto compila** e gera só o array: o `cargo expand` mostra `const LARANJA: [u8; 3] = [249u8, 115u8, 22u8];`. Um `u8` interpolado com `#r` vira um literal com sufixo. Não há custo nenhum em tempo de execução, e uma cor errada não chega a virar programa:

```rust
// ❌ NÃO COMPILA
let c = cor!("#f9731");
```

```text
error: cor inválida: use o formato "#rrggbb"
 --> src/main.rs:4:18
  |
4 |     let c = cor!("#f9731");
  |                  ^^^^^^^^
```

> [!NOTE]
> Para esse caso específico, uma `const fn` (que roda na compilação quando usada numa `const`) também resolveria, e seria a escolha mais simples (lição 08). A macro ganha quando a validação precisa de algo que `const fn` não faz: ler um arquivo, consultar um banco de dados, analisar uma linguagem inteira.

## No ecossistema

Agora dá para ler o que algumas macros que você já usou fazem:

| Macro | Tipo | O que gera |
|---|---|---|
| `#[test]` | atributo (embutido no compilador) | registra a função no executor de testes |
| `#[tokio::main]` | atributo | um `fn main` comum que cria o runtime e chama `block_on` no seu corpo (Módulo 12) |
| `#[async_trait]` | atributo | reescreve cada `async fn` da trait para devolver `Pin<Box<dyn Future + Send + '_>>` (Módulo 12, lição 08) |
| `#[derive(Serialize)]` | derive | `impl Serialize`, lendo `#[serde(...)]` |
| `#[derive(thiserror::Error)]` | derive | `impl Display` e `impl Error` a partir de `#[error("...")]` (Módulo 05) |
| `sqlx::query!("SELECT ...")` | function-like | conecta ao banco **durante a compilação**, confere o SQL e os tipos das colunas |
| `include_str!("arquivo")` | function-like (embutida) | lê um arquivo na compilação e o insere como `&'static str` |

O `#[async_trait]` é um bom exemplo do poder (e do custo) das macros de atributo: ele pega a trait que você escreveu, que o compilador recusaria em `dyn`, e devolve outra, que funciona. Rode `cargo expand` num projeto com `async-trait` para ver o `Box::pin(async move { ... })` em cada método.

Código completo, com testes em [`exemplo/tests/macros.rs`](codigo/macros-procedurais/exemplo/tests/macros.rs) (inclusive o parâmetro `inicio` e o `return` antecipado), em [`codigo/macros-procedurais`](codigo/macros-procedurais/).

## Armadilhas comuns

- **Perder atributos ou a visibilidade da função.** Devolva `#(#attrs)*` e `#vis`; sem eles, a documentação e o `pub` somem.
- **Variáveis da macro sem `mixed_site`.** Um `let x` no `quote!` pode sombrear uma variável do usuário. Os erros que isso produz são confusos, como o `cannot multiply Instant` acima.
- **Ignorar argumentos.** Se a macro não recebe argumentos, rejeite `#[macro(algo)]` em vez de ignorar.
- **Embrulhar `async fn` como se fosse síncrona.** A medição pegaria só a criação da future. Recuse ou trate à parte.

## Resumo

- `#[proc_macro_attribute]` recebe os argumentos do atributo e o item, e devolve o item que fica no lugar. É o decorator do Rust, aplicado na compilação.
- Para funções, `syn::ItemFn` (feature `"full"`) separa atributos, visibilidade, assinatura e corpo.
- Tokens de `quote!` não são higiênicos (`call_site`); crie variáveis internas com `Span::mixed_site()`.
- `#[proc_macro]` cria uma macro `nome!(...)` que pode validar e calcular na compilação.

## Exercícios

O exercício 5 de [`exercicios.md`](exercicios.md) é um derive; os mesmos passos valem para um atributo.

## Leitura complementar

- [The Rust Programming Language, cap. 20.5: Attribute-like e Function-like macros](https://doc.rust-lang.org/stable/book/ch20-05-macros.html#attribute-like-macros)
- [The Rust Reference: Procedural macros](https://doc.rust-lang.org/reference/procedural-macros.html), em especial *Attribute macros* e *Procedural macro hygiene*
- [Documentação de `proc_macro::Span`](https://doc.rust-lang.org/proc_macro/struct.Span.html) (`call_site`, `mixed_site`)
- [Código do `async-trait`](https://github.com/dtolnay/async-trait) e do [`tokio::main`](https://docs.rs/tokio/latest/tokio/attr.main.html)

---

[← Lição anterior](06-erros-e-testes.md) · [Índice do módulo](README.md) · [Próxima lição →](08-quando-nao-usar-macros.md)

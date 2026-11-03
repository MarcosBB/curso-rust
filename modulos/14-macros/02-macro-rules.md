# 02 — `macro_rules!`: regras e fragmentos

**Objetivos.** Ao final, você vai saber:

- escrever uma macro declarativa com uma ou várias regras;
- escolher o fragmento certo (`expr`, `ident`, `ty`, `tt`...) para cada pedaço da entrada;
- o que mudou no fragmento `expr` na edition 2024;
- evitar a avaliação dupla e gerar erros próprios com `compile_error!`.

## A primeira macro

```rust
macro_rules! quadrado {
    ($x:expr) => {
        $x * $x
    };
}

fn main() {
    println!("{}", quadrado!(1 + 2)); // 9
}
```

A estrutura é a de um `match`, só que sobre **código**:

- `macro_rules! quadrado { ... }` define a macro `quadrado!`;
- cada **regra** tem a forma `(padrão) => { expansão };`;
- no padrão, `$x:expr` captura uma expressão e lhe dá o nome `$x`. É uma *metavariável*;
- na expansão, `$x` é substituída pelo que foi capturado.

Os delimitadores são livres: `quadrado!(3)`, `quadrado![3]` e `quadrado!{3}` são a mesma chamada. A convenção é `()` para algo que parece função, `[]` para coleções (`vec![]`) e `{}` para blocos de itens.

## Várias regras: a primeira que casa vence

```rust
macro_rules! cor {
    (vermelho) => {
        [255, 0, 0]
    };
    (verde) => {
        [0, 255, 0]
    };
    (rgb($r:expr, $g:expr, $b:expr)) => {
        [$r, $g, $b]
    };
}

let laranja: [u8; 3] = cor!(rgb(249, 115, 22));
println!("{:?} {laranja:?}", cor!(verde)); // [0, 255, 0] [249, 115, 22]
```

Tudo o que não é metavariável no padrão é **literal**: `vermelho`, `rgb`, os parênteses e as vírgulas precisam aparecer exatamente assim na chamada. As regras são testadas **de cima para baixo**, e a primeira que casa é usada. Por isso as regras mais específicas vêm antes das mais gerais.

Se nenhuma regra casa, o erro diz onde a entrada parou de casar:

```rust
// ❌ NÃO COMPILA
macro_rules! quadrado {
    ($x:expr) => {
        $x * $x
    };
}
fn main() {
    let a = quadrado!(3, 4);
    println!("{a}");
}
```

```text
error: no rules expected `,`
 --> src/main.rs:7:24
  |
1 | macro_rules! quadrado {
  | --------------------- when calling this macro
...
7 |     let a = quadrado!(3, 4);
  |                        ^ no rules expected this token in macro call
  |
note: while trying to match meta-variable `$x:expr`
 --> src/main.rs:2:6
  |
2 |     ($x:expr) => {
  |      ^^^^^^^
```

O compilador leu `3` como a expressão `$x`, esperava o fim da entrada e encontrou `,`. Nenhuma regra aceita mais de um argumento. (Aceitar uma lista é o assunto da lição 03.)

## Fragmentos

O que vem depois dos dois-pontos em `$x:expr` é o **fragmento** (*fragment specifier*): o tipo de coisa sintática que a metavariável aceita.

| Fragmento | Casa com | Exemplo |
|---|---|---|
| `expr` | uma expressão | `1 + 2`, `f(x)`, `if a { b } else { c }` |
| `ident` | um identificador (ou palavra-chave) | `x`, `Ponto`, `soma` |
| `ty` | um tipo | `i32`, `Vec<String>`, `&'a str` |
| `path` | um caminho | `std::mem::swap`, `Vec::<u8>::new` |
| `pat` | um padrão (com `\|`) | `Some(x)`, `1 \| 2`, `_` |
| `pat_param` | um padrão sem `\|` no topo | `Some(x)` |
| `literal` | um literal (com `-` opcional) | `42`, `-1.5`, `"texto"`, `'c'` |
| `block` | um bloco | `{ let a = 1; a }` |
| `stmt` | uma instrução sem o `;` final | `let x = 1` |
| `item` | um item | `fn f() {}`, `struct S;` |
| `lifetime` | um lifetime | `'a`, `'static` |
| `vis` | uma visibilidade, possivelmente vazia | `pub`, `pub(crate)`, nada |
| `meta` | o conteúdo de um atributo | `derive(Debug)`, `allow(dead_code)` |
| `tt` | uma *token tree*: um token ou um grupo `(...)`/`[...]`/`{...}` | `+`, `foo`, `(a, b)` |

Duas regras práticas:

- **Prefira o fragmento mais específico.** Com `$x:expr`, a macro recebe uma expressão já agrupada, e o erro de quem passar outra coisa sai na chamada. Com `tt`, você aceita qualquer coisa e descobre o problema mais tarde, no código gerado.
- **Depois de capturado como `expr`, `ty` etc., o pedaço é opaco.** Ele não pode mais ser desmontado por outra regra. Se você precisa olhar dentro dele, capture como `tt`.

### O que pode vir depois de um fragmento

Para que a gramática de Rust possa crescer sem quebrar macros antigas, alguns fragmentos só podem ser seguidos por certos tokens:

```rust
// ❌ NÃO COMPILA
macro_rules! dois {
    ($a:expr $b:expr) => { $a + $b };
}
```

```text
error: `$a:expr` is followed by `$b:expr`, which is not allowed for `expr` fragments
 --> src/main.rs:2:14
  |
2 |     ($a:expr $b:expr) => { $a + $b };
  |              ^^^^^^^ not allowed after `expr` fragments
  |
  = note: allowed there are: `=>`, `,` or `;`
```

Depois de `expr` e `stmt`, só `=>`, `,` ou `;`. Depois de `ty` e `path`, um conjunto maior (`,`, `=`, `|`, `;`, `:`, `>`, `as`, `where`, `{`...). `ident`, `tt`, `literal` e `lifetime` podem ser seguidos por qualquer coisa. Na prática, separe as metavariáveis com vírgulas ou `=>`, e o problema não aparece.

## Edition 2024: `expr` ficou maior

Na edition 2024, o fragmento `expr` passou a aceitar duas formas que antes não aceitava no topo: o `_` (usado em atribuições como `(a, _) = par`) e os blocos `const { ... }`. Quem precisa do comportamento antigo usa `expr_2021`:

```rust
macro_rules! tipo {
    ($e:expr) => { "expr" };
    ($($t:tt)*) => { "outra coisa" };
}
macro_rules! tipo_2021 {
    ($e:expr_2021) => { "expr" };
    ($($t:tt)*) => { "outra coisa" };
}
fn main() {
    println!("_        -> {} / 2021: {}", tipo!(_), tipo_2021!(_));
    println!("const {{}} -> {} / 2021: {}", tipo!(const { 1 }), tipo_2021!(const { 1 }));
    println!("1 + 2    -> {} / 2021: {}", tipo!(1 + 2), tipo_2021!(1 + 2));
}
```

```text
_        -> expr / 2021: outra coisa
const {} -> expr / 2021: outra coisa
1 + 2    -> expr / 2021: expr
```

(A segunda regra, `$($t:tt)*`, aceita qualquer sequência de tokens. A sintaxe `$(...)*` é a repetição da próxima lição.) Isso só importa se uma macro sua tem uma regra com `$e:expr` seguida de outra regra que trata `_` de um jeito especial: na 2024, a primeira passa a capturar o `_`. O `cargo fix --edition` troca `expr` por `expr_2021` onde a mudança faria diferença.

## A avaliação dupla

Precedência não é problema em Rust (`quadrado!(1 + 2)` dá 9), mas a outra armadilha das macros do C continua: a expressão aparece **duas vezes** na expansão, então roda duas vezes.

```rust
fn proximo(contador: &mut i32) -> i32 {
    *contador += 1;
    *contador
}

let mut c = 0;
let ingenuo = quadrado_ingenuo!(proximo(&mut c)); // a versão `$x * $x`
println!("quadrado_ingenuo!(proximo()) = {ingenuo}, proximo() rodou {c} vezes");
```

```text
quadrado_ingenuo!(proximo()) = 2, proximo() rodou 2 vezes
```

A expansão é `proximo(&mut c) * proximo(&mut c)`, ou seja, `1 * 2`. Com efeitos colaterais (ler um arquivo, sortear um número, incrementar um contador), o resultado fica errado; sem eles, o trabalho é feito duas vezes.

A correção é avaliar uma vez e guardar o valor:

```rust
// ✅ avalia `$x` uma vez só
macro_rules! quadrado {
    ($x:expr) => {
        match $x {
            v => v * v,
        }
    };
}
```

```text
quadrado!(proximo()) = 1, proximo() rodou 1 vez
```

O `match` com um único braço guarda o valor de `$x` em `v` e funciona em qualquer posição onde caiba uma expressão. É o idioma da std: o `assert_eq!` faz exatamente isso com os dois lados. Um bloco `{ let v = $x; v * v }` também serve. Repare também que `v` não colide com nenhuma variável `v` do chamador; isso é a higiene, da lição 04.

## Ferramentas dentro da expansão: `stringify!` e `compile_error!`

Algumas macros da std são úteis dentro das suas:

- `stringify!(...)` transforma tokens em um `&'static str` com o código-fonte. É como o `assert!` mostra a condição que falhou.
- `concat!(...)` junta literais num `&'static str` na compilação.
- `compile_error!("mensagem")` para a compilação com essa mensagem.

```rust
macro_rules! mostrar {
    ($e:expr) => {
        format!("{} = {:?}", stringify!($e), $e)
    };
}

println!("{}", mostrar!(2 * 21)); // 2 * 21 = 42
```

`compile_error!` numa última regra transforma "nenhuma regra casou" num erro que você escreveu:

```rust
// ❌ NÃO COMPILA (de propósito)
macro_rules! cor {
    (vermelho) => { (255, 0, 0) };
    (verde) => { (0, 255, 0) };
    ($outra:ident) => {
        compile_error!(concat!("cor desconhecida: ", stringify!($outra)))
    };
}
fn main() {
    let c = cor!(azul);
    println!("{c:?}");
}
```

```text
error: cor desconhecida: azul
 --> src/main.rs:5:9
  |
5 |         compile_error!(concat!("cor desconhecida: ", stringify!($outra)))
  |         ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
...
9 |     let c = cor!(azul);
  |             ---------- in this macro invocation
```

O erro aponta para a regra **e** para a chamada. Muitas crates usam esse truque para trocar o "no rules expected" genérico por uma mensagem que explica o que a macro aceita.

Código completo, com testes, em [`codigo/macros-declarativas/src/basico.rs`](codigo/macros-declarativas/src/basico.rs).

## Armadilhas comuns

- **Regra geral antes da específica.** Com `($x:expr)` antes de `(vermelho)`, a palavra `vermelho` é capturada como expressão (uma variável chamada `vermelho`), e a regra de baixo nunca é usada. Ordene da mais específica para a mais geral.
- **Esquecer o `;` entre as regras.** Cada regra termina com `;` (a última pode omitir).
- **Avaliar a expressão duas vezes.** Se `$x` aparece mais de uma vez na expansão, guarde-o com `match $x { v => ... }`.
- **Usar `tt` para tudo.** Funciona, mas os erros aparecem longe da chamada. Use `expr`, `ty` e `ident` sempre que der.

## Resumo

- `macro_rules!` define regras `(padrão) => { expansão }`, testadas de cima para baixo.
- Metavariáveis como `$x:expr` capturam um pedaço de código do tipo do fragmento; o resto do padrão é literal.
- Depois de `expr`, só `=>`, `,` ou `;`. Na 2024, `expr` aceita `_` e `const {}`; `expr_2021` mantém o antigo.
- Expressões usadas duas vezes rodam duas vezes: guarde o valor com `match`.
- `stringify!`, `concat!` e `compile_error!` ajudam a montar mensagens e erros.

## Exercícios

Veja o exercício 1 em [`exercicios.md`](exercicios.md) (ele usa também a repetição da próxima lição).

## Leitura complementar

- [The Rust Reference: Macros by example](https://doc.rust-lang.org/reference/macros-by-example.html), em especial *Metavariables* e *Follow-set ambiguity restrictions*
- [Edition Guide: Macro fragment specifiers (2024)](https://doc.rust-lang.org/edition-guide/rust-2024/macro-fragment-specifiers.html)
- [The Little Book of Rust Macros: Fragment specifiers](https://lukaswirth.dev/tlborm/decl-macros/minutiae/fragment-specifiers.html)

---

[← Lição anterior](01-o-que-sao-macros.md) · [Índice do módulo](README.md) · [Próxima lição →](03-repeticoes-e-recursao.md)

# 04 — Higiene, escopo e `$crate`

**Objetivos.** Ao final, você vai saber:

- o que é higiene e por que as variáveis de uma macro não colidem com as suas;
- onde uma macro pode ser usada: ordem no arquivo, módulos, `pub(crate) use`;
- exportar uma macro para outras crates com `#[macro_export]`;
- usar `$crate` para que o código gerado ache os itens da sua crate.

## Higiene: as variáveis da macro são da macro

No C, uma macro que declara uma variável `tmp` quebra o código de quem também tem um `tmp`. Em Rust, isso não acontece:

```rust
macro_rules! com_temporaria {
    ($e:expr) => {{
        let x = 10;
        x + $e
    }};
}

let x = 1;
println!("com_temporaria!(x * 2) = {}", com_temporaria!(x * 2));
```

```text
com_temporaria!(x * 2) = 12
```

Se a expansão fosse texto, `x + x * 2` daria 30. Mas o compilador lembra **de onde veio** cada identificador: o `x` escrito dentro da macro e o `x` que você passou em `$e` são variáveis diferentes, mesmo com o mesmo nome. O resultado é `10 + 1 * 2`. Isso é a **higiene** (*hygiene*).

Funciona nos dois sentidos. Uma variável criada pela macro não aparece para quem chamou:

```rust
// ❌ NÃO COMPILA
macro_rules! cria_x {
    () => {
        let x = 42;
    };
}
fn main() {
    cria_x!();
    println!("{x}");
}
```

```text
error[E0425]: cannot find value `x` in this scope
 --> src/main.rs:8:16
  |
8 |     println!("{x}");
  |                ^ not found in this scope
  |
help: an identifier with the same name is defined here, but is not accessible due to macro hygiene
 --> src/main.rs:3:13
  |
3 |         let x = 42;
  |             ^
...
7 |     cria_x!();
  |     --------- in this macro invocation
```

A mensagem diz o motivo com todas as letras. Para criar uma variável que o chamador possa usar, **o nome tem de vir do chamador**:

```rust
macro_rules! declarar {
    ($nome:ident = $valor:expr) => {
        let $nome = $valor;
    };
}

declarar!(total = 40 + 2);
println!("total = {total}"); // total = 42
```

Agora o identificador `total` foi escrito por quem chamou, então pertence ao escopo dele.

### O que a higiene cobre (e o que não cobre)

A higiene do `macro_rules!` é chamada de *mixed site* ("lugar misto"):

- **variáveis locais e rótulos de laço** (`'externo:`) são resolvidos onde a macro foi **definida**: são higiênicos;
- **todo o resto** (funções, tipos, módulos, macros, constantes) é resolvido onde a macro foi **chamada**.

O segundo ponto é o que mais pega. Uma macro que gera `fn resposta() {}` cria uma função visível para quem chamou. E uma macro que **usa** uma função auxiliar procura essa função no escopo de quem chamou, não no da macro:

```rust
// ❌ NÃO COMPILA
mod relatorio {
    pub fn formatar(rotulo: &str, valor: f64) -> String {
        format!("{rotulo}: {valor:.2}")
    }

    macro_rules! linha {
        ($rotulo:expr, $valor:expr) => {
            formatar($rotulo, $valor as f64)
        };
    }
    pub(crate) use linha;
}

fn main() {
    println!("{}", relatorio::linha!("total", 10));
}
```

```text
error[E0425]: cannot find function `formatar` in this scope
  --> src/main.rs:8:13
   |
 8 |             formatar($rotulo, $valor as f64)
   |             ^^^^^^^^ not found in this scope
...
15 |     println!("{}", relatorio::linha!("total", 10));
   |                    ------------------------------ in this macro invocation
   |
help: consider importing this function
   |
 1 + use crate::relatorio::formatar;
   |
```

Dentro de `relatorio`, `formatar` existe; em `main`, não. A solução não é pedir para todo mundo importar `formatar`, e sim escrever um caminho que funcione de qualquer lugar. Isso é o `$crate`, mais abaixo.

## Onde uma macro existe: escopo textual

Funções podem ser usadas antes de serem definidas. Macros de `macro_rules!`, não: elas existem do ponto da definição **para baixo**, como uma variável.

```rust
// ❌ NÃO COMPILA
fn main() {
    let v = dobro!(2);
    println!("{v}");
}
macro_rules! dobro {
    ($x:expr) => { $x * 2 };
}
```

```text
error: cannot find macro `dobro` in this scope
 --> src/main.rs:2:13
  |
2 |     let v = dobro!(2);
  |             ^^^^^ consider moving the definition of `dobro` before this call
  |
note: a macro with the same name exists, but it appears later
 --> src/main.rs:5:14
  |
5 | macro_rules! dobro {
  |              ^^^^^
```

Esse escopo, chamado **textual**, atravessa módulos filhos declarados depois da macro, mas não sobe para o módulo pai nem vai para módulos irmãos. Há dois jeitos de usar uma macro fora do módulo em que ela foi definida.

**1. Transformá-la num item com `use` (o jeito moderno).** Dentro da crate, `pub(crate) use` dá à macro um caminho, como uma função:

```rust
// em src/basico.rs
macro_rules! quadrado {
    ($x:expr) => {
        match $x {
            v => v * v,
        }
    };
}
pub(crate) use quadrado;

// em qualquer outro módulo da crate
println!("{}", crate::basico::quadrado!(7)); // 49
```

O `use` precisa vir **depois** da definição (ele também segue a ordem textual), e a visibilidade máxima é `pub(crate)`. Para ir além da crate, só com `#[macro_export]`.

**2. `#[macro_use]` no módulo (o jeito antigo).** Estende o escopo textual da macro para o resto do módulo pai:

```rust
#[macro_use]
mod util {
    macro_rules! dobro {
        ($x:expr) => { $x * 2 };
    }
}

mod outro {
    pub fn usa() -> i32 {
        dobro!(21)
    }
}
```

Funciona, mas a ordem dos `mod` em `main.rs`/`lib.rs` passa a importar (`mod util;` precisa vir antes dos módulos que usam a macro), e quem lê `outro` não tem um `use` dizendo de onde `dobro!` veio. Em código novo, prefira o `use`. Sem nenhum dos dois, o compilador sugere justamente o `#[macro_use]`: `help: have you added the #[macro_use] on the module/import?`.

## Exportar para outras crates: `#[macro_export]`

Uma macro com `#[macro_export]` vira pública e fica na **raiz** da crate, não importa em que módulo foi escrita:

```rust
// src/higiene.rs, na crate `macros_declarativas`
#[doc(hidden)]
pub fn formatar_registro(nivel: &str, mensagem: &str) -> String {
    format!("[{nivel}] {mensagem}")
}

#[macro_export]
macro_rules! registrar {
    ($nivel:ident, $($arg:tt)+) => {
        $crate::higiene::formatar_registro(stringify!($nivel), &format!($($arg)+))
    };
}
```

```rust
// tests/exportada.rs: um teste de integração é outra crate
use macros_declarativas::registrar; // da raiz, e não de `higiene::`

#[test]
fn registrar_funciona_fora_da_crate() {
    let total = 3;
    assert_eq!(registrar!(info, "{total} arquivos"), "[info] 3 arquivos");
}
```

Repare no `$($arg:tt)+`: a macro repassa o resto da chamada, intacto, para o `format!`. É o jeito de "herdar" a sintaxe de outra macro.

Dentro da própria crate, uma macro exportada também ganha o caminho `crate::registrar!`, que vale em qualquer módulo e em qualquer ordem.

## `$crate`: o caminho da crate que definiu a macro

O código gerado por `registrar!` roda **na crate de quem chamou**. Lá, `higiene::formatar_registro` não existe, e `crate::higiene::...` apontaria para a crate do chamador. A metavariável especial `$crate` resolve isso: ela sempre significa "a crate onde a macro foi definida".

- Chamada de dentro da `macros_declarativas`, `$crate` vira `crate`.
- Chamada de outra crate, vira `::macros_declarativas`.

Por isso, em toda macro exportada:

- itens da sua crate: `$crate::modulo::item`;
- itens da std: caminho absoluto, `::std::collections::HashMap`, como no `hashmap!` da lição 03 (o usuário pode ter um tipo próprio chamado `HashMap`);
- a função auxiliar precisa ser `pub`, porque o código roda fora da crate. Marque com `#[doc(hidden)]` para não aparecer na documentação.

Essa é a correção do erro do `formatar` acima: `$crate::relatorio::formatar($rotulo, $valor as f64)`.

> [!WARNING]
> O compilador sugere `use crate::relatorio::formatar;` no erro. Isso conserta a chamada **dentro** da sua crate, mas não ajuda quem usa a macro de fora: lá, o erro volta. Em macro exportada, a resposta é sempre `$crate::`.

## Visibilidade como fragmento: `$vis`

O fragmento `vis` captura uma visibilidade, inclusive a ausência dela. Assim, o chamador decide se o item gerado é público:

```rust
macro_rules! constante {
    ($vis:vis $nome:ident = $valor:expr) => {
        $vis const $nome: i32 = $valor;
    };
}

mod config {
    constante!(pub LIMITE = 10);
    constante!(pub(crate) PADRAO = 5);
    constante!(INTERNO = 3);
    constante!(pub(self) TAMBEM_INTERNO = 4);

    pub fn total() -> i32 {
        LIMITE + PADRAO + INTERNO + TAMBEM_INTERNO
    }
}
```

`pub(self)` é o "privado" escrito por extenso, a curiosidade da [tabela de visibilidade do Módulo 08](../08-modulos-e-crates/02-visibilidade.md). Ele é útil justamente em macros: quando uma macro precisa **repassar** uma visibilidade para outra, ou gerar uma visibilidade explícita, `pub(self)` é um token que significa "nada".

> [!NOTE]
> Duas extensões de `macro_rules!` estão em andamento e ainda são **instáveis** no Rust 1.99 (erro E0658): macros de atributo e de derive escritas com `macro_rules!` (`attr() (...) => {...}`), e as expressões de metavariável como `${count($x)}`. Por ora, atributos e derives exigem macros procedurais, que começam na próxima lição.

Código completo, com testes, em [`codigo/macros-declarativas/src/higiene.rs`](codigo/macros-declarativas/src/higiene.rs) e [`tests/exportada.rs`](codigo/macros-declarativas/tests/exportada.rs).

## Armadilhas comuns

- **Usar a macro antes de definir.** Mova a definição para cima, ou para um módulo declarado antes. Com `#[macro_export]`, o caminho `crate::nome!` vale em qualquer ordem.
- **Caminho relativo no código gerado.** `formatar(...)`, `HashMap::new()` e `Comando::Andar` dependem do que o chamador importou. Use `$crate::...` e `::std::...`.
- **Importar a macro exportada pelo módulo.** `use minha_crate::util::registrar;` não funciona: `#[macro_export]` põe a macro na raiz, `use minha_crate::registrar;`.
- **Esperar que a macro crie uma variável para o chamador.** A higiene impede. Receba o nome como `$nome:ident`.

## Resumo

- Variáveis locais e rótulos criados por uma `macro_rules!` não colidem com os do chamador (higiene *mixed site*); funções, tipos e o resto são procurados onde a macro é chamada.
- Uma macro só existe do ponto da definição para baixo. `pub(crate) use nome;` a transforma num item com caminho.
- `#[macro_export]` publica a macro na raiz da crate.
- `$crate::caminho` aponta sempre para a crate que definiu a macro; use caminhos absolutos para a std.
- `$vis:vis` deixa o chamador escolher a visibilidade; `pub(self)` é o privado explícito.

## Exercícios

Veja o exercício 3 em [`exercicios.md`](exercicios.md).

## Leitura complementar

- [The Rust Reference: Hygiene](https://doc.rust-lang.org/reference/macros-by-example.html#hygiene) e [Scoping, exporting, and importing](https://doc.rust-lang.org/reference/macros-by-example.html#scoping-exporting-and-importing)
- [The Rust Reference: `$crate`](https://doc.rust-lang.org/reference/macros-by-example.html#r-macro.decl.meta.dollar-crate)
- [The Little Book of Rust Macros: Hygiene](https://lukaswirth.dev/tlborm/decl-macros/minutiae/hygiene.html)

---

[← Lição anterior](03-repeticoes-e-recursao.md) · [Índice do módulo](README.md) · [Próxima lição →](05-macros-procedurais-e-derive.md)

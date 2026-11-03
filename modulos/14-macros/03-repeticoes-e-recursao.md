# 03 — Repetições e recursão

**Objetivos.** Ao final, você vai saber:

- aceitar listas de argumentos com `$(...),*`, `+` e `?`, inclusive a vírgula final;
- repetir na expansão, com separadores e com repetições aninhadas;
- gerar itens (structs e `impl`s), e não só expressões;
- escrever macros recursivas, incluindo o padrão *tt muncher*.

## `$(...)*`: aceitar uma lista

É assim que o `vec!` aceita qualquer número de elementos. Um `hashmap!` no mesmo estilo:

```rust
macro_rules! hashmap {
    ($($chave:expr => $valor:expr),* $(,)?) => {{
        let mut mapa = ::std::collections::HashMap::new();
        $(
            mapa.insert($chave, $valor);
        )*
        mapa
    }};
}

let precos = hashmap! {
    "café" => 18.5,
    "pão" => 0.9,
};
```

Lendo o padrão por partes:

- `$( ... ),*` é uma **repetição**: o que está dentro dos parênteses pode aparecer zero ou mais vezes, separado por vírgulas;
- dentro dela, `$chave:expr => $valor:expr` é um par. O `=>` é literal;
- `$(,)?` aceita uma vírgula final opcional, para quem escreve um item por linha.

Na expansão, `$( mapa.insert($chave, $valor); )*` repete a instrução **uma vez para cada par capturado**. As chaves duplas `{{ ... }}` não são sintaxe especial: as de fora delimitam a expansão, e as de dentro formam um bloco, para que a macro possa ser usada como expressão (`let precos = hashmap! {...}`).

Os três operadores de repetição:

| Operador | Significa | Exemplo |
|---|---|---|
| `*` | zero ou mais | `$($x:expr),*` aceita `()` e `(1, 2)` |
| `+` | um ou mais | `$($x:expr),+` rejeita `()` |
| `?` | zero ou um (sem separador) | `$(,)?`, `$(mut)?` |

O separador fica entre o `)` e o operador e pode ser qualquer token, menos os delimitadores e os próprios operadores: `$($x:expr);*` separa por `;`, `$($x:ident)*` não tem separador.

## Repetir com um separador na expansão

O separador também funciona do lado da expansão. Para somar uma lista, `+` vira o "separador" que junta os termos:

```rust
macro_rules! soma {
    ($primeiro:expr $(, $resto:expr)* $(,)?) => {
        $primeiro $(+ $resto)*
    };
}

println!("soma!(1, 2, 3) = {}", soma!(1, 2, 3)); // soma!(1, 2, 3) = 6
```

`soma!(1, 2, 3)` expande para `1 + 2 + 3`. O padrão separa o primeiro dos demais para exigir pelo menos um número, e a expansão escreve `+ $resto` para cada um dos outros.

## Repetições aninhadas

Uma repetição pode conter outra. Para uma matriz com as linhas separadas por `;`:

```rust
macro_rules! matriz {
    ($($($x:expr),+);+ $(;)?) => {
        vec![$(vec![$($x),+]),+]
    };
}

println!("{:?}", matriz![1, 2, 3; 4, 5, 6]); // [[1, 2, 3], [4, 5, 6]]
```

`$x` foi capturado em **dois** níveis de repetição (linhas e colunas), então na expansão ele só pode aparecer dentro de dois níveis também. O compilador confere isso.

## Os erros de repetição

Quando duas metavariáveis aparecem na mesma repetição da expansão, elas precisam ter o mesmo número de itens:

```rust
// ❌ NÃO COMPILA
macro_rules! pares {
    ($($k:expr),* ; $($v:expr),*) => {
        vec![$(($k, $v)),*]
    };
}
fn main() {
    let p = pares!(1, 2, 3; "a", "b");
    println!("{p:?}");
}
```

```text
error: meta-variable `k` repeats 3 times, but `v` repeats 2 times
 --> src/main.rs:3:15
  |
3 |         vec![$(($k, $v)),*]
  |               ^^^^^^^^^^
```

E uma metavariável capturada dentro de uma repetição não pode ser usada fora dela:

```rust
// ❌ NÃO COMPILA
macro_rules! lista {
    ($($x:expr),*) => {
        vec![$x]
    };
}
```

```text
error: variable `x` is still repeating at this depth
 --> src/main.rs:3:14
  |
3 |         vec![$x]
  |              ^^
```

`$x` é uma lista, e `vec![$x]` pede um valor só. A correção é `vec![$($x),*]`.

## Gerar itens: structs e `impl`s

Uma macro pode expandir para qualquer coisa que caiba no lugar da chamada. No nível do módulo, isso inclui itens. Um caso típico é gerar vários tipos parecidos:

```rust
use std::fmt;
use std::ops::Add;

macro_rules! unidades {
    ($($nome:ident($tipo:ty) => $sufixo:literal),+ $(,)?) => {
        $(
            #[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
            pub struct $nome(pub $tipo);

            impl fmt::Display for $nome {
                fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                    write!(f, "{} {}", self.0, $sufixo)
                }
            }

            impl Add for $nome {
                type Output = $nome;
                fn add(self, outro: $nome) -> $nome {
                    $nome(self.0 + outro.0)
                }
            }
        )+
    };
}

unidades! {
    Metros(f64) => "m",
    Segundos(f64) => "s",
    Bytes(u64) => "B",
}

fn main() {
    let trajeto = Metros(120.0) + Metros(30.5);
    let tempo = Segundos(12.0);
    println!("{trajeto} em {tempo}; arquivo de {}", Bytes(2048));
}
```

```text
150.5 m em 12 s; arquivo de 2048 B
```

Três *newtypes* (Módulo 13), cada um com `Display` e `Add`, em quatro linhas de chamada. É o uso mais comum de `macro_rules!` em bibliotecas: a std usa macros assim para implementar as mesmas traits para `i8`, `i16`, `i32`... sem copiar o código doze vezes.

Repare em `$tipo:ty` e `$sufixo:literal`: os fragmentos específicos fazem `unidades! { Metros(1 + 1) => x }` falhar com um erro claro na chamada.

## Recursão

Uma macro pode chamar a si mesma. Cada chamada consome uma parte da entrada, até sobrar o caso base:

```rust
macro_rules! contar {
    () => { 0usize };
    ($primeiro:tt $($resto:tt)*) => { 1usize + contar!($($resto)*) };
}

const N: usize = contar!(a b c d);
println!("contar!(a b c d) = {N}"); // contar!(a b c d) = 4
```

`contar!(a b c d)` vira `1 + contar!(b c d)`, que vira `1 + 1 + contar!(c d)`, e assim até `contar!()`, que vira `0`. O resultado é uma expressão constante, então funciona até como tamanho de array: `[0u8; contar!(x y z)]`.

> [!NOTE]
> Existe uma sintaxe própria para contar repetições, `${count($x)}`, mas em novembro de 2026 ela ainda é **instável** (erro E0658, *meta-variable expressions are unstable*). Use a recursão ou o truque do array (`[$($x),*].len()`, que avalia as expressões).

### O padrão *tt muncher*

Com recursão e `tt`, uma macro pode ler uma mini-linguagem pedaço por pedaço. O padrão tem nome: *tt muncher* ("mastigador de tokens"). A cada passo, uma regra reconhece o começo da entrada, faz alguma coisa com ele e chama a macro de novo com o resto.

```rust
#[derive(Debug, PartialEq)]
pub enum Comando {
    Andar(u32),
    Girar,
    Parar,
}

macro_rules! comandos {
    // Fim da entrada: devolve o que foi acumulado.
    (@acc [$($feito:expr),*]) => {
        vec![$($feito),*]
    };
    (@acc [$($feito:expr),*] andar $n:literal; $($resto:tt)*) => {
        comandos!(@acc [$($feito,)* $crate::repeticoes::Comando::Andar($n)] $($resto)*)
    };
    (@acc [$($feito:expr),*] girar; $($resto:tt)*) => {
        comandos!(@acc [$($feito,)* $crate::repeticoes::Comando::Girar] $($resto)*)
    };
    (@acc [$($feito:expr),*] parar; $($resto:tt)*) => {
        comandos!(@acc [$($feito,)* $crate::repeticoes::Comando::Parar] $($resto)*)
    };
    // Ponto de entrada: começa com o acumulador vazio.
    ($($entrada:tt)*) => {
        comandos!(@acc [] $($entrada)*)
    };
}

println!("{:?}", comandos!(andar 3; girar; andar 1; parar;));
```

```text
[Andar(3), Girar, Andar(1), Parar]
```

Duas técnicas aparecem aqui:

- **Regras internas.** As regras que começam com `@acc` são "privadas": quem chama a macro não escreve `@acc`. O `@` não tem significado especial; é só um token que ninguém usaria por acaso. A regra pública fica **por último**, porque `$($entrada:tt)*` casa com tudo.
- **Acumulador.** Uma macro não pode devolver "metade" de uma expressão, então os comandos já lidos vão sendo guardados entre `[...]` e só viram o `vec!` no fim.

O `$crate` é explicado na próxima lição; por ora, leia como "o caminho desta crate".

### O limite de recursão

Cada passo recursivo conta. O padrão é 128 níveis:

```rust
// ❌ NÃO COMPILA
macro_rules! infinita {
    ($($t:tt)*) => { infinita!($($t)* 1) };
}
fn main() { infinita!(0); }
```

```text
error: recursion limit reached while expanding `infinita!`
 --> src/main.rs:2:22
  |
2 |     ($($t:tt)*) => { infinita!($($t)* 1) };
  |                      ^^^^^^^^^^^^^^^^^^^
3 | }
4 | fn main() { infinita!(0); }
  |             ------------ in this macro invocation
  |
  = help: consider increasing the recursion limit by adding a `#![recursion_limit = "256"]` attribute to your crate (`...`)
```

Uma macro que realmente precisa de mais níveis (um *tt muncher* com centenas de tokens) pode aumentar o limite com o atributo sugerido. Mas, quase sempre, o erro significa que falta um caso base ou que nenhuma regra está consumindo a entrada. Além disso, *tt munchers* grandes deixam a compilação lenta: cada passo copia o resto da entrada.

Código completo, com testes, em [`codigo/macros-declarativas/src/repeticoes.rs`](codigo/macros-declarativas/src/repeticoes.rs) e [`gerar_codigo.rs`](codigo/macros-declarativas/src/gerar_codigo.rs).

## Armadilhas comuns

- **Esquecer a vírgula final.** `$($x:expr),*` rejeita `(1, 2,)`. Acrescente `$(,)?` no fim do padrão.
- **`*` quando a lista vazia não faz sentido.** Use `+`, ou escreva o primeiro elemento à parte, como no `soma!`.
- **Avisos no código gerado.** Com `hashmap! {}`, a expansão tem `let mut mapa` sem nenhum `insert`, e o compilador avisa no código de quem chamou:

  ```text
  warning: variable does not need to be mutable
   --> src/repeticoes.rs:6:13
    |
  6 |         let mut mapa = ::std::collections::HashMap::new();
    |             ----^^^^
    |             |
    |             help: remove this `mut`
  ```

  Quando o aviso é esperado para algumas entradas, ponha `#[allow(unused_mut)]` na linha gerada, como faz o código do módulo.
- **Regra "pega-tudo" no começo.** Num *tt muncher*, a regra `$($t:tt)*` precisa ser a última.

## Resumo

- `$(...),*` repete zero ou mais vezes com `,` de separador; `+` exige pelo menos um; `?` torna opcional. `$(,)?` aceita a vírgula final.
- Na expansão, `$(...)*` repete uma vez por item capturado, e todas as metavariáveis dentro dela precisam ter o mesmo número de itens.
- Uma macro pode gerar itens inteiros: structs, `impl`s, funções.
- Recursão consome a entrada aos poucos; o *tt muncher* com regras internas `@acc` lê mini-linguagens.
- O limite padrão é 128 níveis; bater nele geralmente indica um caso base faltando.

## Exercícios

Veja os exercícios 1, 2 e 4 em [`exercicios.md`](exercicios.md).

## Leitura complementar

- [The Rust Reference: Repetitions](https://doc.rust-lang.org/reference/macros-by-example.html#repetitions)
- [The Little Book of Rust Macros: TT munchers](https://lukaswirth.dev/tlborm/decl-macros/patterns/tt-muncher.html), [Internal rules](https://lukaswirth.dev/tlborm/decl-macros/patterns/internal-rules.html) e [Counting](https://lukaswirth.dev/tlborm/decl-macros/building-blocks/counting.html)
- [The Rust Programming Language, cap. 20.5](https://doc.rust-lang.org/stable/book/ch20-05-macros.html) (a versão simplificada de `vec!`)

---

[← Lição anterior](02-macro-rules.md) · [Índice do módulo](README.md) · [Próxima lição →](04-higiene-e-escopo.md)

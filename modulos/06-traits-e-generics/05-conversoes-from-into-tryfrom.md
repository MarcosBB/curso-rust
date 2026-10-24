# 05 — Conversões: `From`, `Into`, `TryFrom` (e a *deref coercion*)

**Objetivos.** Ao final, você vai saber:

- implementar `From` e pedir `Into` nos parâmetros;
- modelar conversões que podem falhar com `TryFrom`, e conversões de texto com `FromStr`;
- por que o `?` converte erros sozinho, agora com todas as peças;
- usar `AsRef` em parâmetros;
- o que é a *deref coercion*, a conversão implícita que você já usa desde o Módulo 02.

## Conversões são traits

O Módulo 01 avisou: `From`, `Into` e `TryFrom` são traits. Não existe conversão implícita entre tipos em Rust, nem construtor "não `explicit`" como em C++, nem `__int__` chamado pelo interpretador como em Python. Toda conversão é uma chamada que você escreve, e o tipo de destino diz qual implementação usar.

| Trait | Método | Para quê |
|---|---|---|
| `From<T>` | `U::from(t)` | conversão que **não falha** e não perde informação |
| `Into<U>` | `t.into()` | o mesmo, visto do outro lado |
| `TryFrom<T>` | `U::try_from(t)` | conversão que **pode falhar** (devolve `Result`) |
| `TryInto<U>` | `t.try_into()` | o mesmo, visto do outro lado |
| `FromStr` | `s.parse::<U>()` | conversão de texto (pode falhar) |
| `AsRef<T>` | `x.as_ref()` | "me empresta como `&T`", barato |

Os números da biblioteca padrão mostram a ideia. `i32` → `i64` sempre cabe, então existe `From`; `i32` → `u8` pode não caber, então só existe `TryFrom`:

```rust
let grande: i32 = 300;
println!("{}", i64::from(grande));        // 300
println!("{:?}", u8::try_from(grande));   // Err(TryFromIntError(PosOverflow))
println!("{}", grande as u8);             // 44
```

O `as`, visto no Módulo 01, faz a conversão em qualquer caso e trunca em silêncio: 300 vira 44. Use `as` só quando o truncamento é o que você quer; para o resto, `From` (não falha) ou `TryFrom` (avisa).

## `From` e `Into`: implemente um, ganhe os dois

Uma conversão sem perda, de `Percentual` para a fração em `f64`:

```rust
impl From<Percentual> for f64 {
    fn from(p: Percentual) -> f64 {
        f64::from(p.0) / 100.0
    }
}

let p = Percentual::try_from(25).unwrap();
let a = f64::from(p);   // 0.25
let b: f64 = p.into();  // 0.25, o mesmo
```

Você só escreveu `From`, e `into()` já funciona. É um *blanket impl* da biblioteca padrão (lição 03):

```rust
impl<T, U> Into<U> for T where U: From<T> {
    fn into(self) -> U { U::from(self) }
}
```

"Se `U` sabe ser criado a partir de `T`, então `T` sabe virar `U`." Por isso a regra é: **implemente `From`, nunca `Into`**. Se você implementar `Into` direto, perde o `From` e o clippy reclama (lint `from_over_into`).

O `into()` sozinho não diz o destino, então ele precisa vir do contexto: uma anotação (`let b: f64`), o tipo de um parâmetro ou o tipo de retorno. Sem contexto, é o erro de inferência da lição 01. E se a conversão não existir, o erro aponta o `From` que falta:

```rust
// ❌ NÃO COMPILA
let grande: i64 = 300;
let x: i32 = grande.into();
```

```text
error[E0277]: the trait bound `i32: From<i64>` is not satisfied
 --> src/main.rs:3:25
  |
3 |     let x: i32 = grande.into();
  |                         ^^^^ the trait `From<i64>` is not implemented for `i32`
  |
  = help: `i32` implements trait `From<T>`:
            From<bool>
            From<i16>
            From<i8>
            From<u16>
            From<u8>
  = note: required for `i64` to implement `Into<i32>`
```

A lista do `help:` é a lista de conversões sem perda para `i32`. `i64` não está nela, porque nem todo `i64` cabe num `i32`: aqui o certo é `i32::try_from(grande)`.

### Peça `Into` nos parâmetros

Do lado de quem **recebe**, `Into` é o bound mais flexível. Uma função que precisa de uma `String` própria pode aceitar tudo o que vira `String`:

```rust
pub fn saudar(nome: impl Into<String>) -> String {
    let mut s: String = nome.into();
    s.insert_str(0, "Olá, ");
    s
}

saudar("Ana");                 // &str
saudar(String::from("Bia"));  // String, sem cópia extra
```

Quem tem um `&str` não precisa escrever `.to_string()`, e quem já tem uma `String` a entrega sem alocar outra. Resumindo a regra das duas pontas: **implemente `From`, peça `Into`**.

## `TryFrom`: quando a conversão pode falhar

O Módulo 04 deixou uma ponta solta: converter um inteiro em enum, ou validar um valor na criação, com `TryFrom`. A trait tem um *tipo associado* `Error` (o tipo de erro da conversão; a lição 08 explica tipos associados):

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Percentual(u8);

#[derive(Debug, PartialEq, Eq)]
pub enum ErroPercentual {
    ForaDaFaixa(i64),
    SemSimbolo,
    NaoNumerico(ParseIntError),
}

impl TryFrom<i64> for Percentual {
    type Error = ErroPercentual;

    fn try_from(valor: i64) -> Result<Self, Self::Error> {
        match valor {
            0..=100 => Ok(Percentual(valor as u8)), // a faixa garante que cabe em u8
            _ => Err(ErroPercentual::ForaDaFaixa(valor)),
        }
    }
}
```

Como o campo de `Percentual` é privado (fora do módulo), `try_from` é a única porta de entrada: todo `Percentual` que existe está entre 0 e 100. É a ideia de "estados inválidos irrepresentáveis" do Módulo 04, agora com a interface padrão. E, como com `From`/`Into`, `try_into()` vem de graça:

```rust
let p: Result<Percentual, _> = (-1_i64).try_into(); // Err(ForaDaFaixa(-1))
```

## `FromStr`: o que está por trás de `parse`

Desde o Módulo 01 você escreve `"42".parse::<i32>()`. A assinatura de `parse` é `fn parse<F: FromStr>(&self) -> Result<F, F::Err>`: ela funciona com qualquer tipo que implemente `FromStr`. Implemente para o seu tipo, e `parse` passa a aceitá-lo:

```rust
impl FromStr for Percentual {
    type Err = ErroPercentual;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let numero = s.trim().strip_suffix('%').ok_or(ErroPercentual::SemSimbolo)?;
        let valor: i64 = numero.parse()?; // ParseIntError -> ErroPercentual
        Percentual::try_from(valor)
    }
}
```

```rust
for texto in ["75%", "150%", "abc%", "42"] {
    match texto.parse::<Percentual>() {
        Ok(p) => println!("{texto:>5} -> {p} = {}", f64::from(p)),
        Err(e) => println!("{texto:>5} -> erro: {e}"),
    }
}
```

```text
  75% -> 75% = 0.75
 150% -> erro: 150 está fora de 0..=100
 abc% -> erro: não é um número
   42 -> erro: faltou o símbolo %
```

O código completo, com testes e o `Display` dos dois tipos, está em [`codigo/conversoes`](codigo/conversoes/).

## O `?` e o `From`: a peça que faltava

Repare no `numero.parse()?` dentro de `from_str`. O `parse::<i64>` falha com `ParseIntError`, e a função devolve `ErroPercentual`. Quem converte?

O Módulo 05 disse que o `?` chama `From::from(e)` antes de devolver o erro. Agora dá para ler isso ao pé da letra: na falha, `x?` faz algo equivalente a `return Err(From::from(e))`, e o tipo de destino vem do retorno da função. Para isso compilar, precisa existir `impl From<ParseIntError> for ErroPercentual`:

```rust
impl From<ParseIntError> for ErroPercentual {
    fn from(e: ParseIntError) -> Self {
        ErroPercentual::NaoNumerico(e)
    }
}
```

É exatamente a "receita 3" do Módulo 05. E o `Box<dyn Error>` aceita qualquer erro no `?` porque a biblioteca padrão tem um blanket impl `impl<E: Error> From<E> for Box<dyn Error>`. Nada de mágico: são traits e blanket impls.

## `AsRef`: emprestar como outro tipo

`AsRef<T>` é para conversões **baratas de referência**: "me dê um `&T` olhando para você". Ela aparece muito em parâmetros que só precisam ler:

```rust
use std::path::Path;

pub fn extensao(caminho: impl AsRef<Path>) -> Option<String> {
    let ext = caminho.as_ref().extension()?;
    Some(ext.to_string_lossy().into_owned())
}

extensao("notas.txt");                // &str
extensao(String::from("a.rs"));       // String
extensao(PathBuf::from("a/b.rs"));    // PathBuf
```

É assim que `std::fs::read_to_string` aceita tanto `"poem.txt"` quanto um `PathBuf`: a assinatura é `pub fn read_to_string<P: AsRef<Path>>(path: P)`.

A diferença para `Into`: `Into<String>` **consome** o valor e pode alocar; `AsRef<Path>` só empresta e não custa nada. Use `Into` quando a função vai guardar o valor, e `AsRef` quando só vai olhar. Existe ainda a parente `Borrow`, que aparece na busca de `HashMap` (Módulo 07).

## A *deref coercion*: a conversão implícita que você já usa

No Projeto 1, a dica da etapa 3 dizia que `positional.push(arg)` funcionava com `arg: &String` num `Vec<&str>` porque o compilador converte `&String` em `&str` sozinho, por *deref coercion*. É a hora de explicar.

Veja esta família de chamadas, todas aceitas:

```rust
fn tamanho(s: &str) -> usize {
    s.len()
}

fn soma(v: &[i32]) -> i32 {
    let mut total = 0;
    for x in v {
        total += x;
    }
    total
}

let s = String::from("olá");
let b: Box<String> = Box::new(String::from("caixa"));
let v = vec![1, 2, 3];
println!("{} {} {}", tamanho(&s), tamanho(&b), soma(&v));
println!("{} {}", s.to_uppercase(), b.starts_with("ca"));
```

```text
4 5 6
OLÁ true
```

`tamanho` pede `&str` e recebeu `&String` e até `&Box<String>`. `soma` pede `&[i32]` e recebeu `&Vec<i32>`. E `to_uppercase` e `starts_with` são métodos de `str`, chamados numa `String` e num `Box<String>`.

Quem faz isso é a trait `std::ops::Deref`. Um tipo que implementa `Deref<Target = U>` diz "eu me comporto como uma referência para `U`": `String` implementa `Deref<Target = str>`, `Vec<T>` implementa `Deref<Target = [T]>`, e `Box<T>` implementa `Deref<Target = T>`. Com isso, o compilador aplica duas regras:

1. **Na passagem de argumentos** (e em outros lugares onde o tipo esperado é conhecido), se você tem `&A`, o esperado é `&B`, e `A: Deref<Target = B>`, ele insere a conversão. E repete quantas vezes precisar: `&Box<String>` → `&String` → `&str`.
2. **Na chamada de método**, se o método não existe em `A`, o compilador procura em `*A`, depois em `**A`, e assim por diante. Por isso os métodos de `str` aparecem numa `String`.

É uma das pouquíssimas conversões implícitas do Rust, e ela é restrita: só vale de **referência para referência**, só segue `Deref`, e não cria valor novo (nada é copiado ou alocado). É por isso que a recomendação de assinaturas do Módulo 03 funciona: uma função que pede `&str` aceita `&String` de graça, mas uma que pede `&String` não aceita `&str`.

Comparando: em C++, um construtor não `explicit` ou um `operator T()` permitem conversões implícitas entre valores, com uma cadeia que pode surpreender. Rust não tem nada disso; a *deref coercion* é o único caso, e só para ponteiros.

> [!NOTE]
> Aqui você só **usa** `Deref`. Implementar `Deref` para os seus tipos (um *smart pointer* próprio, por exemplo) é o Módulo 10, junto com `Box` e companhia. Um aviso antecipado: implementar `Deref` num newtype só para "herdar" os métodos do tipo de dentro é considerado antipadrão. O Módulo 10 explica por quê.

## Armadilhas comuns

**Implementar `Into` em vez de `From`.** Você perde o `From` e o clippy avisa (`from_over_into`). Implemente `From`, e o `Into` vem pelo blanket impl.

**`From` para uma conversão que pode falhar ou perder dados.** `From` promete que sempre funciona. Se existe entrada inválida, é `TryFrom`; esconder um `panic!` ou um truncamento dentro de um `from` surpreende quem chama.

**`as` para converter entre inteiros de tamanhos diferentes.** `300 as u8` dá 44, sem aviso. Prefira `u8::try_from(x)` e trate o erro, ou `u8::from` quando a conversão sempre cabe.

**`into()` sem contexto.** O compilador não sabe o destino (E0282 ou E0283). Anote o tipo da variável ou use `Tipo::from(x)`, que deixa o destino explícito.

**Esperar *deref coercion* em valores.** Ela só converte referências. Um parâmetro `s: String` não aceita um `&str`; aí é `to_string()` ou um `impl Into<String>`.

## Resumo

- Conversões são traits, e são sempre explícitas: `From`/`Into` (não falham), `TryFrom`/`TryInto` (devolvem `Result`), `FromStr` (o `parse`), `AsRef` (referência barata).
- Implemente `From`; o `Into` vem de graça. Peça `impl Into<T>` nos parâmetros que vão guardar o valor e `impl AsRef<T>` nos que só leem.
- `as` trunca em silêncio; prefira `From`/`TryFrom` entre inteiros.
- O `?` converte o erro com `From::from`. É por isso que a receita do Módulo 05 tem um `impl From` por tipo de erro de baixo.
- *Deref coercion*: `&String` → `&str`, `&Vec<T>` → `&[T]`, `&Box<T>` → `&T`, aplicada pelo compilador porque esses tipos implementam `Deref`. Implementar `Deref` é no Módulo 10.

## Exercícios

Veja o exercício 4 em [`exercicios.md`](exercicios.md).

## Leitura complementar

- [Documentação de `std::convert`](https://doc.rust-lang.org/std/convert/index.html)
- [Documentação de `std::ops::Deref`](https://doc.rust-lang.org/std/ops/trait.Deref.html), seção *Deref coercion*
- [Effective Rust, Item 5: Understand type conversions](https://effective-rust.com/casts.html)
- [Effective Rust, Item 8: Familiarize yourself with reference and pointer types](https://effective-rust.com/references.html)
- [Comprehensive Rust: From and Into](https://google.github.io/comprehensive-rust/std-traits/from-and-into.html)
- [The Rust Programming Language, 15.2: Implicit Deref Coercions](https://doc.rust-lang.org/stable/book/ch15-02-deref.html)

---

[← Lição anterior](04-traits-da-std-e-derive.md) · [Índice do módulo](README.md) · [Próxima lição →](06-trait-objects-e-dyn.md)

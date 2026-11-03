# 05 — Blanket impls, extension traits e sealed traits

**Objetivos.** Ao final, você vai saber:

- acrescentar métodos a tipos que não são seus (`str`, `Vec`, todo `Iterator`) com uma *extension trait*;
- escrever um *blanket impl* da sua trait para uma família inteira de tipos;
- *selar* uma trait, para que outros crates possam usá-la mas não implementá-la;
- reconhecer esses padrões nas crates que você já usa.

## Métodos novos em tipos alheios

Você quer `"Ame a ema".eh_palindromo()`. Em Python, daria para pendurar uma função numa classe sua em tempo de execução (*monkey patching*), mas não em `str`: `str.eh_palindromo = ...` dá `TypeError: cannot set 'eh_palindromo' attribute of immutable type 'str'`. Em C++, você escreveria uma função livre, `eh_palindromo(s)`, porque não há como acrescentar membros a `std::string`.

Em Rust, você não pode escrever um `impl str { ... }` fora da biblioteca padrão. Mas pode criar uma trait **sua** e implementá-la para `str`. A orphan rule (Módulo 06) permite, porque a trait é do seu crate:

```rust
pub trait TextoExt {
    /// Ignora maiúsculas, espaços e pontuação.
    fn eh_palindromo(&self) -> bool;
    /// "o rato roeu" -> "O Rato Roeu".
    fn capitalizar_palavras(&self) -> String;
}

impl TextoExt for str {
    fn eh_palindromo(&self) -> bool {
        let letras: Vec<char> = self
            .chars()
            .filter(|c| c.is_alphanumeric())
            .flat_map(char::to_lowercase)
            .collect();
        letras.iter().eq(letras.iter().rev())
    }

    fn capitalizar_palavras(&self) -> String {
        self.split_whitespace()
            .map(|p| {
                let mut cs = p.chars();
                match cs.next() {
                    Some(primeira) => primeira.to_uppercase().chain(cs).collect(),
                    None => String::new(),
                }
            })
            .collect::<Vec<String>>()
            .join(" ")
    }
}
```

```rust
use extensoes::texto::TextoExt;

for frase in ["Ame a ema", "Rust", "A base do teto desaba"] {
    println!("{frase:?}: palíndromo? {}", frase.eh_palindromo());
}
println!("{}", "o rato roeu a roupa".capitalizar_palavras());
```

```text
"Ame a ema": palíndromo? true
"Rust": palíndromo? false
"A base do teto desaba": palíndromo? true
O Rato Roeu A Roupa
```

Isso é uma *extension trait*: uma trait cujo propósito é só acrescentar métodos a um tipo existente. A convenção é o sufixo `Ext`. Como implementamos para `str` (e não para `String`), o método funciona também em `String`, `&String` e `Box<str>`, pela *deref coercion* (Módulo 06).

A diferença para o monkey patching é o escopo: os métodos só existem onde a trait foi importada. Ninguém "estraga" `str` para o resto do programa, e duas bibliotecas podem ter cada uma o seu `eh_palindromo` sem conflito.

### A trait precisa estar em escopo

O lado ruim dessa regra é o erro mais comum com extension traits:

```rust
// ❌ NÃO COMPILA
fn main() {
    println!("{}", "Ame a ema".eh_palindromo());
}
```

```text
error[E0599]: no method named `eh_palindromo` found for reference `&'static str` in the current scope
  --> src/main.rs:12:32
   |
12 |     println!("{}", "Ame a ema".eh_palindromo());
   |                                ^^^^^^^^^^^^^ method not found in `&'static str`
   |
   = help: items from traits can only be used if the trait is in scope
help: trait `TextoExt` which provides `eh_palindromo` is implemented but not in scope; perhaps you want to import it
   |
 1 + use extensoes::texto::TextoExt;
   |
```

O compilador sabe que a trait existe e sugere o `use` certo. É por isso que muitas crates têm um módulo `prelude` (`use rayon::prelude::*`, Módulo 11): ele reexporta as extension traits de uma vez.

Você já usou várias extension traits sem saber o nome:

| Trait | Crate | Acrescenta a |
|---|---|---|
| `AsyncReadExt`, `AsyncWriteExt` | tokio | todo `AsyncRead`/`AsyncWrite` (Módulo 12) |
| `IntoParallelRefIterator` (o `par_iter`) | rayon | `Vec`, slices e outras coleções (Módulo 11) |
| `Itertools` | itertools | todo `Iterator` |
| `StreamExt`, `FutureExt` | futures | todo `Stream`/`Future` |
| `Context` | anyhow | todo `Result` (Módulo 05) |

## Blanket impls: uma `impl` para muitos tipos

`TextoExt` foi implementada para um tipo só. E se você quiser um método novo em **todo** iterador? Não dá para listar os tipos (são infinitos: cada `map` com uma closure diferente gera um tipo). A saída é o *blanket impl* do [Módulo 06](../06-traits-e-generics/03-trait-bounds-where-e-impl-trait.md), agora numa trait sua:

```rust
use std::collections::HashMap;
use std::hash::Hash;

/// Extension trait com blanket impl: vale para **todo** iterador.
pub trait IteratorExt: Iterator {
    /// Quantas vezes cada item aparece.
    fn contagem(self) -> HashMap<Self::Item, usize>
    where
        Self: Sized,
        Self::Item: Hash + Eq,
    {
        let mut mapa = HashMap::new();
        for item in self {
            *mapa.entry(item).or_insert(0) += 1;
        }
        mapa
    }

    /// O item mais frequente (empate: qualquer um deles).
    fn moda(self) -> Option<Self::Item>
    where
        Self: Sized,
        Self::Item: Hash + Eq,
    {
        self.contagem()
            .into_iter()
            .max_by_key(|&(_, n)| n)
            .map(|(item, _)| item)
    }
}

impl<I: Iterator> IteratorExt for I {}
```

As peças:

- `trait IteratorExt: Iterator`: a *supertrait* (Módulo 06) dá acesso a `Self::Item` e a tudo de `Iterator` dentro dos métodos padrão.
- Os métodos têm corpo na própria trait (métodos padrão). A `impl` fica vazia.
- `impl<I: Iterator> IteratorExt for I {}`: "todo tipo que é `Iterator` ganha `IteratorExt`". É isso que faz o método aparecer em qualquer iterador.
- `where Self: Sized`: o método consome `self` por valor, o que exige tamanho conhecido (lição 02). Pôr o bound no **método**, e não na trait, mantém a trait usável com `dyn` (o `Iterator` da std faz o mesmo).
- `where Self::Item: Hash + Eq`: um bound só para este método. Iteradores de `f64` continuam tendo `IteratorExt`; só não conseguem chamar `contagem`.

```rust
let texto = "o rato roeu a roupa do rei de roma o rato";
let contagem = texto.split_whitespace().contagem();
println!("'rato' aparece {} vezes", contagem["rato"]);
println!("moda das letras de 'abracadabra': {:?}", "abracadabra".chars().moda());
```

```text
'rato' aparece 2 vezes
moda das letras de 'abracadabra': Some('a')
```

É exatamente como a crate `itertools` funciona: `pub trait Itertools: Iterator { ... }` com dezenas de métodos padrão e `impl<T: Iterator + ?Sized> Itertools for T {}`.

> [!WARNING]
> Um blanket impl tem um custo de compatibilidade: depois dele, você não pode escrever outra `impl` da mesma trait para um tipo que ele já cobre (o E0119 do [Módulo 06](../06-traits-e-generics/08-tipos-associados-e-traits-genericas.md)). E acrescentar um blanket impl a uma trait **pública** que já tem implementações de outros crates é uma mudança incompatível (*breaking change*): as impls deles passam a colidir com a sua.

### Blanket impls para ponteiros

Outro uso comum: fazer a sua trait funcionar através de referências e `Box`. Se uma função recebe `impl Forma` e alguém tem um `&Circulo` ou um `Box<dyn Forma>`, não funciona sozinho. A std resolve isso para as traits dela (`impl<R: Read + ?Sized> Read for &mut R`, `impl<I: Iterator + ?Sized> Iterator for Box<I>`), e você pode fazer o mesmo:

```rust
impl<T: Forma + ?Sized> Forma for &T {
    fn area(&self) -> f64 {
        (**self).area()
    }
}

impl<T: Forma + ?Sized> Forma for Box<T> {
    fn area(&self) -> f64 {
        (**self).area()
    }
}
```

Com isso, `soma_areas(&formas)` aceita `Vec<Circulo>`, `Vec<&Circulo>` e `Vec<Box<dyn Forma>>`. O `?Sized` é o que deixa `T` ser `dyn Forma`.

## Sealed traits: usar sim, implementar não

Às vezes você publica uma trait que outros crates devem **usar** (como bound em funções, como `dyn`), mas não **implementar**. Dois motivos comuns:

- **Evoluir sem quebrar.** Acrescentar um método sem corpo a uma trait pública quebra todo crate que a implementou. Se só você implementa, pode acrescentar métodos numa versão *minor* (o SemVer do [Módulo 08](../08-modulos-e-crates/README.md)).
- **Um conjunto fechado.** Formatos de arquivo que o seu código sabe ler, tipos numéricos que a sua biblioteca suporta. Uma implementação de fora não teria o suporte do resto do código.

Rust não tem uma palavra-chave `sealed` (como Kotlin, Scala ou Java 17). O idioma usa a visibilidade: a trait pública exige uma supertrait que fica num módulo **privado**.

```rust
pub mod formato {
    /// O módulo `selo` é privado: ninguém fora deste crate consegue nomear `Selado`.
    mod selo {
        pub trait Selado {}
    }

    /// Sealed trait: quem está fora do crate pode **usar**, mas não **implementar**.
    pub trait Formato: selo::Selado {
        fn extensao(&self) -> &'static str;
        fn serializar(&self, campos: &[(&str, &str)]) -> String;
    }

    pub struct Json;
    pub struct Csv;

    impl selo::Selado for Json {}
    impl selo::Selado for Csv {}

    impl Formato for Json { /* ... */ }
    impl Formato for Csv { /* ... */ }
}
```

`Selado` é `pub` (senão não poderia aparecer como supertrait de uma trait pública), mas mora em `selo`, que é privado. De fora do crate, ninguém consegue escrever `impl selo::Selado for X`, e portanto ninguém consegue implementar `Formato`. Usar continua normal:

```rust
use extensoes::formato::{Csv, Formato, Json};

let campos = [("nome", "Ana"), ("cidade", "Recife")];
let formatos: [&dyn Formato; 2] = [&Json, &Csv];
for f in formatos {
    println!("--- .{}\n{}", f.extensao(), f.serializar(&campos));
}
```

```text
--- .json
{"nome": "Ana", "cidade": "Recife"}
--- .csv
nome,cidade
Ana,Recife
```

Tentar implementar em outro crate:

```rust
// ❌ NÃO COMPILA (em outro crate)
use extensoes::formato::Formato;

struct Xml;

impl Formato for Xml {
    fn extensao(&self) -> &'static str {
        "xml"
    }
}
```

```text
error[E0277]: the trait bound `Xml: formato::selo::Selado` is not satisfied
  --> src/main.rs:5:18
   |
 5 | impl Formato for Xml {
   |                  ^^^ unsatisfied trait bound
   |
help: the trait `formato::selo::Selado` is not implemented for `Xml`
  --> src/main.rs:3:1
   |
 3 | struct Xml;
   | ^^^^^^^^^^
...
note: required by a bound in `Formato`
  --> src/lib.rs:23:24
   |
23 |     pub trait Formato: selo::Selado {
   |                        ^^^^^^^^^^^^ required by this bound in `Formato`
   = note: `Formato` is a "sealed trait", because to implement it you also need to implement `extensoes::formato::selo::Selado`, which is not accessible; this is usually done to force you to use one of the provided types that already implement it
   = help: the following types implement the trait:
             extensoes::formato::Json
             extensoes::formato::Csv
```

O compilador reconhece o padrão pelo nome e explica: *"`Formato` is a "sealed trait""*. Dentro do próprio crate, o selo não protege nada (o módulo privado é visível para os vizinhos); a garantia é contra os **outros** crates.

Na std, várias traits são seladas assim, e a documentação de cada uma avisa que ela não pode ser implementada fora da biblioteca padrão. Exemplos: `std::slice::SliceIndex` e as extension traits de sistema operacional, como `std::os::unix::fs::PermissionsExt` (que é selada justamente para a std poder acrescentar métodos depois).

> [!TIP]
> Quer o contrário, deixar implementar mas não chamar um método? Isso também existe (um método que recebe um parâmetro de tipo privado), mas é bem mais raro. O livro *Rust for Rustaceans*, cap. 3, discute as variações.

Código completo, com testes, em [`codigo/extensoes`](codigo/extensoes/src/lib.rs).

## Armadilhas comuns

- **Esquecer o `use` da extension trait.** O E0599 com *"is implemented but not in scope"* é isso. Importe a trait (ou o `prelude` da crate).
- **Implementar para `String` em vez de `str`.** Implemente para `str` (ou `[T]` em vez de `Vec<T>`): a deref coercion leva o método para os tipos donos.
- **`Self: Sized` na trait inteira.** Pôr `trait IteratorExt: Iterator + Sized` impede `dyn IteratorExt`. Ponha o `where Self: Sized` em cada método que consome `self`.
- **Blanket impl numa trait pública depois de publicada.** Quebra quem já implementou a trait para um tipo coberto. Decida antes da primeira versão.
- **Achar que o selo protege dentro do crate.** Ele só barra outros crates.
- **Esperar que `eh_palindromo` trate acentos.** `"ô".to_lowercase()` continua `"ô"`, diferente de `"o"`. Comparar texto ignorando acentos exige normalização Unicode (a crate `unicode-normalization`).

## Resumo

- Extension trait (`TextoExt`, `IteratorExt`): trait sua, implementada para um tipo alheio, para acrescentar métodos. Só vale onde a trait foi importada.
- Blanket impl (`impl<I: Iterator> IteratorExt for I {}`) estende uma família inteira de tipos; os métodos ficam como padrão na trait.
- Bounds por método (`where Self: Sized`, `where Self::Item: Hash`) deixam a trait mais usável.
- Blanket impls para `&T` e `Box<T>` fazem a sua trait atravessar ponteiros.
- Sealed trait: supertrait pública num módulo privado; outros crates usam, mas não implementam. Permite acrescentar métodos sem quebrar SemVer.

## Exercícios

Veja o exercício 3 em [`exercicios.md`](exercicios.md).

## Leitura complementar

- [Rust API Guidelines: C-SEALED](https://rust-lang.github.io/api-guidelines/future-proofing.html#c-sealed)
- [RFC 445: extension trait conventions](https://rust-lang.github.io/rfcs/0445-extension-trait-conventions.html)
- [Documentação da crate `itertools`](https://docs.rs/itertools) (um exemplo grande de extension trait com blanket impl)
- *Rust for Rustaceans*, de Jon Gjengset, cap. 3 (*Designing Interfaces*)

---

[← Lição anterior](04-builder-e-idiomas.md) · [Índice do módulo](README.md) · [Próxima lição →](06-tipos-associados-avancados-e-gats.md)

# Módulo 06 — Traits e generics

Generics e traits são o jeito Rust de escrever código que serve para vários tipos: funções e structs com parâmetros de tipo, e *traits* que dizem o que cada tipo sabe fazer. Este módulo explica o que estava por trás de várias receitas dos módulos anteriores (`derive`, `Display`, `From`, `Box<dyn Error>`, a *deref coercion*), compara com templates e herança do C++ e com o *duck typing* do Python, e termina com o minigrep ficando testável com `&mut impl Write`.

**Pré-requisitos:** Módulos 01 a 05 e o [Projeto 1](../../projetos/01-minigrep/README.md) (em especial structs, enums e `impl`, do [Módulo 04](../04-structs-enums-e-patterns/README.md), e a receita de tipos de erro, do [Módulo 05](../05-tratamento-de-erros/README.md)).

**Objetivos.** Ao final do módulo, você vai saber:

- escrever funções, structs e enums genéricos, e entender o custo da monomorfização;
- definir e implementar traits, com métodos padrão, e respeitar a *orphan rule*;
- restringir generics com *trait bounds*, `where` e `impl Trait`, e ler os erros mais comuns;
- usar e derivar as traits essenciais da std (`Debug`, `Clone`, `PartialEq`, `Ord`, `Hash`, `Default`...) e as de conversão (`From`, `TryFrom`, `FromStr`, `AsRef`);
- usar trait objects (`dyn Trait`), saber como funcionam na memória e quando uma trait é *dyn compatible*;
- escolher entre generics, `dyn Trait` e `enum`, e trocar herança por composição.

## Lições

| # | Lição | Assunto |
|---|---|---|
| 01 | [Generics](01-generics.md) | `<T>` em funções, structs, enums e `impl`; turbofish; monomorfização; templates do C++ |
| 02 | [Definindo e implementando traits](02-definindo-e-implementando-traits.md) | `trait`, `impl Trait for Tipo`, métodos padrão, `Display` à mão, trait em escopo, orphan rule e newtype |
| 03 | [Trait bounds, `where` e `impl Trait`](03-trait-bounds-where-e-impl-trait.md) | bounds, `where`, `impl Trait` no argumento e no retorno, E0369/E0599/E0277, `impl` condicional, blanket impls, `&mut impl Write` |
| 04 | [Traits da std e `derive`](04-traits-da-std-e-derive.md) | `Debug`, `Clone`/`Copy`, `PartialEq`/`Eq`, `PartialOrd`/`Ord`, `Hash`, `Default`, supertraits, dunders do Python |
| 05 | [Conversões: `From`, `Into`, `TryFrom`](05-conversoes-from-into-tryfrom.md) | `From`/`Into`, `TryFrom`, `FromStr`, `AsRef`, o `?` e o `From`, *deref coercion* |
| 06 | [Trait objects e `dyn`](06-trait-objects-e-dyn.md) | `Box<dyn Trait>`, fat pointer e vtable, `Sized`, o `'static` de `Box<dyn Trait>`, *dyn compatibility*, upcasting e `Any` |
| 07 | [Estático x dinâmico, e composição](07-estatico-x-dinamico-e-composicao.md) | "Rust é OO?", composição e decorador, supertraits, generics x `dyn` x `enum` |
| 08 | [Tipos associados e traits genéricas](08-tipos-associados-e-traits-genericas.md) | `type Item` x `From<T>`, coerência e E0119, `Add<Rhs = Self>`, sintaxe qualificada |

## Exercícios

[`exercicios.md`](exercicios.md): 6 exercícios, o último retomando o minigrep. Soluções em [`solucoes/06-traits-e-generics`](../../solucoes/06-traits-e-generics/).

## Código

Os exemplos completos estão em [`codigo/`](codigo/):

- [`monomorfizacao`](codigo/monomorfizacao/): uma função genérica x uma com `dyn`, para inspecionar com `nm` (lições 01 e 06);
- [`resumo`](codigo/resumo/): traits, métodos padrão, `Display` à mão, newtype (lição 02);
- [`relatorio`](codigo/relatorio/): bounds, `where`, `impl Trait`, `impl` condicional, `Write`/`BufRead` testados com `Vec<u8>` (lição 03);
- [`traits-da-std`](codigo/traits-da-std/): `derive`, ordem, `Hash` coerente com `PartialEq`, `Default` (lição 04);
- [`conversoes`](codigo/conversoes/): `TryFrom`, `FromStr`, `From`, `Into` e `AsRef` (lição 05);
- [`formas`](codigo/formas/): trait objects, tamanhos, upcasting e `Any`, generics x `dyn` (lições 06 e 07);
- [`composicao`](codigo/composicao/): um decorador de `Write` e uma supertrait (lição 07).

Para rodar:

```bash
cd codigo/formas
cargo run
cargo test
```

## Referências principais

- [The Rust Programming Language, cap. 10: Generic Types, Traits, and Lifetimes](https://doc.rust-lang.org/stable/book/ch10-00-generics.html)
- [The Rust Programming Language, cap. 18: Object-Oriented Programming Features](https://doc.rust-lang.org/stable/book/ch18-00-oop.html) e [20.2 Advanced Traits](https://doc.rust-lang.org/stable/book/ch20-02-advanced-traits.html)
- [The Rust Reference: Traits](https://doc.rust-lang.org/reference/items/traits.html)
- [Comprehensive Rust: Methods and Traits, Generics e Standard Library Traits](https://google.github.io/comprehensive-rust/)
- [Effective Rust, Items 2, 5, 10 e 12](https://effective-rust.com/)

---

[← Anterior: Projeto 1](../../projetos/01-minigrep/README.md) · [Índice do curso](../../README.md) · [Próximo módulo →](../07-colecoes-iteradores-e-closures/README.md)

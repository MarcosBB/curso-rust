# Módulo 13 — Tópicos avançados de tipos e traits

O Módulo 06 ensinou a usar traits e generics. Este módulo mostra como usá-los para **projetar** APIs: fazer o compilador recusar unidades misturadas, transições de estado fora de ordem e campos obrigatórios esquecidos; acrescentar métodos a tipos alheios; e entender as regras de lifetimes que ficaram de fora do Módulo 03 (variância, HRTB e a captura em `impl Trait` da edition 2024). Quase tudo aqui tem custo zero em tempo de execução: o trabalho todo é do compilador.

**Pré-requisitos:** Módulos 03 (lifetimes), 04 (newtype, enums de estado), 06 (traits, generics, `dyn`, tipos associados) e 07 (closures e iteradores). O Módulo 12 ajuda na lição 06 (`async fn` em traits usa o mesmo mecanismo do RPITIT).

**Objetivos.** Ao final do módulo, você vai saber:

- sobrecarregar operadores com `std::ops`, inclusive por referência e com tipos diferentes dos dois lados;
- usar newtypes, aliases, o tipo `!` e tipos sem tamanho (`?Sized`) sabendo o que cada um garante;
- modelar estados como tipos com `PhantomData` (typestate) e testar que um uso errado não compila;
- escrever builders, extension traits, blanket impls e sealed traits;
- usar constantes associadas, RPITIT e GATs (o *lending iterator*);
- explicar a variância, escrever `T: 'a` e HRTBs (`for<'a>`), e controlar a captura de `impl Trait` com `use<..>`.

## Lições

| # | Lição | Assunto |
|---|---|---|
| 01 | [Sobrecarga de operadores](01-sobrecarga-de-operadores.md) | `Add`, `Mul<f64>`, `f64 * Vetor`, `AddAssign`, `Index`, operadores por referência, `Sum` |
| 02 | [Newtype, aliases, `!` e tipos sem tamanho](02-newtype-aliases-never-e-dsts.md) | unidades que não se misturam, `type`, never type, DSTs, *fat pointers*, `?Sized`, coerções |
| 03 | [`PhantomData` e typestate](03-phantomdata-e-typestate.md) | `Id<Usuario>`, a armadilha do `derive`, `Pedido<Carrinho>` → `Pedido<Pago>`, doctests `compile_fail` |
| 04 | [Builders e outros idiomas](04-builder-e-idiomas.md) | `Default` + `..Default::default()`, builder com `self` ou `&mut self`, `#[must_use]`, builder typestate |
| 05 | [Blanket impls, extension e sealed traits](05-blanket-extension-e-sealed.md) | `TextoExt`, `IteratorExt`, blanket impls para `&T`/`Box<T>`, selar uma trait |
| 06 | [Tipos associados avançados e GATs](06-tipos-associados-avancados-e-gats.md) | constantes associadas, `Item: Display`, RPITIT, *lending iterator* com `type Item<'a>` |
| 07 | [Variância e lifetimes avançados](07-variancia-e-lifetimes-avancados.md) | covariância, invariância atrás de `&mut`, `T: 'a`, `'a: 'b`, `PhantomData` e `Send` |
| 08 | [HRTB e captura em `impl Trait`](08-hrtb-e-captura-em-impl-trait.md) | `for<'a>`, a closure do Módulo 07, regras da 2024, `use<..>` |

As lições 01 a 05 são de projeto de API e podem ser lidas em qualquer ordem. As lições 06 a 08 são mais densas e se apoiam umas nas outras.

## Exercícios

[`exercicios.md`](exercicios.md): 5 exercícios. Soluções em [`solucoes/13-tipos-e-traits-avancados`](../../solucoes/13-tipos-e-traits-avancados/).

## Código

Os exemplos completos estão em [`codigo/`](codigo/), sem dependências externas:

- [`operadores`](codigo/operadores/): `Vetor`, `Polinomio` por referência e `Matriz` com `Index` (lição 01);
- [`tipos-e-tamanhos`](codigo/tipos-e-tamanhos/): `Metros` x `Pes`, aliases, `!`, `?Sized` e os tamanhos dos *fat pointers* (lição 02);
- [`typestate-e-builder`](codigo/typestate-e-builder/): `Id<T>`, o pedido com typestate e o builder de requisição, com doctests `compile_fail` (lições 03 e 04);
- [`extensoes`](codigo/extensoes/): extension traits, blanket impl e a trait selada `Formato` (lição 05);
- [`gats`](codigo/gats/): constantes associadas, RPITIT e o *lending iterator* (lição 06);
- [`variancia-e-hrtb`](codigo/variancia-e-hrtb/): variância, `T: 'a`, HRTB e `use<..>` (lições 07 e 08).

Para rodar:

```bash
cd codigo/typestate-e-builder
cargo run
cargo test     # inclui os doctests que exigem que o código NÃO compile
```

## Referências principais

- [The Rust Programming Language, 20.2 Advanced Traits](https://doc.rust-lang.org/stable/book/ch20-02-advanced-traits.html) e [20.3 Advanced Types](https://doc.rust-lang.org/stable/book/ch20-03-advanced-types.html)
- [The Rust Reference: Subtyping and Variance](https://doc.rust-lang.org/reference/subtyping.html), [Trait and lifetime bounds](https://doc.rust-lang.org/reference/trait-bounds.html)
- [The Rustonomicon: Subtyping and Variance](https://doc.rust-lang.org/nomicon/subtyping.html), [PhantomData](https://doc.rust-lang.org/nomicon/phantom-data.html), [HRTB](https://doc.rust-lang.org/nomicon/hrtb.html)
- [Edition Guide: RPIT lifetime capture rules](https://doc.rust-lang.org/edition-guide/rust-2024/rpit-lifetime-capture.html)
- [Documentação de `std::marker`](https://doc.rust-lang.org/std/marker/index.html) e de [`std::ops`](https://doc.rust-lang.org/std/ops/index.html)
- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/) (C-NEWTYPE, C-BUILDER, C-SEALED, C-OVERLOAD)
- *Rust for Rustaceans*, de Jon Gjengset, caps. 1 e 3

O próximo módulo, [Macros](../14-macros/README.md), mostra como gerar código: é o que as crates `bon` e `typed-builder` (lição 04) e `async-trait` (Módulo 12) fazem por baixo.

---

[← Anterior: Projeto 5](../../projetos/05-chat-tcp/README.md) · [Índice do curso](../../README.md) · [Próximo: Módulo 14 →](../14-macros/README.md)

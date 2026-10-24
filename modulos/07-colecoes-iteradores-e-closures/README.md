# Módulo 07 — Coleções, iteradores e closures

Este módulo aprofunda as três ferramentas que você usa em quase todo programa Rust e que o Módulo 01 só apresentou como receita. Primeiro as coleções da std, com o custo de cada operação e a *Entry API*. Depois as closures: como capturam o ambiente, o `move` e as traits `Fn`, `FnMut` e `FnOnce`. Por fim, os iteradores: a trait `Iterator`, o que o `for` faz por baixo, os adaptadores preguiçosos e como escrever o seu próprio. No caminho, o minigrep do Projeto 1 ganha uma versão com iteradores.

**Pré-requisitos:** Módulos 01 a 05 e o [Módulo 06](../06-traits-e-generics/README.md) (generics, traits, `impl Trait`, `dyn Trait` e tipos associados). O kit do [Módulo 01](../01-tour-da-sintaxe/README.md) (lições 07 e 08) é o ponto de partida.

**Objetivos.** Ao final do módulo, você vai saber:

- escolher entre `Vec`, `VecDeque`, `HashMap`, `BTreeMap`, `HashSet`, `BTreeSet` e `BinaryHeap`, sabendo o custo de cada operação;
- remover, ordenar e atualizar sem brigar com o *borrow checker* (`retain`, `sort_by`, `total_cmp`, *Entry API*);
- prever como uma closure captura cada variável, quando usar `move` e qual das traits `Fn*` ela implementa;
- guardar e devolver closures (`impl Fn`, `Box<dyn Fn>`, genéricos);
- explicar o `for` em termos de `IntoIterator` e `Iterator`, e a diferença entre `iter`, `iter_mut` e `into_iter`;
- encadear adaptadores e consumidores, e escolher o tipo de saída do `collect`;
- implementar `Iterator` e `IntoIterator` para os seus tipos.

## Lições

| # | Lição | Assunto |
|---|---|---|
| 01 | [`Vec` em profundidade](01-vec-em-profundidade.md) | capacidade, remover (`retain`, `drain`, `extract_if`, `dedup`), ordenar (`sort_by`, `Reverse`, `total_cmp`), `binary_search`, `windows`/`chunks` |
| 02 | [`HashMap` e a *Entry API*](02-hashmap-e-entry-api.md) | `Hash + Eq`, posse no `insert`, `Borrow` no `get`, `entry`/`or_insert`/`or_default`/`and_modify`, `retain`, `get_disjoint_mut`, ordem arbitrária |
| 03 | [Outras coleções](03-outras-colecoes.md) | `HashSet`, `BTreeMap`/`BTreeSet`, `VecDeque`, `BinaryHeap`, `LinkedList`, qual coleção usar |
| 04 | [Closures e captura](04-closures-e-captura.md) | captura por `&`/`&mut`/valor, a closure como struct, captura disjunta, `move`, E0373, closures com referências |
| 05 | [`Fn`, `FnMut` e `FnOnce`](05-fn-fnmut-fnonce.md) | as três traits, que bound pedir, E0525 x E0507, ponteiros `fn`, closures em structs, `impl Fn` x `Box<dyn Fn>` |
| 06 | [`Iterator` e `IntoIterator`](06-iterator-e-intoiterator.md) | `next`, o `for` desaçucarado, `iter`/`iter_mut`/`into_iter`, o `&&x` do `filter`, `by_ref`, `impl IntoIterator` |
| 07 | [Adaptadores, consumidores e `collect`](07-adaptadores-e-consumidores.md) | *laziness* e o modelo *pull*, adaptadores, consumidores, `collect` e `FromIterator`, iterador x laço |
| 08 | [Implementando seu próprio iterador](08-implementando-seu-iterador.md) | `impl Iterator`, iteradores que emprestam, `IntoIterator for &T`, `-> impl Iterator`, `from_fn`/`successors` |

## Exercícios

[`exercicios.md`](exercicios.md): 6 exercícios, incluindo a retomada do minigrep do Projeto 1. Soluções em [`solucoes/07-colecoes-iteradores-e-closures`](../../solucoes/07-colecoes-iteradores-e-closures/).

## Código

Os exemplos completos estão em [`codigo/`](codigo/):

- [`estoque`](codigo/estoque/): `HashMap` com *Entry API*, `retain`, `BTreeMap`, `HashSet` e `BinaryHeap` com `Reverse` (lições 01 a 03);
- [`closures-na-pratica`](codigo/closures-na-pratica/): bounds `FnMut` e `FnOnce`, fábrica de contadores, composição e um validador com `Vec<Box<dyn Fn>>` (lições 04 e 05);
- [`relatorio-de-notas`](codigo/relatorio-de-notas/): a ordem de execução de uma cadeia (`cargo run`) e um relatório feito só com iteradores (lições 06 e 07);
- [`iterador-proprio`](codigo/iterador-proprio/): `Regressiva`, `Fibonacci`, `Palavras<'a>`, `Playlist` com `IntoIterator` e iteradores com `successors` (lição 08).

Para rodar:

```bash
cd codigo/relatorio-de-notas
cargo run
cargo test
```

## Referências principais

- [The Rust Programming Language, cap. 8: Common Collections](https://doc.rust-lang.org/stable/book/ch08-00-common-collections.html)
- [The Rust Programming Language, cap. 13: Iterators and Closures](https://doc.rust-lang.org/stable/book/ch13-00-functional-features.html)
- Documentação de [`std::collections`](https://doc.rust-lang.org/std/collections/index.html) e de [`std::iter`](https://doc.rust-lang.org/std/iter/index.html)
- [Comprehensive Rust: Closures e Iterators](https://google.github.io/comprehensive-rust/)
- [Effective Rust, Item 9: iterator transforms](https://effective-rust.com/iterators.html)

---

[← Módulo anterior](../06-traits-e-generics/README.md) · [Índice do curso](../../README.md) · [Próximo módulo →](../08-modulos-e-crates/README.md)

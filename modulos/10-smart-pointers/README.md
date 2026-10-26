# Módulo 10 — Smart pointers e mutabilidade interior

Até aqui, todo valor teve um dono só, e toda mudança passou por um `&mut` verificado na compilação. Este módulo mostra as ferramentas para os casos em que isso não basta: `Box` para tipos recursivos e valores no heap, `Rc` para vários donos, `Cell` e `RefCell` para mudar algo através de uma referência compartilhada, `Weak` para não vazar com ciclos e `Cow` para copiar só quando precisa. Também cumpre duas promessas antigas: escrever o seu próprio `Deref` e o seu próprio `Drop`.

**Pré-requisitos:** Módulos 02 e 03 (posse, empréstimos e `Drop`), 06 (traits, `dyn` e a apresentação de `Deref`) e o [Projeto 2](../../projetos/02-csvlite/README.md), cujo benchmark a lição 06 retoma.

**Objetivos.** Ao final do módulo, você vai saber:

- usar `Box` para alocar no heap e para escrever tipos recursivos;
- implementar `Deref`, `DerefMut` e `Drop` num tipo seu, e saber quando isso vale a pena;
- compartilhar a posse com `Rc` e entender o custo dos contadores;
- mudar um valor através de `&self` com `Cell`, `RefCell` e `OnceCell`, e evitar o pânico do `RefCell`;
- reconhecer um ciclo de `Rc` e quebrá-lo com `Weak`;
- evitar cópias desnecessárias com `Cow`.

## Lições

| # | Lição | Assunto |
|---|---|---|
| 01 | [`Box` e tipos recursivos](01-box-e-tipos-recursivos.md) | heap com dono único, layout, E0072, árvore de expressões, `Box<[T]>`, `large_enum_variant`, `unique_ptr` |
| 02 | [`Deref` e `Drop` próprios](02-deref-e-drop.md) | `MeuBox<T>`, E0614, *deref coercion*, `Deref` não é herança, guarda RAII, E0040 |
| 03 | [`Rc`: posse compartilhada](03-rc-posse-compartilhada.md) | `Rc::clone`, contadores, só leitura (E0596), `make_mut`, `shared_ptr`, refcount do Python, `Rc` não é `Send` |
| 04 | [`Cell` e `RefCell`: mutabilidade interior](04-cell-e-refcell.md) | `Cell`, `RefCell`, `OnceCell`, `Rc<RefCell<T>>`, o pânico em runtime, temporários na edition 2024, `mutable` do C++ |
| 05 | [`Weak` e ciclos de referência](05-weak-e-ciclos.md) | o ciclo que vaza, `Weak`, `upgrade`, árvore com pai fraco, GC de ciclos do Python, qual ponteiro escolher |
| 06 | [`Cow`: copiar só quando precisa](06-cow.md) | `Borrowed`/`Owned`, `to_mut`, `from_utf8_lossy`, o parser do csvlite sem cópia, medido com criterion |

## Exercícios

[`exercicios.md`](exercicios.md): 5 exercícios. Soluções em [`solucoes/10-smart-pointers`](../../solucoes/10-smart-pointers/).

## Código

Os exemplos completos estão em [`codigo/`](codigo/):

- [`box-e-recursao`](codigo/box-e-recursao/): tamanhos de `Box` e uma árvore de expressões (lição 01);
- [`meu-box`](codigo/meu-box/): um ponteiro feito à mão com `Deref`/`DerefMut`/`Drop` e uma guarda de transação (lição 02);
- [`rc-e-refcell`](codigo/rc-e-refcell/): `Rc`, `make_mut`, um logger com `&self`, `OnceCell` e `Rc<RefCell<T>>` (lições 03 e 04);
- [`arvore-weak`](codigo/arvore-weak/): o ciclo que vaza, a versão com `Weak` e uma árvore de pastas (lição 05);
- [`cow-csv`](codigo/cow-csv/): campos de CSV com `Cow`, com testes e benchmark (lição 06).

Para rodar:

```bash
cd codigo/arvore-weak
cargo run
cargo test
```

## Referências principais

- [The Rust Programming Language, cap. 15: Smart Pointers](https://doc.rust-lang.org/stable/book/ch15-00-smart-pointers.html)
- Documentação de [`std::boxed`](https://doc.rust-lang.org/std/boxed/index.html), [`std::rc`](https://doc.rust-lang.org/std/rc/index.html), [`std::cell`](https://doc.rust-lang.org/std/cell/index.html) e [`std::borrow::Cow`](https://doc.rust-lang.org/std/borrow/enum.Cow.html)
- [Comprehensive Rust: Smart Pointers](https://google.github.io/comprehensive-rust/smart-pointers.html) e [Interior Mutability](https://google.github.io/comprehensive-rust/borrowing/interior-mutability.html)
- [Learning Rust With Entirely Too Many Linked Lists](https://rust-unofficial.github.io/too-many-lists/) (escrito numa edition antiga; leia como exercício, não como referência de estilo)

O próximo módulo leva tudo isso para várias threads: `Arc` é o `Rc` com contador atômico, e `Mutex`/`RwLock` são os `RefCell` que bloqueiam em vez de dar pânico.

---

[← Anterior: Projeto 3](../../projetos/03-tarefas/README.md) · [Índice do curso](../../README.md) · [Próximo módulo →](../11-concorrencia/README.md)

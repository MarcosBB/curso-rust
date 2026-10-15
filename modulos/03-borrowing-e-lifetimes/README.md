# Módulo 03 — Borrowing e lifetimes

O Módulo 02 mostrou que todo valor tem um único dono. Este módulo mostra como **usar** um valor sem ser dono dele: referências, as regras que o *borrow checker* aplica a elas e os *lifetimes* que ligam cada referência ao dado para o qual ela aponta. É o módulo em que mais gente trava. Por isso, cada regra vem com o erro real do compilador, a explicação de cada rótulo e a correção.

**Pré-requisitos:** Módulos [00](../00-introducao/README.md), [01](../01-tour-da-sintaxe/README.md) e [02](../02-ownership/README.md) (move, `Copy`/`Clone`, `Drop`, `String` x `&str`, UTF-8).

**Objetivos.** Ao final do módulo, você vai saber:

- usar `&T` e `&mut T` e explicar a regra "vários leitores **ou** um escritor";
- ler os erros E0502, E0499, E0506, E0505, E0596, E0597, E0515, E0716, E0106 e E0621 com o modelo de permissões R/W/O;
- trabalhar com slices (`&str`, `&[T]`, `&mut [T]`) e pegar empréstimos disjuntos com `split_at_mut` e `get_disjoint_mut`;
- anotar lifetimes em funções e structs quando a elisão não basta, usar `'_` e entender `'static`;
- decidir se um erro de empréstimo indica um programa inseguro ou uma limitação do checker, e aplicar a correção idiomática (incluindo `HashMap::entry`).

## Lições

| # | Lição | Assunto |
|---|---|---|
| 01 | [Referências e empréstimos](01-referencias-e-emprestimos.md) | `&T`, `&mut T`, desreferência, a regra de aliasing, permissões R/W/O, comparação com C++ e Python |
| 02 | [O borrow checker e NLL](02-borrow-checker-e-nll.md) | regiões de empréstimo, E0502/E0499/E0506/E0596, *reborrow*, *two-phase borrows*, *iterator invalidation* |
| 03 | [Slices em profundidade](03-slices-em-profundidade.md) | *fat pointers*, ranges, `&str`/`&[T]` em parâmetros, `&mut [T]`, `split_at_mut`, `get_disjoint_mut` |
| 04 | [Lifetimes em funções](04-lifetimes-em-funcoes.md) | referências penduradas, E0106, `<'a>`, a assinatura como contrato, E0621 |
| 05 | [Elisão e lifetime anônimo](05-elisao-e-lifetime-anonimo.md) | as três regras de elisão, `'_`, lint `mismatched_lifetime_syntaxes` |
| 06 | [Lifetimes em structs e `'static`](06-lifetimes-em-structs-e-static.md) | `struct Excerpt<'a>`, `impl<'a>`, `&str` x `&'a str`, `'static` e seus mitos |
| 07 | [Brigando menos com o borrow checker](07-brigando-menos-com-o-borrow-checker.md) | inseguro x limitação, catálogo de correções, "busca ou insere" com `entry`, `clone()` consciente |

Variância, HRTB (`for<'a>`), restrições `T: 'a` e a captura de lifetimes em `impl Trait` (`use<..>`) ficam para o Módulo 13.

## Exercícios

[`exercicios.md`](exercicios.md): 6 exercícios. Soluções em [`solucoes/03-borrowing-e-lifetimes`](../../solucoes/03-borrowing-e-lifetimes/).

## Código

Os exemplos completos estão em [`codigo/`](codigo/):

| Projeto | Lições |
|---|---|
| [`emprestimos`](codigo/emprestimos/) | 01 e 02 |
| [`slices`](codigo/slices/) | 03 |
| [`lifetimes`](codigo/lifetimes/) | 04 a 06 |
| [`padroes-borrow`](codigo/padroes-borrow/) | 07 |

Para rodar:

```bash
cd codigo/slices
cargo run
cargo test
```

## Referências principais

- [The Rust Programming Language, 4.2 References and Borrowing](https://doc.rust-lang.org/stable/book/ch04-02-references-and-borrowing.html), [4.3 The Slice Type](https://doc.rust-lang.org/stable/book/ch04-03-slices.html) e [10.3 Validating References with Lifetimes](https://doc.rust-lang.org/stable/book/ch10-03-lifetime-syntax.html)
- [Brown University: Rust Book com quizzes](https://rust-book.cs.brown.edu/ch04-02-references-and-borrowing.html) (modelo de permissões R/W/O)
- [Comprehensive Rust: Borrowing](https://google.github.io/comprehensive-rust/borrowing.html) e [Lifetimes](https://google.github.io/comprehensive-rust/lifetimes.html)
- [Reference: Lifetime elision](https://doc.rust-lang.org/reference/lifetime-elision.html)
- [pretzelhammer: Common Rust Lifetime Misconceptions](https://github.com/pretzelhammer/rust-blog/blob/master/posts/common-rust-lifetime-misconceptions.md)

---

[← Módulo anterior](../02-ownership/README.md) · [Índice do curso](../../README.md) · [Próximo módulo →](../04-structs-enums-e-patterns/README.md)

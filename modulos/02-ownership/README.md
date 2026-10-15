# Módulo 02 — Ownership (o coração do Rust)

Este é o módulo que separa o Rust das outras linguagens. Em vez de deixar a liberação de memória com você (C), com destrutores que não impedem uso depois da liberação (C++) ou com um coletor de lixo (Python), o Rust dá a cada valor **um único dono** e verifica na compilação quem libera o quê, e quando. Quase todo erro que você vai encontrar nas primeiras semanas vem daqui, e por isso vale a pena ir devagar.

**Pré-requisitos:** [Módulo 01](../01-tour-da-sintaxe/README.md) (variáveis, funções, `Vec`, `for`, closures e testes).

**Objetivos.** Ao final do módulo, você vai saber:

- explicar o que vai para a stack e para o heap, e como uma `String` é organizada na memória;
- aplicar as três regras de ownership e prever onde acontece um *move*;
- resolver o erro E0382 do jeito certo: reordenando, emprestando ou clonando;
- distinguir `Copy` de `Clone`, e evitar clonar só para calar o compilador;
- prever quando e em que ordem os valores são descartados (`Drop`);
- escolher entre `String` e `&str`, e trabalhar com texto UTF-8 sem *panic*.

## Lições

| # | Lição | Assunto |
|---|---|---|
| 01 | [Quem libera a memória?](01-quem-libera-a-memoria.md) | stack x heap, layout de `String`, as respostas de C, C++, Python e Rust |
| 02 | [Regras de ownership e move](02-regras-de-ownership-e-move.md) | as três regras, *move*, E0382, prévia de `&`, *partial move*, E0507 |
| 03 | [Copy e Clone](03-copy-e-clone.md) | tipos `Copy`, `derive`, E0204, `clone()` como custo visível |
| 04 | [Drop e RAII](04-drop-e-raii.md) | ordem de descarte, `drop()`, `let _ =` x `let _x =`, E0040, E0184 |
| 05 | [`String` e `&str`](05-string-e-str.md) | dona x vista emprestada, conversões, assinaturas, concatenação |
| 06 | [Strings UTF-8](06-strings-utf8.md) | bytes x `char` x grafemas, por que `s[0]` não compila, fatias seguras |

## Exercícios

[`exercicios.md`](exercicios.md): 6 exercícios. Soluções em [`solucoes/02-ownership`](../../solucoes/02-ownership/).

## Código

Os exemplos completos estão em [`codigo/`](codigo/):

- [`memoria-da-string`](codigo/memoria-da-string/): tamanhos, `len` x `capacity`, endereços do buffer num move e num clone (lições 01 a 03);
- [`ordem-do-drop`](codigo/ordem-do-drop/): o tipo `Barulhento` mostrando cada regra de descarte (lição 04);
- [`texto`](codigo/texto/): `String` x `&str` e as três visões de um texto UTF-8 (lições 05 e 06).

Para rodar:

```bash
cd codigo/ordem-do-drop
cargo run
```

## Referências principais

- [The Rust Programming Language, cap. 4 (Ownership)](https://doc.rust-lang.org/stable/book/ch04-00-understanding-ownership.html) e [8.2 (Strings)](https://doc.rust-lang.org/stable/book/ch08-02-strings.html)
- [Brown "Rust Book Experiment", cap. 4](https://rust-book.cs.brown.edu/ch04-00-understanding-ownership.html): a versão com o modelo de permissões
- [Comprehensive Rust: Memory Management](https://google.github.io/comprehensive-rust/memory-management.html)
- [Rust by Example: Scoping rules](https://doc.rust-lang.org/rust-by-example/scope.html)

---

[← Módulo anterior](../01-tour-da-sintaxe/README.md) · [Índice do curso](../../README.md) · [Próximo módulo →](../03-borrowing-e-lifetimes/README.md)

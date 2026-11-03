# Módulo 15 — Unsafe e FFI

> Última revisão: 2026-11-03 (Rust 1.99, Miri nightly 2026-11-02, cc 1.11, bindgen 0.73, PyO3 0.30)

Até aqui, o compilador provou tudo por você. Este módulo trata do código que ele não consegue provar: as estruturas de baixo nível por trás de `Vec`, `split_at_mut` e `Cell`, e a fronteira com outras linguagens. Você vai ver exatamente o que `unsafe` libera (e o que não libera), escrever abstrações seguras sobre ele, conhecer o comportamento indefinido e caçá-lo com o Miri, e fazer Rust conversar com C nos dois sentidos, com Python e com C++.

**Pré-requisitos:** Módulos 02 e 03 (ownership, empréstimos, slices), 08 (crates, `build.rs`, tipos de crate), 10 (`Box`, `Cell`/`RefCell`) e 11 (`Send`/`Sync`, atômicos, `static`). Para as lições 05 e 06, um compilador C instalado (clang, gcc ou MSVC). Para o Miri, o toolchain nightly (a lição 04 mostra como instalar).

**Objetivos.** Ao final do módulo, você vai saber:

- listar os cinco superpoderes de `unsafe` e documentar cada uso com `// SAFETY:` e `# Safety`;
- usar ponteiros crus (`&raw const`/`&raw mut`, `add`, `Box::into_raw`, `NonNull`) sem criar referências inválidas;
- escrever uma função segura sobre código `unsafe` e manter as invariantes do módulo;
- reconhecer comportamento indefinido e rodar o Miri;
- chamar C a partir de Rust (`unsafe extern "C"`, `cc`, `CStr`/`CString`, `bindgen`) e expor Rust para C (`#[unsafe(no_mangle)]`, `cdylib`/`staticlib`, `catch_unwind`);
- escolher entre `ctypes`, PyO3 e `cxx` para integrar com Python e C++.

## Lições

| # | Lição | Assunto |
|---|---|---|
| 01 | [O que `unsafe` permite](01-o-que-unsafe-permite.md) | os cinco superpoderes, `unsafe fn` e `unsafe_op_in_unsafe_fn`, traits `unsafe`, `union`, `// SAFETY:`, lints, C++ x Rust |
| 02 | [Ponteiros crus](02-ponteiros-crus.md) | `&raw const`/`&raw mut`, nulo, `add`, `read`/`write`, `Box::into_raw`, `NonNull`, `static mut`, `packed`, `Send` |
| 03 | [Abstrações seguras sobre `unsafe`](03-abstracoes-seguras.md) | `split_at_mut`, invariantes e a fronteira do módulo, `UnsafeCell`, um `Box` mínimo com `unsafe impl Send` |
| 04 | [Comportamento indefinido e Miri](04-comportamento-indefinido-e-miri.md) | o que é UB, sete exemplos reais, Miri, Stacked x Tree Borrows, sanitizers do C++ |
| 05 | [Chamando C a partir de Rust](05-chamando-c.md) | `unsafe extern "C"`, `safe fn`, `#[repr(C)]`, `CStr`/`CString`/`c"..."`, `build.rs` com `cc`, callbacks, `bindgen` |
| 06 | [Expondo Rust para C](06-expondo-rust-para-c.md) | `#[unsafe(no_mangle)]`, `cdylib`/`staticlib`, cabeçalho e `cbindgen`, ponteiros opacos, `catch_unwind` |
| 07 | [Rust com Python e C++](07-python-e-cpp.md) | API C do CPython, `ctypes`, PyO3 e maturin, `cxx`, qual ferramenta usar |

## Exercícios

[`exercicios.md`](exercicios.md): 5 exercícios. Soluções em [`solucoes/15-unsafe-e-ffi`](../../solucoes/15-unsafe-e-ffi/).

## Código

Os exemplos completos estão em [`codigo/`](codigo/):

- [`ponteiros-crus`](codigo/ponteiros-crus/): criar e desreferenciar, nulo, aritmética, `Box::into_raw`, `static mut` e `packed` (lições 01 e 02);
- [`abstracoes-seguras`](codigo/abstracoes-seguras/): `dividir_mut`, `Leitor`, `Celula` e `MeuBox`, com testes que passam no Miri (lição 03);
- [`chamando-c`](codigo/chamando-c/): um `.c` compilado pelo `build.rs` com `cc`, declarações cruas e wrappers seguros, strings e callbacks (lição 05);
- [`rust-para-c`](codigo/rust-para-c/): biblioteca `cdylib`/`staticlib` com cabeçalho, um programa C e um script Python com `ctypes` (lições 06 e 07).

Para rodar:

```bash
cd codigo/abstracoes-seguras
cargo test
cargo +nightly miri test     # precisa do Miri (lição 04)

cd ../rust-para-c
cargo build --release
cc c/main.c -Iinclude target/release/librust_para_c.a -o main && ./main
python3 python/usar_ctypes.py
```

Os endereços de memória impressos pelos exemplos mudam a cada execução. Os exemplos de comportamento indefinido da lição 04 ficam só na lição (eles não entram em `codigo/`), e as saídas mostradas foram obtidas num Mac com Apple Silicon: em outra máquina, ou com outra versão do compilador, um programa com UB pode se comportar de outro jeito. É esse o ponto da lição.

## Referências principais

- [The Rust Programming Language, 20.1: Unsafe Rust](https://doc.rust-lang.org/stable/book/ch20-01-unsafe-rust.html)
- [The Rustonomicon](https://doc.rust-lang.org/nomicon/) (a referência para quem escreve `unsafe`)
- [The Rust Reference: Behavior considered undefined](https://doc.rust-lang.org/reference/behavior-considered-undefined.html)
- [Edition Guide: mudanças de `unsafe` na 2024](https://doc.rust-lang.org/edition-guide/rust-2024/index.html) (`unsafe extern`, atributos `unsafe`, `unsafe_op_in_unsafe_fn`, `static mut`)
- Documentação de [`std::ptr`](https://doc.rust-lang.org/std/ptr/index.html) e [`std::ffi`](https://doc.rust-lang.org/std/ffi/index.html)
- [Miri](https://github.com/rust-lang/miri)
- [cc](https://docs.rs/cc), [bindgen](https://rust-lang.github.io/rust-bindgen/), [cbindgen](https://github.com/mozilla/cbindgen), [PyO3](https://pyo3.rs/), [cxx](https://cxx.rs/)

---

[← Módulo anterior](../14-macros/README.md) · [Índice do curso](../../README.md) · [Próximo módulo →](../16-ecossistema-na-pratica/README.md)

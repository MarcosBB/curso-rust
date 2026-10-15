# 10 — Kit: testes

> [!NOTE]
> Lição do **kit mínimo**: o suficiente para escrever testes unitários e rodar os exercícios do curso. Testes de integração (`tests/`), doctests, `#[should_panic]`, testes que devolvem `Result` e `assert_matches!` ficam para o Módulo 09.

**Objetivos.** Ao final, você vai saber:

- escrever um teste com `#[test]` dentro de um módulo `#[cfg(test)]`;
- usar `assert!`, `assert_eq!` e `assert_ne!`, com mensagem;
- rodar os testes com `cargo test` e ler a saída de uma falha.

## O teste que o `cargo new --lib` gera

```bash
cargo new --lib calculos
```

O `src/lib.rs` já vem com um teste:

```rust
pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
```

Cada peça tem um papel:

- **`mod tests`**: um módulo comum, escrito no próprio arquivo (veja a [lição anterior](09-kit-modulos-lib-e-main.md)). Os testes unitários ficam **no mesmo arquivo** do código que testam.
- **`#[cfg(test)]`**: um atributo que faz o módulo só ser compilado em `cargo test`. No `cargo build`, ele não existe, e o executável não leva os testes.
- **`use super::*`**: traz tudo do módulo pai para dentro do `tests`. Como `tests` está **dentro** do módulo testado, ele enxerga até os itens privados. Dá para testar uma função auxiliar sem torná-la `pub`.
- **`#[test]`**: marca a função como teste. Uma função de teste não recebe argumentos e não devolve nada.

## As macros de asserção

```rust
assert!(lista.is_empty());                 // a condição precisa ser true
assert_eq!(media(&[6.0, 8.0]), Some(7.0)); // os dois lados precisam ser iguais
assert_ne!(conceito(9.0), 'D');            // os dois lados precisam ser diferentes
```

Um teste **falha quando dá *panic***, e é isso que essas macros fazem quando a condição não vale. Um `unwrap()` num `None` ou um índice fora dos limites também derrubam o teste, pelo mesmo motivo.

Todas aceitam uma mensagem opcional no formato do `format!`, mostrada só na falha:

```rust
assert_eq!(conceito(4.9), 'D', "4.9 ainda é reprovação");
assert!(total > 0, "total deveria ser positivo, mas é {total}");
```

Para comparar, `assert_eq!` exige que os valores possam ser comparados com `==` e mostrados com `{:?}`. Números, textos, `Vec`, tuplas e `Option` já atendem. Seus próprios tipos vão precisar de `#[derive(PartialEq, Debug)]`, que você verá nos Módulos 04 e 06.

> [!TIP]
> A ordem dos argumentos do `assert_eq!` não muda nada. A saída de falha chama os dois lados de `left` e `right`, e não de "esperado" e "obtido".

## Rodando os testes

Os testes do projeto [`codigo/boletim-escolar`](codigo/boletim-escolar/) estão em `src/lib.rs` e em `src/notas.rs`. Cada módulo tem o próprio `mod tests`:

```rust
// src/notas.rs (trecho)
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn media_de_lista_vazia() {
        assert_eq!(media(&[]), None);
    }

    #[test]
    fn conceitos_nas_fronteiras() {
        assert_eq!(conceito(10.0), 'A');
        assert_eq!(conceito(9.0), 'A');
        assert_eq!(conceito(8.95), 'B');
        assert_eq!(conceito(5.0), 'C');
        assert_eq!(conceito(4.9), 'D', "4.9 ainda é reprovação");
    }

    #[test]
    fn soma_privada_e_testavel() {
        assert_eq!(soma(&[1.5, 2.5]), 4.0);
    }
}
```

```bash
cargo test
```

```text
   Compiling boletim-escolar v0.1.0 (/caminho/boletim-escolar)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.18s
     Running unittests src/lib.rs (target/debug/deps/boletim_escolar-37f5c5146adf60dc)

running 6 tests
test notas::tests::conceitos_nas_fronteiras ... ok
test notas::tests::media_de_lista_vazia ... ok
test notas::tests::media_simples ... ok
test tests::nomes_saem_em_ordem ... ok
test notas::tests::soma_privada_e_testavel ... ok
test tests::agrupa_notas_do_mesmo_aluno ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running unittests src/main.rs (target/debug/deps/boletim_escolar-2e1292fb32e70aae)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

   Doc-tests boletim_escolar

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

Três blocos, um para cada coisa que o Cargo testa: a crate da biblioteca, a crate binária e os exemplos da documentação (*doctests*, Módulo 08). Os nomes mostram o caminho do teste (`notas::tests::...`), e a ordem varia porque os testes rodam **em paralelo**.

Para rodar só alguns, passe um pedaço do nome: `cargo test media` roda os dois testes com "media" no nome e mostra `4 filtered out`.

## Lendo uma falha

Suponha um erro de digitação na fronteira do conceito C (`49..=69` em vez de `50..=69`):

```text
running 6 tests
test notas::tests::media_de_lista_vazia ... ok
test notas::tests::media_simples ... ok
test notas::tests::soma_privada_e_testavel ... ok
test tests::nomes_saem_em_ordem ... ok
test tests::agrupa_notas_do_mesmo_aluno ... ok
test notas::tests::conceitos_nas_fronteiras ... FAILED

failures:

---- notas::tests::conceitos_nas_fronteiras stdout ----

thread 'notas::tests::conceitos_nas_fronteiras' (208132) panicked at src/notas.rs:49:9:
assertion `left == right` failed: 4.9 ainda é reprovação
  left: 'C'
 right: 'D'
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace


failures:
    notas::tests::conceitos_nas_fronteiras

test result: FAILED. 5 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

error: test failed, to rerun pass `--lib`
```

O que olhar:

1. **Qual teste**: `conceitos_nas_fronteiras`.
2. **Onde**: `src/notas.rs:49:9`, a linha exata da asserção que falhou (o teste tem cinco).
3. **Por quê**: a mensagem que você escreveu e os dois valores, `left: 'C'` (o que `conceito(4.9)` devolveu) e `right: 'D'` (o esperado).

O número entre parênteses depois do nome da thread é um identificador que muda a cada execução. Note também que, quando os testes da biblioteca falham, o Cargo para ali e nem roda os blocos seguintes.

## Comparação

- **Python (`pytest`)**: lá, um teste é descoberto pelo nome (`test_*`) e costuma ficar num arquivo separado. Aqui, é o atributo `#[test]` que marca, e os testes unitários ficam no mesmo arquivo, com acesso ao que é privado.
- **C++**: não há framework na linguagem (GoogleTest e Catch2 são bibliotecas externas, com configuração no CMake). Em Rust, o framework básico vem embutido no compilador e no Cargo.

## Armadilhas comuns

- **Esquecer o `#[test]`**: a função compila e nunca roda. Confira se o número em `running N tests` bate com o esperado.
- **Esquecer o `use super::*`**: os testes não enxergam as funções do módulo (E0425 `cannot find function`).
- **Testar a ordem de um `HashMap`**: a ordem muda a cada execução. Compare valores com `get` ou ordene antes.
- **Comparar floats com `assert_eq!` depois de contas**: `0.1 + 0.2 != 0.3`, como em qualquer linguagem. Para resultados de contas, compare com tolerância: `assert!((x - 0.3).abs() < 1e-9)`.

## Resumo

- Os testes unitários ficam num `#[cfg(test)] mod tests` no mesmo arquivo, com `use super::*`.
- Cada teste é uma função com `#[test]`. Ele falha quando dá *panic*.
- `assert!`, `assert_eq!` e `assert_ne!` aceitam uma mensagem no formato do `format!`.
- `cargo test` roda tudo, `cargo test nome` filtra. Os testes rodam em paralelo.
- Na falha, leia o nome do teste, a linha e os valores `left`/`right`.

## Exercícios

Os exercícios 2 a 6 de [`exercicios.md`](exercicios.md) vêm com testes prontos.

## Leitura complementar

- [The Rust Programming Language, 11.1 How to Write Tests](https://doc.rust-lang.org/stable/book/ch11-01-writing-tests.html)
- [The Rust Programming Language, 11.2 Controlling How Tests Are Run](https://doc.rust-lang.org/stable/book/ch11-02-running-tests.html)
- [The Cargo Book: `cargo test`](https://doc.rust-lang.org/cargo/commands/cargo-test.html)

---

[← Lição anterior](09-kit-modulos-lib-e-main.md) · [Índice do módulo](README.md)

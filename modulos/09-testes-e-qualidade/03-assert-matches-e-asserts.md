# 03 — `assert_matches!` e as outras asserções

**Objetivos.** Ao final, você vai saber:

- usar `assert_matches!` (estável desde a 1.96), com a importação certa;
- quando ele é melhor que `assert_eq!` e que `assert!(matches!(...))`;
- o papel de `debug_assert!` e como comparar floats.

## O problema: testar a variante de um erro

`ErroIdade` não deriva `PartialEq` (e não poderia derivar com facilidade se carregasse um `io::Error`, que não implementa `PartialEq`). Então o `assert_eq!` nem compila:

```rust
// ❌ NÃO COMPILA
#[test]
fn texto_vazio() {
    assert_eq!(idade(""), Err(ErroIdade::Vazia));
}
```

```text
error[E0369]: binary operation `==` cannot be applied to type `Result<u32, ErroIdade>`
  --> src/lib.rs:22:9
   |
22 |         assert_eq!(idade(""), Err(ErroIdade::Vazia));
   |         ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
   |         |
   |         Result<u32, ErroIdade>
   |         Result<u32, ErroIdade>
   |
note: an implementation of `PartialEq` might be missing for `ErroIdade`
...
help: consider annotating `ErroIdade` with `#[derive(PartialEq)]`
```

Mesmo quando o `PartialEq` existe, muitas vezes você não quer comparar **tudo**. Para o erro "não é número", o que interessa é a variante `NaoNumero`, e não os detalhes do `ParseIntError` dentro dela. O que você quer é um *pattern matching* (o mesmo do `match`, do [Módulo 04](../04-structs-enums-e-patterns/05-match-e-padroes.md)).

## `assert_matches!`

```rust
use std::assert_matches;

#[test]
fn texto_vazio() {
    assert_matches!(idade("   "), Err(ErroIdade::Vazia));
}

#[test]
fn texto_que_nao_e_numero() {
    assert_matches!(idade("abc"), Err(ErroIdade::NaoNumero(_)));
}

#[test]
fn idade_acima_do_limite() {
    assert_matches!(idade("200"), Err(ErroIdade::ForaDaFaixa(n)) if n == 200);
}

#[test]
fn negativo_nao_e_numero() {
    assert_matches!(
        idade("-1"),
        Err(ErroIdade::NaoNumero(_)),
        "u32 não aceita sinal de menos"
    );
}
```

O segundo argumento é um padrão, com `_`, variáveis e até uma guarda `if`. Como nas outras asserções, uma mensagem opcional pode vir no fim. O valor precisa implementar `Debug`, para aparecer na falha.

### A importação

A macro **não** está no *prelude* (o conjunto de nomes importados automaticamente). Sem importar:

```text
error: cannot find macro `assert_matches` in this scope
 --> src/lib.rs:5:9
  |
5 |         assert_matches!(Some(3), Some(_));
  |         ^^^^^^^^^^^^^^
  |
help: consider importing this macro
  |
3 +     use std::assert_matches;
  |
```

O certo é `use std::assert_matches;` (ou o caminho completo `std::assert_matches!(...)`). Muitos blogs, escritos quando a macro ainda era instável, ensinam outro caminho, que hoje não existe:

```rust
// ❌ NÃO COMPILA
use std::assert_matches::assert_matches;
```

```text
error[E0432]: unresolved import `std::assert_matches`
 --> src/lib.rs:3:14
  |
3 |     use std::assert_matches::assert_matches;
  |              ^^^^^^^^^^^^^^ could not find `assert_matches` in `std`
```

## Por que não `assert!(matches!(...))`?

A macro `matches!` existe há anos e devolve `bool`. Dá para escrever `assert!(matches!(idade("x"), Err(ErroIdade::Vazia)))`. Compare as duas falhas:

```text
---- falhas::com_assert_matches stdout ----

thread 'falhas::com_assert_matches' (97241) panicked at src/lib.rs:219:9:
assertion `left matches right` failed
  left: Err(NaoNumero(ParseIntError { kind: InvalidDigit }))
 right: Err(ErroIdade::Vazia)

---- falhas::com_matches_antigo stdout ----

thread 'falhas::com_matches_antigo' (97242) panicked at src/lib.rs:224:9:
assertion failed: matches!(idade("x"), Err(ErroIdade::Vazia))
```

O `assert_matches!` mostra **o valor que veio**. O `assert!` só repete o código. Na hora de investigar, isso é a diferença entre saber e adivinhar.

É a mesma razão pela qual existe `assert_eq!`. O `assert` do pytest reescreve a expressão para mostrar os valores dos dois lados; o `assert!` do Rust não faz isso, então são as macros especializadas que mostram os valores.

## Quando usar cada uma

| Situação | Macro |
|---|---|
| condição booleana simples | `assert!(lista.is_empty())` |
| o valor inteiro importa e o tipo tem `PartialEq` | `assert_eq!` / `assert_ne!` |
| só a "forma" importa (variante, parte dos campos), ou o tipo não tem `PartialEq` | `assert_matches!` |
| um float depois de contas | `assert!((x - esperado).abs() < 1e-9)` |

Para floats, a tolerância é a mesma do Módulo 01. A crate `approx` oferece `assert_relative_eq!` para quem faz muita conta.

Em Rust, toda asserção é **fatal** para o teste, como o `ASSERT_*` do GoogleTest. Não existe o `EXPECT_*`, que registra a falha e continua. Se você quer ver várias falhas de uma vez, divida em testes menores (ou use um teste por caso com `rstest`, lição 10).

## `debug_assert!` e companhia

`debug_assert!`, `debug_assert_eq!` e `debug_assert_matches!` funcionam como as versões normais, mas **somem** no perfil `release`. Elas servem para invariantes caras de verificar no **código de produção**, que você quer conferidas enquanto desenvolve e testa:

```rust
fn media_ordenada(v: &[u32]) -> u32 {
    debug_assert!(v.is_sorted(), "a entrada deveria vir ordenada");
    // ...
    v[v.len() / 2]
}
```

Dentro de testes, use as versões normais: `cargo test --release` existe, e um teste que deixa de verificar no `release` não testa nada.

## Comparação

| | GoogleTest | pytest | Rust |
|---|---|---|---|
| igualdade | `ASSERT_EQ(a, b)` | `assert a == b` | `assert_eq!(a, b)` |
| padrão | matchers do gMock (`EXPECT_THAT`) | `match` + `assert` | `assert_matches!` |
| erro esperado | `EXPECT_THROW` | `with pytest.raises(E):` | `assert_matches!(r, Err(E::...))` |
| não fatal | `EXPECT_*` | (plugins) | não há |

## Armadilhas comuns

- **`use std::assert_matches::assert_matches;`** dá E0432. Use `use std::assert_matches;`.
- **`assert!(matches!(...))`** funciona, mas a falha não mostra o valor.
- **`debug_assert!` em teste.** Some no `release`. Em teste, use `assert!`.
- **`assert_eq!` com floats depois de contas.** Use tolerância.

## Resumo

- `assert_matches!(valor, padrão)` (1.96) verifica um padrão, com guarda `if` e mensagem opcional.
- Importe com `use std::assert_matches;`. O caminho `std::assert_matches::assert_matches` não existe.
- A falha mostra o valor recebido, ao contrário de `assert!(matches!(...))`.
- Use `assert_eq!` quando o valor inteiro importa, e `assert_matches!` quando só a forma importa.
- `debug_assert*!` são para invariantes no código de produção e somem no `release`.

## Exercícios

Veja o exercício 1 em [`exercicios.md`](exercicios.md).

## Leitura complementar

- [Documentação de `assert_matches!`](https://doc.rust-lang.org/std/macro.assert_matches.html)
- [Documentação de `matches!`](https://doc.rust-lang.org/std/macro.matches.html) e [de `debug_assert!`](https://doc.rust-lang.org/std/macro.debug_assert.html)
- [RustTraining (C/C++): Testing Patterns](https://microsoft.github.io/RustTraining/c-cpp-book/ch08-1-testing-patterns.html)

---

[← Lição anterior](02-testes-unitarios-a-fundo.md) · [Índice do módulo](README.md) · [Próxima lição →](04-testes-de-integracao.md)

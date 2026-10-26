# 02 — Testes unitários a fundo

**Objetivos.** Ao final, você vai saber:

- o que `#[cfg(test)]` faz de verdade, e o erro de usar algo só de teste no código normal;
- testar *panics* com `#[should_panic(expected = ...)]`;
- escrever testes que devolvem `Result`, e a armadilha da falha sem linha;
- montar *fixtures* com funções e `Drop`, e manter os testes isolados.

Os exemplos estão em [`codigo/idade`](codigo/idade/): uma função `idade(texto) -> Result<u32, ErroIdade>` que aceita de 0 a 150, e uma `porcentagem(parte, total)` que panica com `total == 0`.

## `#[cfg(test)]` remove o código do build normal

O `mod tests` do Módulo 01 vem marcado com `#[cfg(test)]`. Isso não é só uma convenção: o item **deixa de existir** fora do `cargo test`. Qualquer item pode receber o atributo, inclusive um helper fora do `mod tests`. Mas aí o código de produção não pode usá-lo:

```rust
// ❌ NÃO COMPILA
#[cfg(test)]
fn idades_de_exemplo() -> Vec<&'static str> {
    vec!["10", "20"]
}

pub fn quantas() -> usize {
    idades_de_exemplo().len()
}
```

```text
error[E0425]: cannot find function `idades_de_exemplo` in this scope
 --> src/lib.rs:7:5
  |
7 |     idades_de_exemplo().len()
  |     ^^^^^^^^^^^^^^^^^ not found in this scope
  |
note: found an item that was configured out
 --> src/lib.rs:2:4
  |
1 | #[cfg(test)]
  |       ---- the item is gated here
2 | fn idades_de_exemplo() -> Vec<&'static str> {
  |    ^^^^^^^^^^^^^^^^^
```

O `note` diz exatamente o que houve: o item existe, mas foi "configurado para fora". Coloque helpers de teste **dentro** do `mod tests` e o problema não aparece.

Como o `mod tests` é filho do módulo testado, ele enxerga os itens privados (`soma` no exemplo é privada e tem teste). Em C++, testar um membro privado exige `friend` ou o truque `#define private public`. Em Rust, é o comportamento normal dos módulos.

## `#[should_panic]`: quando o *panic* é o comportamento certo

`porcentagem` documenta que panica com `total == 0`. Para testar isso:

```rust
#[test]
#[should_panic(expected = "não pode ser zero")]
fn porcentagem_de_zero_panica() {
    porcentagem(1, 0);
}
```

```text
test tests::porcentagem_de_zero_panica - should panic ... ok
```

O `expected` é um pedaço que precisa estar **contido** na mensagem do *panic*. Se a mensagem for outra, a falha explica bem:

```text
---- falhas::mensagem_errada stdout ----

thread 'falhas::mensagem_errada' (97244) panicked at src/lib.rs:51:5:
total não pode ser zero
note: panic did not contain expected string
      panic message: "total não pode ser zero"
 expected substring: "negativo"
```

Use **sempre** com `expected`. Um `#[should_panic]` sozinho passa com qualquer *panic*, inclusive um `unwrap()` esquecido no meio do teste, que não tem nada a ver com o que você queria verificar.

> [!WARNING]
> `#[should_panic]` é para *panics*. Para "esta função devolve erro", **não** use `should_panic`: teste o `Result` (com `assert!(r.is_err())` ou, melhor, com `assert_matches!`, na próxima lição). É a diferença entre bug e falha esperada do [Módulo 05](../05-tratamento-de-erros/01-panic-e-erros-irrecuperaveis.md). O `pytest.raises` do Python corresponde ao teste do `Result`, e não ao `should_panic`.

## Testes que devolvem `Result`

Um teste também pode devolver `Result<(), E>`, desde que `E: Debug`. A vantagem é poder usar `?` no corpo:

```rust
#[test]
fn media_usa_so_as_validas() -> Result<(), ErroIdade> {
    let base = idade("30")?;
    let media = media_idades(&["30", "abc", "50", ""]).expect("há duas idades válidas");
    assert!(
        (media - f64::from(base + 10)).abs() < 1e-9,
        "média foi {media}"
    );
    Ok(())
}
```

O teste falha se devolver `Err`. Mas veja **como** ele falha. Este teste tem um erro de digitação (a letra O no lugar do zero):

```rust
#[test]
fn idade_com_interrogacao() -> Result<(), ErroIdade> {
    let n = idade("4O")?; // letra O no lugar do zero
    assert_eq!(n, 40);
    Ok(())
}
```

```text
---- falhas::idade_com_interrogacao stdout ----
Error: NaoNumero(ParseIntError { kind: InvalidDigit })
```

Só isso: **sem arquivo, sem linha**. Com cinco `?` no teste, você não sabe qual falhou. Por isso, a regra prática:

- use `?` no **preparo** (ler um arquivo, montar um valor), onde uma falha é "o teste nem começou";
- use `assert!`/`assert_eq!`/`assert_matches!` no que está sendo **verificado**, porque eles mostram a linha e os valores.

### O que não combina

Um teste com `#[should_panic]` precisa devolver `()`, e uma função de teste não recebe argumentos:

```rust
// ❌ NÃO COMPILA
#[test]
#[should_panic]
fn com_result() -> Result<(), String> {
    Err("x".into())
}

#[test]
fn com_argumento(idade: u32) {
    assert_eq!(idade, 18);
}
```

```text
error: functions using `#[should_panic]` must return `()`
 --> src/lib.rs:5:5
  |
5 | /     fn com_result() -> Result<(), String> {
6 | |         Err("x".into())
7 | |     }
  | |_____^

error: functions used as tests can not have any arguments
  --> src/lib.rs:10:5
   |
10 | /     fn com_argumento(idade: u32) {
11 | |         assert_eq!(idade, 18);
12 | |     }
   | |_____^
```

Para rodar o mesmo teste com vários valores (o `@pytest.mark.parametrize`), a std oferece o laço sobre uma tabela de casos, como o teste `conceitos_nas_fronteiras` do Módulo 01. A crate `rstest` (lição 10) gera um teste por caso.

## *Fixtures*: funções e `Drop`

A std não tem um sistema de *fixtures* (valores preparados para vários testes) como o `@pytest.fixture` ou o `SetUp`/`TearDown` do GoogleTest. O idioma em Rust é mais simples:

- **o preparo é uma função** que devolve o valor pronto;
- **a limpeza é o `Drop`** do valor (o RAII do [Módulo 02](../02-ownership/04-drop-e-raii.md)), que roda no fim do teste, mesmo se ele falhar.

```rust
fn idades_de_exemplo() -> Vec<&'static str> {
    vec!["10", "20", "30", "999"]
}

/// Um arquivo temporário que se apaga sozinho no fim do teste.
struct ArquivoTemporario {
    caminho: PathBuf,
}

impl ArquivoTemporario {
    fn novo(nome: &str, conteudo: &str) -> ArquivoTemporario {
        // Nome único por processo e por teste: os testes rodam em paralelo.
        let caminho =
            std::env::temp_dir().join(format!("m09-idade-{}-{nome}", std::process::id()));
        fs::write(&caminho, conteudo).unwrap();
        ArquivoTemporario { caminho }
    }
}

impl Drop for ArquivoTemporario {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.caminho);
    }
}

#[test]
fn idades_lidas_de_arquivo() {
    let arquivo = ArquivoTemporario::novo("idades.txt", "18\n\n200\n40\n");
    let texto = fs::read_to_string(&arquivo.caminho).unwrap();
    let linhas: Vec<&str> = texto.lines().collect();
    assert_eq!(media_idades(&linhas), Some(29.0));
} // `arquivo` sai de escopo aqui, e o Drop apaga o arquivo
```

Na prática, para arquivos e pastas temporários, use a crate `tempfile` (lição 04), que faz exatamente isso com nomes aleatórios.

## Isolamento: testes paralelos no mesmo processo

Os testes de um bloco rodam em **threads do mesmo processo**. Tudo o que é global é compartilhado: o diretório atual, as variáveis de ambiente, um arquivo de nome fixo. Dois testes que escrevem em `/tmp/saida.txt` vão falhar "às vezes".

As variáveis de ambiente são o caso mais traiçoeiro, e a edition 2024 deixa isso explícito: `set_var` virou `unsafe`, justamente porque mudar o ambiente com outras threads lendo é comportamento indefinido em várias plataformas.

```rust
// ❌ NÃO COMPILA
#[test]
fn com_set_var() {
    std::env::set_var("IGNORE_CASE", "1");
}
```

```text
error[E0133]: call to unsafe function `set_var` is unsafe and requires unsafe block
  --> src/lib.rs:16:9
   |
16 |         std::env::set_var("IGNORE_CASE", "1");
   |         ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ call to unsafe function
   |
   = note: consult the function's documentation for information on how to avoid undefined behavior
```

A saída não é um bloco `unsafe`. É **injetar o valor**: a função recebe o que precisa por parâmetro, e só o `main` lê o ambiente. O Projeto 1 já fez isso com o parâmetro `ignore_case_env: bool` de `Config::from_args`. Num teste de integração, que roda o executável num processo separado, você passa a variável só para aquele processo com `.env("IGNORE_CASE", "1")` no `Command` (lição 04).

Isso vale também para o relógio, números aleatórios e a rede. Em Python, o `monkeypatch` e o `unittest.mock.patch` trocam qualquer função em tempo de execução. Em Rust isso não existe, e o desenho testável (passar a dependência como parâmetro ou por uma *trait*) é a resposta. A lição 10 volta ao assunto.

## Armadilhas comuns

- **`#[should_panic]` sem `expected`.** Passa com qualquer *panic*. Sempre diga a mensagem esperada.
- **`?` na verificação.** A falha não mostra a linha. Use `?` só no preparo.
- **`should_panic` para testar erro.** Um `Err` não é *panic*. Teste o `Result`.
- **Nome fixo de arquivo temporário.** Testes paralelos (e duas execuções simultâneas) disputam o arquivo. Use um nome único ou `tempfile`.
- **Helper com `#[cfg(test)]` usado no código normal.** E0425 com `found an item that was configured out`.

## Resumo

- `#[cfg(test)]` faz o item sumir fora do `cargo test`; o `mod tests` enxerga os privados do módulo pai.
- `#[should_panic(expected = "trecho")]` testa *panics*. Para erros esperados, teste o `Result`.
- Testes podem devolver `Result<(), E: Debug>`, mas a falha por `?` não mostra a linha.
- *Fixture* é uma função; *teardown* é o `Drop`.
- Testes rodam em paralelo no mesmo processo: nada de estado global. `set_var` é `unsafe` na 2024 por esse motivo.

## Exercícios

Veja o exercício 1 em [`exercicios.md`](exercicios.md).

## Leitura complementar

- [The Rust Programming Language, 11.1 How to Write Tests](https://doc.rust-lang.org/stable/book/ch11-01-writing-tests.html) (inclui `should_panic` e `Result`)
- [The Rust Reference: testing attributes](https://doc.rust-lang.org/reference/attributes/testing.html)
- [Documentação de `std::env::set_var`](https://doc.rust-lang.org/std/env/fn.set_var.html) (a seção *Safety*)

---

[← Lição anterior](01-anatomia-do-cargo-test.md) · [Índice do módulo](README.md) · [Próxima lição →](03-assert-matches-e-asserts.md)

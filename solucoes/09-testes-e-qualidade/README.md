# Soluções — Módulo 09

Enunciados em [`modulos/09-testes-e-qualidade/exercicios.md`](../../modulos/09-testes-e-qualidade/exercicios.md).

Cada solução é um projeto Cargo. Para conferir:

```bash
cd nome-do-projeto
cargo test
CARGO_BUILD_WARNINGS=deny cargo clippy --all-targets
```

## 1. Do `should_panic` ao `Result`

Projeto: [`termometro/`](termometro/src/lib.rs).

```rust
pub fn ler(texto: &str) -> Result<Termometro, ErroTemperatura> {
    let texto = texto.trim();
    let celsius: f64 = texto
        .parse()
        .map_err(|_| ErroTemperatura::NaoNumero(texto.to_string()))?;
    if celsius < ZERO_ABSOLUTO {
        return Err(ErroTemperatura::AbaixoDoZeroAbsoluto(celsius));
    }
    Ok(Termometro::new(celsius))
}
```

`ler` confere o limite **antes** de chamar `Termometro::new`, então o `assert!` de dentro nunca dispara por ela. O `new` continua panicando, porque um valor inválido ali é bug de quem chamou.

Os testes:

```rust
use std::assert_matches;

#[test]
#[should_panic(expected = "abaixo do zero absoluto")]
fn abaixo_do_zero_absoluto() {
    Termometro::new(-300.0);
}

#[test]
fn texto_invalido() {
    assert_matches!(ler("abc"), Err(ErroTemperatura::NaoNumero(t)) if t == "abc");
}

#[test]
fn muito_frio() {
    assert_matches!(
        ler("-300"),
        Err(ErroTemperatura::AbaixoDoZeroAbsoluto(c)) if c == -300.0
    );
}

#[test]
fn agua_fervendo() -> Result<(), ErroTemperatura> {
    let t = ler("100")?;
    assert!((t.fahrenheit() - 212.0).abs() < 1e-9);
    Ok(())
}
```

- `texto_invalido` era um `should_panic` que devolvia `Result`, o que não compila. E mesmo que compilasse, a ideia estava errada: `ler("abc")` não panica, devolve um erro. O teste certo verifica o `Err`.
- `muito_frio` usava `assert_eq!`, que exige `PartialEq` (E0369). Com `assert_matches!`, a guarda `if c == -300.0` confere o valor. Comparar floats com `==` aqui é seguro: o valor não vem de conta, vem do `parse` do mesmo texto.
- O `should_panic` sem `expected` passaria com qualquer *panic*. Com o trecho da mensagem, ele só passa com o *panic* certo.

## 2. Integração do `conta-linhas`

Projeto: [`conta-linhas/`](conta-linhas/).

O `run` ganhou o caso do `-`:

```rust
let text = if path == "-" {
    let mut text = String::new();
    io::stdin()
        .read_to_string(&mut text)
        .map_err(|source| Failure::Read {
            path: "a entrada padrão".to_string(),
            source,
        })?;
    text
} else {
    fs::read_to_string(&path).map_err(|source| Failure::Read { path, source })?
};
```

Os testes ficaram em [`tests/cli.rs`](conta-linhas/tests/cli.rs) (os três do Módulo 05, com `tempfile`) e [`tests/stdin.rs`](conta-linhas/tests/stdin.rs) (os dois novos), e os helpers em [`tests/common/mod.rs`](conta-linhas/tests/common/mod.rs), com:

```rust
#![allow(dead_code, reason = "nem todo arquivo de teste usa todos os helpers")]
```

Por que `allow`: o `common` é compilado duas vezes, uma como módulo de `cli.rs` (que não usa `rodar_com_stdin`, então o lint dispara) e outra como módulo de `stdin.rs` (que usa, então não dispara). Um `expect` ficaria não cumprido na segunda crate.

Por que não `tests/common.rs`: todo arquivo direto em `tests/` vira uma crate de teste. O Cargo compilaria um executável `common` com zero testes. Numa subpasta, ele é só um módulo.

O `tempfile::tempdir()` apaga a pasta no `Drop`, mesmo se o teste falhar no meio. O nome com `process::id()` do Módulo 05 evitava a colisão entre testes, mas deixava o arquivo para trás numa falha.

## 3. Lints com prazo de validade

Projeto: [`lints-com-prazo/`](lints-com-prazo/src/lib.rs).

Trocando os `allow` por `expect`, o clippy aponta dois não cumpridos:

```text
warning: this lint expectation is unfulfilled
8 | #[expect(unused_variables, reason = "x")]
warning: this lint expectation is unfulfilled
29 | #[expect(clippy::too_many_arguments, reason = "x")]
```

Eles estavam silenciando nada: `total` é usado, e três argumentos estão longe do limite. Foram apagados (e o `let total` virou só `valores.iter().sum()`).

Dos dois que se cumprem:

- `needless_range_loop` em `dobra`: o certo é **corrigir** o código, como o clippy sugere: `for v in valores.iter_mut() { *v *= 2; }`.
- `dead_code` em `formato_antigo`: virou `#[expect(dead_code, reason = "...")]` com um motivo de verdade. Se ninguém tem um motivo, a resposta certa é apagar a função.

Com o `pedantic` ligado (`pedantic = { level = "warn", priority = -1 }` e `must_use_candidate = "allow"`), aparecem dois avisos novos:

```text
warning: docs for function returning `Result` missing `# Errors` section
 --> src/lib.rs:4:1
warning: casting `usize` to `u32` may truncate the value on targets with 64-bit wide pointers
  --> src/lib.rs:19:21
```

O primeiro se resolve com a seção `# Errors` na documentação. O segundo, sem `as`:

```rust
pub fn media(valores: &[u32]) -> u32 {
    match u32::try_from(valores.len()) {
        Ok(n) if n > 0 => soma(valores) / n,
        _ => 0,
    }
}
```

A guarda `n > 0` trata a lista vazia, e o `_` cobre também o caso (teórico) de mais de 4 bilhões de valores.

## 4. CI do minigrep

Arquivo: [`ci-minigrep/ci.yml`](ci-minigrep/ci.yml). No seu repositório, ele vai em `.github/workflows/ci.yml`.

É o workflow da [lição 07](../../modulos/09-testes-e-qualidade/07-ci-com-github-actions.md), sem mudar nada: o minigrep é um pacote só, com `Cargo.lock`, e passa em todos os passos. Os comandos foram conferidos localmente (`CARGO_BUILD_WARNINGS=deny` com fmt, clippy, test, doc, e `cargo +1.96 check --all-targets --locked`).

Com o `let x = 5;` no `main`, o passo `cargo clippy --all-targets --locked` falha assim:

```text
warning: unused variable: `x`
  --> src/main.rs:16:9
   |
16 |     let x = 5;
   |         ^ help: if this is intentional, prefix it with an underscore: `_x`
   |
   = note: `#[warn(unused_variables)]` (part of `#[warn(unused)]`) on by default

error: `minigrep` (bin "minigrep") generated 1 warning (run `cargo clippy --fix --bin "minigrep" -p minigrep -- ` to apply 1 suggestion)
error: warnings are denied by `build.warnings` configuration
error: `minigrep` (bin "minigrep" test) generated 1 warning (1 duplicate)
error: warnings are denied by `build.warnings` configuration
```

O aviso aparece duas vezes porque `--all-targets` compila o binário normal e o binário de testes. Repare que o `cargo test` nem chega a rodar: o job para no primeiro passo que falha. O job `msrv` continua verde, porque na 1.96 o `CARGO_BUILD_WARNINGS` é ignorado (e é por isso que ele só compila).

## 5. Propriedade de ida e volta

Projeto: [`data-proptest/`](data-proptest/src/lib.rs).

```rust
fn uma_data() -> impl Strategy<Value = Data> {
    (1000u16..=9999, 1u8..=12, 1u8..=31).prop_map(|(ano, mes, dia)| Data { ano, mes, dia })
}

proptest! {
    #[test]
    fn ida_e_volta(data in uma_data()) {
        prop_assert_eq!(data.to_string().parse::<Data>(), Ok(data));
    }
}
```

Com o código do enunciado, a falha vem na primeira tentativa, e o *shrinking* leva cada campo ao menor valor da faixa:

```text
Test failed: assertion failed: `(left == right)`
  left: `Err(Formato)`,
 right: `Ok(Data { ano: 1000, mes: 1, dia: 1 })` at src/lib.rs:92.
minimal failing input: data = Data {
    ano: 1000,
    mes: 1,
    dia: 1,
}
	successes: 0
```

Lendo o caso: `1000-1-01` tem 9 caracteres, e o `from_str` exige 10. O bug está no `Display`, que não completa o mês com zero: `{}` em vez de `{:02}`. A correção é `write!(f, "{:04}-{:02}-{:02}", ...)`. O teste com nome:

```rust
#[test]
fn mes_de_um_digito() {
    let data = Data::nova(1000, 1, 1).unwrap();
    assert_eq!(data.to_string(), "1000-01-01");
    assert_eq!(data.to_string().parse(), Ok(data));
}
```

Lembre de commitar o `proptest-regressions/lib.txt` que a falha gerou.

## 6. Medir antes de otimizar

Projeto: [`contar-palavras/`](contar-palavras/), com o benchmark em [`benches/palavras.rs`](contar-palavras/benches/palavras.rs).

Uma execução (`cargo bench -- --warm-up-time 1 --measurement-time 3`), com uns 50 KB de texto:

```text
split_whitespace        time:   [56.622 µs 57.481 µs 58.716 µs]
Found 10 outliers among 100 measurements (10.00%)
...
laço manual            time:   [50.755 µs 54.419 µs 58.768 µs]
Found 20 outliers among 100 measurements (20.00%)
```

O laço manual parece uns 5% mais rápido, mas os intervalos se sobrepõem (o do laço vai até 58,8 µs, o do `split_whitespace` começa em 56,6 µs), e 20% de *outliers* mostram uma medição barulhenta. A conclusão honesta: **na prática, empatam**. As duas percorrem os caracteres uma vez e decodificam UTF-8 do mesmo jeito; o `split_whitespace` não aloca nada (devolve fatias), então não há custo escondido.

É exatamente o tipo de resultado que justifica medir: o laço manual dá mais trabalho, tem mais chance de bug (por isso o teste que compara as duas) e não ganha nada que se possa provar. Fique com o `split_whitespace`.

---

[Índice do módulo](../../modulos/09-testes-e-qualidade/README.md)

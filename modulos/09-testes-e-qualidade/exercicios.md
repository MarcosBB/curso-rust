# Exercícios — Módulo 09

Soluções em [`solucoes/09-testes-e-qualidade`](../../solucoes/09-testes-e-qualidade/).

Em todos os exercícios, crie o projeto com `cargo new` (ou `cargo new --lib`), acrescente `rust-version = "1.96"` no `Cargo.toml` e cole o código inicial. O objetivo final é sempre o mesmo: `cargo test` passando e `CARGO_BUILD_WARNINGS=deny cargo clippy --all-targets` sem avisos.

## 1. Do `should_panic` ao `Result` (fácil, faça compilar)

Lições: [02](02-testes-unitarios-a-fundo.md) e [03](03-assert-matches-e-asserts.md).

Crie `cargo new --lib termometro` e use este `src/lib.rs`. A função `ler` ainda não existe, e os testes têm três erros de compilação e dois testes fracos.

```rust
pub const ZERO_ABSOLUTO: f64 = -273.15;

#[derive(Debug)]
pub struct Termometro {
    celsius: f64,
}

#[derive(Debug)]
pub enum ErroTemperatura {
    /// O texto não é um número.
    NaoNumero(String),
    /// O número está abaixo do zero absoluto.
    AbaixoDoZeroAbsoluto(f64),
}

impl Termometro {
    pub fn new(celsius: f64) -> Termometro {
        assert!(
            celsius >= ZERO_ABSOLUTO,
            "abaixo do zero absoluto: {celsius}"
        );
        Termometro { celsius }
    }

    pub fn fahrenheit(&self) -> f64 {
        self.celsius * 9.0 / 5.0 + 32.0
    }
}

/// Lê uma temperatura em Celsius de um texto (com espaços nas pontas ou não).
pub fn ler(texto: &str) -> Result<Termometro, ErroTemperatura> {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::assert_matches::assert_matches;

    #[test]
    #[should_panic]
    fn abaixo_do_zero_absoluto() {
        Termometro::new(-300.0);
    }

    #[test]
    #[should_panic]
    fn texto_invalido() -> Result<(), ErroTemperatura> {
        ler("abc")?;
        Ok(())
    }

    #[test]
    fn muito_frio() {
        assert_eq!(
            ler("-300").unwrap_err(),
            ErroTemperatura::AbaixoDoZeroAbsoluto(-300.0)
        );
    }

    #[test]
    fn agua_fervendo() {
        let t = ler("100").unwrap();
        assert!((t.fahrenheit() - 212.0).abs() < 1e-9);
    }
}
```

Faça, **sem** derivar `PartialEq` em `ErroTemperatura`:

1. Implemente `ler`: `NaoNumero` guarda o texto (sem os espaços); `AbaixoDoZeroAbsoluto` guarda o valor lido. `ler` nunca panica.
2. Conserte a importação de `assert_matches!`.
3. Reescreva `texto_invalido` e `muito_frio` com `assert_matches!`, conferindo também o conteúdo da variante (o texto `"abc"` e o valor `-300.0`).
4. Ponha `expected` no `should_panic` que sobrar.
5. Faça `agua_fervendo` devolver `Result` e use `?` no preparo.

<details>
<summary>Dica</summary>

Os três erros são E0432 (o caminho `std::assert_matches::assert_matches` não existe), "functions using `#[should_panic]` must return `()`" e E0369 (o `==` precisa de `PartialEq`). Para conferir o conteúdo, use uma guarda: `Err(ErroTemperatura::NaoNumero(t)) if t == "abc"`.

</details>

## 2. Integração do `conta-linhas` (fácil/médio)

Lição: [04](04-testes-de-integracao.md).

Volte ao `conta-linhas` do [exercício 2 do Módulo 05](../05-tratamento-de-erros/exercicios.md) (ou copie a [solução](../../solucoes/05-tratamento-de-erros/conta-linhas/)). Faça:

1. Quando o argumento for `-`, o programa lê da **entrada padrão** em vez de um arquivo.
2. Mova a função `conta_linhas()` dos testes para `tests/common/mod.rs`, junto com um helper novo `rodar_com_stdin(args, entrada) -> Output`.
3. Troque o arquivo temporário de `tests/cli.rs` (o `temp_dir()` + `process::id()`) por `tempfile::tempdir()`.
4. Crie `tests/stdin.rs` com dois testes: `"a\nb\n"` pela entrada dá `2`, e a entrada vazia dá `0`.
5. Resolva o aviso de `dead_code` que vai aparecer, e explique num comentário por que a solução é `allow`, e não `expect`.

Responda também: por que `tests/common.rs` não serviria?

<details>
<summary>Dica</summary>

`io::stdin().read_to_string(&mut texto)` lê tudo (precisa de `use std::io::Read`). `tempfile` entra com `cargo add --dev tempfile`. O aviso aparece porque `cli.rs` não usa `rodar_com_stdin`.

</details>

## 3. Lints com prazo de validade (fácil)

Lições: [05](05-lints-allow-expect-deny.md) e [06](06-clippy-e-rustfmt-a-fundo.md).

Crie `cargo new --lib lints-com-prazo` com este `src/lib.rs`:

```rust
use std::num::ParseIntError;

/// Lê valores separados por vírgula.
pub fn ler_valores(texto: &str) -> Result<Vec<u32>, ParseIntError> {
    texto.split(',').map(|v| v.trim().parse()).collect()
}

#[allow(unused_variables)]
pub fn soma(valores: &[u32]) -> u32 {
    let total: u32 = valores.iter().sum();
    total
}

/// A média inteira, ou 0 para a lista vazia.
pub fn media(valores: &[u32]) -> u32 {
    if valores.is_empty() {
        return 0;
    }
    soma(valores) / valores.len() as u32
}

#[allow(clippy::needless_range_loop)]
pub fn dobra(valores: &mut [u32]) {
    for i in 0..valores.len() {
        valores[i] *= 2;
    }
}

#[allow(clippy::too_many_arguments)]
pub fn ponto(x: i32, y: i32, z: i32) -> (i32, i32, i32) {
    (x, y, z)
}

#[allow(dead_code)]
fn formato_antigo(valores: &[u32]) -> String {
    format!("{valores:?}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn le_e_calcula() {
        let v = ler_valores("2, 4,9").unwrap();
        assert_eq!(soma(&v), 15);
        assert_eq!(media(&v), 5);
        assert_eq!(media(&[]), 0);
    }

    #[test]
    fn dobra_no_lugar() {
        let mut v = [1, 2, 3];
        dobra(&mut v);
        assert_eq!(v, [2, 4, 6]);
        assert_eq!(ponto(1, 2, 3), (1, 2, 3));
    }
}
```

1. Troque cada `#[allow(...)]` por `#[expect(..., reason = "...")]` e rode `cargo clippy --all-targets`. Apague os que estiverem não cumpridos.
2. Dos que sobrarem, decida: corrigir o código (como o clippy sugere) ou manter o `expect` com um motivo de verdade.
3. Acrescente no `Cargo.toml` o grupo `pedantic` (com a prioridade certa) e desligue `must_use_candidate`. Resolva os avisos novos **sem** `allow` e sem `as`.

Ao final, o arquivo não deve ter nenhum `allow`, e os testes não mudam.

<details>
<summary>Dica</summary>

Dois `expect` ficam não cumpridos (o `let total` é usado; três argumentos estão longe do limite do `too_many_arguments`). O `pedantic` cobra uma seção `# Errors` e reclama do `valores.len() as u32`: `u32::try_from` resolve, e um `match` com guarda trata a lista vazia junto.

</details>

## 4. CI do minigrep (médio)

Lição: [07](07-ci-com-github-actions.md).

Num repositório **seu** no GitHub, com o minigrep do [Projeto 1](../../projetos/01-minigrep/README.md) na raiz, crie `.github/workflows/ci.yml` com:

- um job que roda `fmt --check`, `clippy --all-targets`, `test` e `doc --no-deps` na stable, com cache, `--locked` e qualquer aviso falhando;
- um job que compila na 1.96.

Critério: o *badge* (ou a aba *Actions*) fica verde. Depois, abra um *pull request* com um `let x = 5;` não usado no `main` e confirme que a CI fica vermelha **no passo do clippy**.

<details>
<summary>Dica</summary>

O `Cargo.lock` precisa estar commitado para o `--locked`. Antes de subir, rode os mesmos comandos localmente com `CARGO_BUILD_WARNINGS=deny`, e `cargo +1.96 check --all-targets --locked` (instale com `rustup toolchain install 1.96`).

</details>

## 5. Propriedade de ida e volta (médio)

Lição: [08](08-property-testing-com-proptest.md).

Crie `cargo new --lib data-proptest`, `cargo add --dev proptest`, e use este `src/lib.rs`. Ele tem um bug plantado, que os dois testes de exemplo não pegam.

```rust
use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Data {
    pub ano: u16,
    pub mes: u8,
    pub dia: u8,
}

#[derive(Debug, PartialEq, Eq)]
pub enum ErroData {
    /// O texto não está no formato `AAAA-MM-DD`.
    Formato,
    /// O formato está certo, mas o mês ou o dia não existem.
    Invalida,
}

impl Data {
    /// Cria uma data (simplificada: todo mês tem até 31 dias).
    ///
    /// # Errors
    ///
    /// [`ErroData::Invalida`] se o mês não for de 1 a 12 ou o dia de 1 a 31.
    pub fn nova(ano: u16, mes: u8, dia: u8) -> Result<Data, ErroData> {
        if !(1..=12).contains(&mes) || !(1..=31).contains(&dia) {
            return Err(ErroData::Invalida);
        }
        Ok(Data { ano, mes, dia })
    }
}

impl fmt::Display for Data {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:04}-{}-{:02}", self.ano, self.mes, self.dia)
    }
}

impl FromStr for Data {
    type Err = ErroData;

    /// Lê exatamente `AAAA-MM-DD`.
    fn from_str(s: &str) -> Result<Data, ErroData> {
        let b = s.as_bytes();
        if b.len() != 10 || b[4] != b'-' || b[7] != b'-' {
            return Err(ErroData::Formato);
        }
        let numero =
            |faixa: std::ops::Range<usize>| s[faixa].parse().map_err(|_| ErroData::Formato);
        let ano = numero(0..4)?;
        let mes = u8::try_from(numero(5..7)?).map_err(|_| ErroData::Formato)?;
        let dia = u8::try_from(numero(8..10)?).map_err(|_| ErroData::Formato)?;
        Data::nova(ano, mes, dia)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn le_uma_data() {
        assert_eq!("2026-10-24".parse(), Data::nova(2026, 10, 24));
    }

    #[test]
    fn formato_errado() {
        assert_eq!("24/10/2026".parse::<Data>(), Err(ErroData::Formato));
        assert_eq!("2026-13-01".parse::<Data>(), Err(ErroData::Invalida));
    }
}
```

1. Escreva a propriedade: para qualquer data válida (ano de 1000 a 9999), `data.to_string().parse::<Data>()` devolve `Ok(data)`.
2. Rode, leia o caso mínimo, e corrija o bug.
3. Transforme o caso mínimo num teste de exemplo com nome.

<details>
<summary>Dica</summary>

Uma estratégia para `Data`: `(1000u16..=9999, 1u8..=12, 1u8..=31).prop_map(|(ano, mes, dia)| Data { ano, mes, dia })`. Se quiser dar nome a ela, escreva uma função `fn uma_data() -> impl Strategy<Value = Data>`.

</details>

## 6. Medir antes de otimizar (médio)

Lição: [09](09-benchmarks-com-criterion.md).

Crie `cargo new --lib contar-palavras` com duas funções que contam palavras separadas por espaço em branco:

- `contar_split(texto: &str) -> usize`, com `split_whitespace().count()`;
- `contar_manual(texto: &str) -> usize`, com um laço sobre `chars()` que conta as transições de "espaço" para "não espaço".

1. Escreva um teste que confere que as duas concordam em pelo menos cinco textos (vazio, uma palavra, espaços nas pontas, `\n` e `\t`, acentos).
2. Escreva um benchmark com `criterion` que mede as duas com o mesmo texto de uns 50 KB (`"...".repeat(500)`).
3. Rode duas vezes e anote os números. Qual é mais rápida? A diferença é maior que o ruído?

<details>
<summary>Dica</summary>

Lembre do `[[bench]]` com `harness = false` e do `std::hint::black_box`. Compare os **intervalos**, não só o número do meio: se eles se sobrepõem, você não pode dizer que uma versão é mais rápida.

</details>

---

[Índice do módulo](README.md)

# Exercícios — Módulo 05

Soluções em [`solucoes/05-tratamento-de-erros`](../../solucoes/05-tratamento-de-erros/).

Em todos os exercícios, crie um projeto com `cargo new` (ou `cargo new --lib`, quando indicado), acrescente `rust-version = "1.96"` no `Cargo.toml` e cole o código inicial. O objetivo é sempre o mesmo: `cargo test` passando, e `cargo clippy` sem avisos.

## 1. Espalhe `?` (fácil)

Lições: [02](02-result-e-o-operador-interrogacao.md) e [04](04-tipos-de-erro-proprios.md) (`Box<dyn Error>`).

A função abaixo lê um arquivo de configuração com duas linhas (o nome de um serviço e o número máximo de usuários), mas panica com qualquer problema. Crie um projeto com `cargo new --lib espalhe-interrogacao` e coloque isto no `src/lib.rs`:

```rust
use std::fs;

#[derive(Debug, PartialEq)]
pub struct Config {
    pub name: String,
    pub max_users: u32,
}

/// Lê um arquivo com duas linhas: o nome do serviço e o número máximo de usuários.
pub fn read_config(path: &str) -> Config {
    let text = fs::read_to_string(path).unwrap();
    let mut lines = text.lines();
    let name = lines.next().unwrap().trim().to_string();
    let max_users = lines.next().unwrap().trim().parse().unwrap();
    Config { name, max_users }
}
```

Mude a assinatura para `read_config(path: &str) -> Result<Config, Box<dyn Error>>` e tire **todos** os `unwrap`. Quando faltar uma linha, o erro deve ser a mensagem `"faltou a linha do nome"` ou `"faltou a linha de max_users"`. Cole os testes no fim do arquivo e faça-os passar:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::io;
    use std::path::PathBuf;

    /// Cria um arquivo temporário com um nome único para este teste.
    fn temp_file(name: &str, content: &str) -> PathBuf {
        let path = std::env::temp_dir().join(format!("m05-ex1-{}-{name}", std::process::id()));
        fs::write(&path, content).unwrap();
        path
    }

    #[test]
    fn arquivo_valido() {
        let p = temp_file("ok.txt", "loja\n42\n");
        let config = read_config(p.to_str().unwrap()).unwrap();
        assert_eq!(
            config,
            Config {
                name: "loja".to_string(),
                max_users: 42
            }
        );
    }

    #[test]
    fn arquivo_inexistente() {
        let e = read_config("/caminho/que/nao/existe.txt").unwrap_err();
        let io_err = e
            .downcast_ref::<io::Error>()
            .expect("deveria ser io::Error");
        assert_eq!(io_err.kind(), io::ErrorKind::NotFound);
    }

    #[test]
    fn numero_invalido() {
        let p = temp_file("num.txt", "loja\nmuitos\n");
        let e = read_config(p.to_str().unwrap()).unwrap_err();
        assert_eq!(e.to_string(), "invalid digit found in string");
    }

    #[test]
    fn linha_faltando() {
        let p = temp_file("falta.txt", "loja\n");
        let e = read_config(p.to_str().unwrap()).unwrap_err();
        assert_eq!(e.to_string(), "faltou a linha de max_users");

        let p = temp_file("vazio.txt", "");
        let e = read_config(p.to_str().unwrap()).unwrap_err();
        assert_eq!(e.to_string(), "faltou a linha do nome");
    }
}
```

<details>
<summary>Dica</summary>

`lines.next()` devolve `Option`. Para usar `?` numa função que devolve `Result`, transforme o `None` num erro com `ok_or("mensagem")`. Um `&str` vira `Box<dyn Error>` sozinho, pelo `From`. O `parse()` não precisa de anotação de tipo: o compilador descobre pelo campo `max_users`.

</details>

## 2. Códigos de saída (fácil)

Lição: [02](02-result-e-o-operador-interrogacao.md) (`ExitCode`).

Este programa conta as linhas de um arquivo, mas panica (e sai com código 101) se faltar o argumento ou se o arquivo não existir. Crie o projeto com `cargo new conta-linhas` e use este `src/main.rs`:

```rust
use std::fs;

fn main() {
    let path = std::env::args().nth(1).unwrap();
    let text = fs::read_to_string(&path).unwrap();
    println!("{}", text.lines().count());
}
```

Reescreva para que o programa:

| Situação | Saída padrão | Saída de erro | Código |
|---|---|---|---|
| tudo certo | o número de linhas | nada | 0 |
| arquivo não pôde ser lido | nada | `erro: não consegui ler <arquivo>: <motivo>` | 1 |
| faltou o argumento | nada | `uso: conta-linhas <arquivo>` | 2 |

Sem `unwrap`, sem `expect` e sem `process::exit`. Os testes rodam o executável de verdade. Crie a pasta `tests/` e salve isto em `tests/cli.rs`:

```rust
use std::process::Command;

fn conta_linhas() -> Command {
    Command::new(env!("CARGO_BIN_EXE_conta-linhas"))
}

#[test]
fn conta_as_linhas() {
    let path = std::env::temp_dir().join(format!("m05-ex2-{}.txt", std::process::id()));
    std::fs::write(&path, "um\ndois\ntrês\n").unwrap();

    let out = conta_linhas().arg(&path).output().unwrap();
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(String::from_utf8_lossy(&out.stdout), "3\n");
    assert!(out.stderr.is_empty());
}

#[test]
fn arquivo_inexistente_sai_com_1() {
    let out = conta_linhas().arg("nao-existe.txt").output().unwrap();
    assert_eq!(out.status.code(), Some(1));
    assert!(out.stdout.is_empty());
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.starts_with("erro: não consegui ler nao-existe.txt: "),
        "stderr foi: {stderr}"
    );
}

#[test]
fn sem_argumento_sai_com_2() {
    let out = conta_linhas().output().unwrap();
    assert_eq!(out.status.code(), Some(2));
    assert_eq!(
        String::from_utf8_lossy(&out.stderr),
        "uso: conta-linhas <arquivo>\n"
    );
}
```

(`env!("CARGO_BIN_EXE_conta-linhas")` é o caminho do executável que o Cargo compilou para os testes. Testes em `tests/` são o assunto do Módulo 09.)

<details>
<summary>Dica</summary>

Escreva uma `fn run() -> Result<usize, ...>` e deixe o `main` só traduzir o resultado em mensagem e `ExitCode`. Como os dois erros levam a códigos diferentes, o `main` precisa distinguir qual aconteceu: um enum pequeno com duas variantes resolve.

</details>

## 3. Combinadores (médio)

Lição: [03](03-unwrap-expect-e-combinadores.md).

Cada função abaixo usa `match`. Reescreva todas **sem** `match`, `if let` ou laços, usando um ou dois combinadores em cada uma. Os testes garantem que o comportamento não mudou. Projeto: `cargo new --lib combinadores`.

```rust
use std::num::ParseIntError;

/// 1. O número, ou 0 se não for número.
pub fn parse_or_zero(s: &str) -> i32 {
    match s.parse() {
        Ok(n) => n,
        Err(_) => 0,
    }
}

/// 2. O dobro do número.
pub fn parse_double(s: &str) -> Result<i32, ParseIntError> {
    match s.parse::<i32>() {
        Ok(n) => Ok(n * 2),
        Err(e) => Err(e),
    }
}

/// 3. O número, com uma mensagem de erro em português.
pub fn parse_with_message(s: &str) -> Result<i32, String> {
    match s.parse() {
        Ok(n) => Ok(n),
        Err(e) => Err(format!("'{s}' não é um número: {e}")),
    }
}

/// 4. A primeira palavra do texto.
pub fn first_word(s: &str) -> Result<&str, String> {
    match s.split_whitespace().next() {
        Some(w) => Ok(w),
        None => Err("texto vazio".to_string()),
    }
}

/// 5. O número, se for um.
pub fn parse_optional(s: &str) -> Option<i32> {
    match s.parse() {
        Ok(n) => Some(n),
        Err(_) => None,
    }
}

/// 6. Um número estritamente positivo.
pub fn parse_positive(s: &str) -> Result<u32, String> {
    match s.parse::<u32>() {
        Ok(n) => {
            if n > 0 {
                Ok(n)
            } else {
                Err("zero não é positivo".to_string())
            }
        }
        Err(e) => Err(e.to_string()),
    }
}

/// 7. Todos os números separados por vírgula, ou o primeiro erro.
pub fn parse_all(s: &str) -> Result<Vec<i32>, ParseIntError> {
    let mut out = Vec::new();
    for item in s.split(',') {
        match item.trim().parse() {
            Ok(n) => out.push(n),
            Err(e) => return Err(e),
        }
    }
    Ok(out)
}

/// 8. Um campo opcional que, se presente, precisa ser um número.
pub fn parse_field(field: Option<&str>) -> Result<Option<i32>, ParseIntError> {
    match field {
        None => Ok(None),
        Some(s) => match s.parse() {
            Ok(n) => Ok(Some(n)),
            Err(e) => Err(e),
        },
    }
}

/// 9. Maior de idade, se a idade for conhecida.
pub fn is_adult(age: Option<u32>) -> bool {
    match age {
        Some(a) => a >= 18,
        None => false,
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn t1_parse_or_zero() {
        assert_eq!(parse_or_zero("12"), 12);
        assert_eq!(parse_or_zero("x"), 0);
    }

    #[test]
    fn t2_parse_double() {
        assert_eq!(parse_double("21"), Ok(42));
        assert!(parse_double("x").is_err());
    }

    #[test]
    fn t3_parse_with_message() {
        assert_eq!(parse_with_message("7"), Ok(7));
        assert_eq!(
            parse_with_message("sete"),
            Err("'sete' não é um número: invalid digit found in string".to_string())
        );
    }

    #[test]
    fn t4_first_word() {
        assert_eq!(first_word("  olá mundo"), Ok("olá"));
        assert_eq!(first_word("   "), Err("texto vazio".to_string()));
    }

    #[test]
    fn t5_parse_optional() {
        assert_eq!(parse_optional("5"), Some(5));
        assert_eq!(parse_optional("cinco"), None);
    }

    #[test]
    fn t6_parse_positive() {
        assert_eq!(parse_positive("3"), Ok(3));
        assert_eq!(parse_positive("0"), Err("zero não é positivo".to_string()));
        assert_eq!(
            parse_positive("-3"),
            Err("invalid digit found in string".to_string())
        );
    }

    #[test]
    fn t7_parse_all() {
        assert_eq!(parse_all("1, 2,3"), Ok(vec![1, 2, 3]));
        assert!(parse_all("1,x,3").is_err());
    }

    #[test]
    fn t8_parse_field() {
        assert_eq!(parse_field(None), Ok(None));
        assert_eq!(parse_field(Some("8")), Ok(Some(8)));
        assert!(parse_field(Some("oito")).is_err());
    }

    #[test]
    fn t9_is_adult() {
        assert!(is_adult(Some(30)));
        assert!(!is_adult(Some(10)));
        assert!(!is_adult(None));
    }
}
```

Testes (eles já passam com o código original, e devem continuar passando):

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn t1_parse_or_zero() {
        assert_eq!(parse_or_zero("12"), 12);
        assert_eq!(parse_or_zero("x"), 0);
    }

    #[test]
    fn t2_parse_double() {
        assert_eq!(parse_double("21"), Ok(42));
        assert!(parse_double("x").is_err());
    }

    #[test]
    fn t3_parse_with_message() {
        assert_eq!(parse_with_message("7"), Ok(7));
        assert_eq!(
            parse_with_message("sete"),
            Err("'sete' não é um número: invalid digit found in string".to_string())
        );
    }

    #[test]
    fn t4_first_word() {
        assert_eq!(first_word("  olá mundo"), Ok("olá"));
        assert_eq!(first_word("   "), Err("texto vazio".to_string()));
    }

    #[test]
    fn t5_parse_optional() {
        assert_eq!(parse_optional("5"), Some(5));
        assert_eq!(parse_optional("cinco"), None);
    }

    #[test]
    fn t6_parse_positive() {
        assert_eq!(parse_positive("3"), Ok(3));
        assert_eq!(parse_positive("0"), Err("zero não é positivo".to_string()));
        assert_eq!(
            parse_positive("-3"),
            Err("invalid digit found in string".to_string())
        );
    }

    #[test]
    fn t7_parse_all() {
        assert_eq!(parse_all("1, 2,3"), Ok(vec![1, 2, 3]));
        assert!(parse_all("1,x,3").is_err());
    }

    #[test]
    fn t8_parse_field() {
        assert_eq!(parse_field(None), Ok(None));
        assert_eq!(parse_field(Some("8")), Ok(Some(8)));
        assert!(parse_field(Some("oito")).is_err());
    }

    #[test]
    fn t9_is_adult() {
        assert!(is_adult(Some(30)));
        assert!(!is_adult(Some(10)));
        assert!(!is_adult(None));
    }
}
```

<details>
<summary>Dica</summary>

Rode `cargo clippy` no código original: ele aponta três das reescritas. Para as outras, os nomes que você precisa são: `map`, `map_err`, `ok_or_else`, `and_then`, `collect`, `transpose` e `is_some_and`.

</details>

## 4. Um tipo de erro à mão (médio)

Lição: [04](04-tipos-de-erro-proprios.md).

Implemente `parse_kv`, que interpreta uma linha `chave = valor` (o valor é um `i64`, e espaços em volta são ignorados), e a receita completa do tipo de erro: `Display`, `Error` (com `source()`) e `From<ParseIntError>`. Sem `thiserror` neste exercício. Projeto: `cargo new --lib parse-kv`.

```rust
use std::num::ParseIntError;

#[derive(Debug)]
pub enum ParseError {
    /// A linha não tem `=`.
    MissingEquals,
    /// A chave (antes do `=`) está vazia.
    EmptyKey,
    /// O valor (depois do `=`) não é um `i64`.
    BadNumber(ParseIntError),
}

// TODO: impl Display, impl Error (com source) e impl From<ParseIntError>.

/// Interpreta uma linha `chave = valor`, em que o valor é um inteiro.
pub fn parse_kv(line: &str) -> Result<(String, i64), ParseError> {
    todo!("interpretar {line}")
}
```

As mensagens do `Display` são: ``faltou o `=` ``, `chave vazia` e `valor inválido`. Testes:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error;

    #[test]
    fn linha_valida() {
        assert_eq!(parse_kv("idade = 42").unwrap(), ("idade".to_string(), 42));
        assert_eq!(parse_kv("  x=-7  ").unwrap(), ("x".to_string(), -7));
    }

    #[test]
    fn sem_igual() {
        let e = parse_kv("idade 42").unwrap_err();
        assert!(matches!(e, ParseError::MissingEquals));
        assert_eq!(e.to_string(), "faltou o `=`");
        assert!(e.source().is_none());
    }

    #[test]
    fn chave_vazia() {
        let e = parse_kv("  = 42").unwrap_err();
        assert!(matches!(e, ParseError::EmptyKey));
        assert_eq!(e.to_string(), "chave vazia");
    }

    #[test]
    fn numero_invalido_guarda_a_causa() {
        let e = parse_kv("idade = muitos").unwrap_err();
        assert!(matches!(e, ParseError::BadNumber(_)));
        assert_eq!(e.to_string(), "valor inválido");
        let cause = e.source().expect("BadNumber deveria ter causa");
        assert_eq!(cause.to_string(), "invalid digit found in string");
    }

    #[test]
    fn from_converte_parse_int_error() {
        let pe = "x".parse::<i64>().unwrap_err();
        let e: ParseError = pe.into();
        assert!(matches!(e, ParseError::BadNumber(_)));
    }
}
```

<details>
<summary>Dica</summary>

`split_once('=')` devolve `Option<(&str, &str)>`. Com o `impl From` pronto, o `parse()?` converte o `ParseIntError` sozinho. A ordem das verificações importa: a linha `" = 42"` tem `=`, mas a chave é vazia.

</details>

## 5. Troque os panics por `Result` (médio)

Lições: [01](01-panic-e-erros-irrecuperaveis.md), [03](03-unwrap-expect-e-combinadores.md) e [04](04-tipos-de-erro-proprios.md).

Esta calculadora em notação polonesa reversa (`"3 4 + 2 *"` vale 14) panica com qualquer entrada inválida, e ainda tem um bug escondido: um overflow que no build de release passa em silêncio. Projeto: `cargo new --lib rpn`.

```rust
//! Calculadora em notação polonesa reversa (RPN): `"3 4 + 2 *"` vale 14.

pub fn eval(expr: &str) -> i64 {
    let mut stack: Vec<i64> = Vec::new();
    for token in expr.split_whitespace() {
        match token {
            "+" | "-" | "*" | "/" => {
                let b = stack.pop().expect("faltou operando");
                let a = stack.pop().expect("faltou operando");
                let r = match token {
                    "+" => a + b,
                    "-" => a - b,
                    "*" => a * b,
                    _ => {
                        if b == 0 {
                            panic!("divisão por zero");
                        }
                        a / b
                    }
                };
                stack.push(r);
            }
            _ => stack.push(token.parse().unwrap()),
        }
    }
    assert_eq!(stack.len(), 1, "a pilha deve terminar com um valor");
    stack[0]
}
```

Mude a assinatura para `eval(expr: &str) -> Result<i64, EvalError>`, com este tipo de erro. A posição (`pos`) é o índice do token, a partir de 0:

```rust
use std::error::Error;
use std::fmt;

/// `pos` é a posição do token (a partir de 0) em que o erro aconteceu.
#[derive(Debug, PartialEq)]
pub enum EvalError {
    /// Um operador sem dois números antes dele.
    MissingOperand { pos: usize },
    /// Um token que não é número nem operador.
    BadToken { pos: usize, token: String },
    /// Divisão por zero.
    DivisionByZero { pos: usize },
    /// O resultado não cabe em um `i64`.
    Overflow { pos: usize },
    /// No fim, a pilha deveria ter exatamente um número.
    WrongStackSize { count: usize },
}
```

Implemente também `Display` e `Error` (não há causas, então o bloco do `Error` fica vazio). Testes:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn expressoes_validas() {
        assert_eq!(eval("3 4 +"), Ok(7));
        assert_eq!(eval("3 4 + 2 *"), Ok(14));
        assert_eq!(eval("10 3 -"), Ok(7));
        assert_eq!(eval("7 2 /"), Ok(3));
        assert_eq!(eval("-5"), Ok(-5));
    }

    #[test]
    fn falta_operando() {
        assert_eq!(eval("1 +"), Err(EvalError::MissingOperand { pos: 1 }));
    }

    #[test]
    fn token_invalido() {
        assert_eq!(
            eval("1 x +"),
            Err(EvalError::BadToken {
                pos: 1,
                token: "x".to_string()
            })
        );
    }

    #[test]
    fn divisao_por_zero() {
        assert_eq!(eval("4 0 /"), Err(EvalError::DivisionByZero { pos: 2 }));
    }

    #[test]
    fn overflow() {
        assert_eq!(
            eval("9223372036854775807 1 +"),
            Err(EvalError::Overflow { pos: 2 })
        );
    }

    #[test]
    fn pilha_errada_no_fim() {
        assert_eq!(eval("1 2"), Err(EvalError::WrongStackSize { count: 2 }));
        assert_eq!(eval(""), Err(EvalError::WrongStackSize { count: 0 }));
    }

    #[test]
    fn mensagens() {
        assert_eq!(
            eval("4 0 /").unwrap_err().to_string(),
            "posição 2: divisão por zero"
        );
        assert_eq!(
            eval("1 2").unwrap_err().to_string(),
            "a expressão deixou 2 valores na pilha (esperava 1)"
        );
    }
}
```

<details>
<summary>Dica</summary>

`split_whitespace().enumerate()` dá a posição de cada token. Para o overflow, troque `a + b` por `a.checked_add(b)`, que devolve `Option<i64>`, e use `ok_or`. O mesmo vale para `-`, `*` e `/` (a divisão `i64::MIN / -1` também transborda). Para o `BadToken`, o erro do `parse` não interessa: `map_err(|_| ...)`.

</details>

## 6. De `Box<dyn Error>` para `thiserror` + `anyhow` (médio)

Lição: [05](05-thiserror-e-anyhow.md).

Um pequeno projeto com biblioteca e aplicação. A biblioteca lê um arquivo de notas (`nome;nota` por linha, notas inteiras de 0 a 100) e calcula a média. Hoje ela usa `Box<dyn Error>` e mensagens em `String`, então quem chama não consegue distinguir os erros.

```bash
cargo new notas
cd notas
cargo add thiserror anyhow
```

`src/lib.rs`:

```rust
use std::error::Error;
use std::fs;

pub fn load(path: &str) -> Result<Vec<(String, u32)>, Box<dyn Error>> {
    let text = fs::read_to_string(path)?;
    parse(&text)
}

pub fn parse(text: &str) -> Result<Vec<(String, u32)>, Box<dyn Error>> {
    let mut out = Vec::new();
    for (i, raw) in text.lines().enumerate() {
        let line = i + 1;
        if raw.trim().is_empty() {
            continue;
        }
        let Some((name, grade)) = raw.split_once(';') else {
            return Err(format!("linha {line}: esperava `nome;nota`").into());
        };
        let grade: u32 = grade
            .trim()
            .parse()
            .map_err(|e| format!("linha {line}: nota inválida: {e}"))?;
        if grade > 100 {
            return Err(format!("linha {line}: nota {grade} fora do intervalo 0 a 100").into());
        }
        out.push((name.trim().to_string(), grade));
    }
    Ok(out)
}

pub fn average(notas: &[(String, u32)]) -> Result<f64, Box<dyn Error>> {
    if notas.is_empty() {
        return Err("nenhum aluno no arquivo".into());
    }
    let sum: u32 = notas.iter().map(|(_, g)| g).sum();
    Ok(f64::from(sum) / notas.len() as f64)
}
```

`src/main.rs`:

```rust
fn main() {
    let path = std::env::args().nth(1).expect("uso: notas <arquivo>");
    let notas = notas::load(&path).unwrap();
    let media = notas::average(&notas).unwrap();
    println!("{} alunos, média {media:.1}", notas.len());
}
```

Faça o seguinte:

1. Na biblioteca, crie `pub enum NotasError` com `thiserror` e use-o nas três funções. As variantes e mensagens são:

   | Variante | Mensagem (`Display`) |
   |---|---|
   | `Io { path: String, source: io::Error }` | `não consegui ler {path}` |
   | `MissingSeparator { line: usize }` | ``linha {line}: esperava `nome;nota` `` |
   | `BadGrade { line: usize, source: ParseIntError }` | `linha {line}: nota inválida` |
   | `OutOfRange { line: usize, grade: u32 }` | `linha {line}: nota {grade} fora do intervalo 0 a 100` |
   | `Empty` | `nenhum aluno no arquivo` |

2. Na aplicação, troque os `unwrap`/`expect` por `anyhow`: `main` devolve `anyhow::Result<()>`, a falta de argumento usa `bail!`, e os erros da biblioteca ganham o contexto `"falha ao calcular a média"`.

Testes (no fim do `src/lib.rs`). O último pega a mensagem duplicada:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error;

    #[test]
    fn media_de_uma_turma() {
        let notas = parse("ana;90\n\nbruno; 70\n").unwrap();
        assert_eq!(
            notas,
            vec![("ana".to_string(), 90), ("bruno".to_string(), 70)]
        );
        assert_eq!(average(&notas).unwrap(), 80.0);
    }

    #[test]
    fn variantes() {
        assert!(matches!(
            parse("ana 90"),
            Err(NotasError::MissingSeparator { line: 1 })
        ));
        assert!(matches!(
            parse("ana;90\nbia;101"),
            Err(NotasError::OutOfRange {
                line: 2,
                grade: 101
            })
        ));
        assert!(matches!(average(&[]), Err(NotasError::Empty)));
    }

    #[test]
    fn nota_invalida_tem_causa() {
        let e = parse("ana;90\nbruno;setenta").unwrap_err();
        assert!(matches!(e, NotasError::BadGrade { line: 2, .. }));
        assert_eq!(e.to_string(), "linha 2: nota inválida");
        assert!(e.source().is_some());
    }

    #[test]
    fn arquivo_inexistente_tem_o_caminho_e_a_causa() {
        let e = load("nao-existe.txt").unwrap_err();
        assert_eq!(e.to_string(), "não consegui ler nao-existe.txt");
        let NotasError::Io { source, .. } = &e else {
            panic!("esperava NotasError::Io, veio {e:?}");
        };
        assert_eq!(source.kind(), io::ErrorKind::NotFound);
    }

    #[test]
    fn cadeia_sem_mensagem_duplicada() {
        let e = anyhow::Error::from(parse("bruno;setenta").unwrap_err())
            .context("falha ao calcular a média");
        assert_eq!(
            format!("{e:#}"),
            "falha ao calcular a média: linha 1: nota inválida: invalid digit found in string"
        );
    }
}
```

Rodando a aplicação com um arquivo que tem `bruno;setenta` na linha 2, a saída deve ser:

```text
Error: falha ao calcular a média

Caused by:
    0: linha 2: nota inválida
    1: invalid digit found in string
```

<details>
<summary>Dica</summary>

`Io` e `BadGrade` têm campos extras além da causa, então não dá para usar `#[from]`: marque a causa com `#[source]` e construa a variante com `map_err(|source| NotasError::BadGrade { line, source })`. E não ponha `{source}` no `#[error]`.

</details>

---

[Índice do módulo](README.md)

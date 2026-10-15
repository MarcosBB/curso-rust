# Soluções — Módulo 05

Enunciados em [`modulos/05-tratamento-de-erros/exercicios.md`](../../modulos/05-tratamento-de-erros/exercicios.md).

Cada solução é um projeto Cargo com os testes do enunciado. Para conferir:

```bash
cd nome-do-projeto
cargo test
```

## 1. Espalhe `?`

Projeto: [`espalhe-interrogacao/`](espalhe-interrogacao/src/lib.rs).

```rust
pub fn read_config(path: &str) -> Result<Config, Box<dyn Error>> {
    let text = fs::read_to_string(path)?;
    let mut lines = text.lines();
    let name = lines.next().ok_or("faltou a linha do nome")?.trim().to_string();
    let max_users = lines
        .next()
        .ok_or("faltou a linha de max_users")?
        .trim()
        .parse()?;
    Ok(Config { name, max_users })
}
```

Cada `unwrap` virou um `?`, e os dois que eram em `Option` ganharam um `ok_or` antes, para que o `None` tenha um erro para devolver. Três conversões diferentes acontecem pelo `From`, sem você escrever nenhuma: `io::Error`, `ParseIntError` e `&str` viram `Box<dyn Error>`.

O teste `arquivo_inexistente` mostra o preço do `Box<dyn Error>`: para saber se foi `NotFound`, o chamador precisa de `downcast_ref::<io::Error>()` e precisa saber que tipo procurar. Com um enum (exercício 4), seria um `match`.

## 2. Códigos de saída

Projeto: [`conta-linhas/`](conta-linhas/src/main.rs).

```rust
enum Failure {
    Usage,
    Read { path: String, source: io::Error },
}

fn run() -> Result<usize, Failure> {
    let path = std::env::args().nth(1).ok_or(Failure::Usage)?;
    let text = fs::read_to_string(&path).map_err(|source| Failure::Read { path, source })?;
    Ok(text.lines().count())
}
```

O `main` faz um `match` no resultado de `run()`: imprime e devolve `ExitCode::SUCCESS`, `ExitCode::FAILURE` (1) ou `ExitCode::from(2)`. O enum `Failure` não implementa `Error` porque não precisa: ele nunca sai deste arquivo, e serve só para o `main` saber qual código usar. Repare também que a mensagem de erro de leitura inclui o motivo (`{source}`) no texto: como ninguém percorre a cadeia de causas aqui, é a única forma de ele aparecer.

Por que não `process::exit(2)` dentro de `run`? Porque aí `run` não poderia ser testada nem reaproveitada, e os `Drop` pendentes não rodariam. O Projeto 1 usa exatamente esse formato.

## 3. Combinadores

Projeto: [`combinadores/`](combinadores/src/lib.rs).

| # | Reescrita |
|---|---|
| 1 | `s.parse().unwrap_or_default()` (ou `unwrap_or(0)`) |
| 2 | `s.parse::<i32>().map(\|n\| n * 2)` |
| 3 | `s.parse().map_err(\|e\| format!("'{s}' não é um número: {e}"))` |
| 4 | `s.split_whitespace().next().ok_or_else(\|\| "texto vazio".to_string())` |
| 5 | `s.parse().ok()` |
| 6 | `s.parse::<u32>().map_err(\|e\| e.to_string()).and_then(\|n\| if n > 0 { Ok(n) } else { Err(...) })` |
| 7 | `s.split(',').map(\|item\| item.trim().parse()).collect()` |
| 8 | `field.map(\|s\| s.parse()).transpose()` |
| 9 | `age.is_some_and(\|a\| a >= 18)` |

Os três que o `clippy` aponta no código original são o 1 (`unwrap_or_default`), o 5 (`ok()`) e o 7 (o `match` dentro do laço pode virar `?`; dali para o `collect` é um passo).

No 4, `ok_or_else` evita criar a `String` quando há palavra. Com `ok_or("texto vazio".to_string())`, a `String` seria alocada sempre. No 6, a ordem importa: o `map_err` vem antes do `and_then` para que os dois lados tenham o mesmo tipo de erro (`String`). No 7, o tipo de retorno da função (`Result<Vec<i32>, _>`) é o que diz ao `collect` para parar no primeiro erro.

## 4. Um tipo de erro à mão

Projeto: [`parse-kv/`](parse-kv/src/lib.rs).

```rust
pub fn parse_kv(line: &str) -> Result<(String, i64), ParseError> {
    let (key, value) = line.split_once('=').ok_or(ParseError::MissingEquals)?;
    let key = key.trim();
    if key.is_empty() {
        return Err(ParseError::EmptyKey);
    }
    let value: i64 = value.trim().parse()?;
    Ok((key.to_string(), value))
}
```

Os três blocos `impl` seguem a receita da lição 04. Pontos de atenção:

- O `Display` de `BadNumber` é só `"valor inválido"`. O texto do `ParseIntError` sai pelo `source()`, e o teste `numero_invalido_guarda_a_causa` confere as duas coisas separadamente.
- No `source()`, as variantes sem causa estão listadas explicitamente (`MissingEquals | EmptyKey => None`), e não com `_`. Assim, se você acrescentar uma variante com causa, o compilador obriga a decidir o que o `source()` devolve para ela.
- Sem o `impl From<ParseIntError>`, o `parse()?` não compila (é o erro E0277 da lição 02).

## 5. Troque os panics por `Result`

Projeto: [`rpn/`](rpn/src/lib.rs).

Cada panic do original virou um erro:

| Original | Solução |
|---|---|
| `stack.pop().expect(...)` | `stack.pop().ok_or(EvalError::MissingOperand { pos })?` |
| `token.parse().unwrap()` | `token.parse().map_err(\|_\| EvalError::BadToken { pos, token: token.to_string() })?` |
| `panic!("divisão por zero")` | `return Err(EvalError::DivisionByZero { pos })` |
| `a + b` (overflow: panic em debug, valor errado em release) | `a.checked_add(b)` + `ok_or(EvalError::Overflow { pos })?` |
| `assert_eq!(stack.len(), 1, ...)` | `if stack.len() != 1 { return Err(EvalError::WrongStackSize { .. }) }` |

O overflow era o bug escondido: no build de debug, `9223372036854775807 1 +` panica, e no release devolve um número negativo sem avisar ninguém. Com `checked_*`, as duas versões devolvem `Overflow`. A divisão também usa `checked_div`, porque `i64::MIN / -1` transborda.

O `BadToken` descarta o `ParseIntError` (`map_err(|_| ...)`) de propósito: a mensagem útil para o usuário é o token e a posição, e não "invalid digit found in string". Por isso o `impl Error` fica vazio: nenhuma variante tem causa.

Esse é o roteiro geral para transformar código que panica em código que devolve erros: cada `unwrap`/`expect` vira `?` (com `ok_or` ou `map_err` quando precisa), cada `panic!` vira `return Err(...)`, e cada operação que "não pode falhar" é conferida mais uma vez.

## 6. De `Box<dyn Error>` para `thiserror` + `anyhow`

Projeto: [`notas/`](notas/). Biblioteca em [`src/lib.rs`](notas/src/lib.rs), aplicação em [`src/main.rs`](notas/src/main.rs).

O tipo de erro:

```rust
#[derive(Debug, Error)]
pub enum NotasError {
    #[error("não consegui ler {path}")]
    Io {
        path: String,
        #[source]
        source: io::Error,
    },

    #[error("linha {line}: esperava `nome;nota`")]
    MissingSeparator { line: usize },

    #[error("linha {line}: nota inválida")]
    BadGrade {
        line: usize,
        #[source]
        source: ParseIntError,
    },

    #[error("linha {line}: nota {grade} fora do intervalo 0 a 100")]
    OutOfRange { line: usize, grade: u32 },

    #[error("nenhum aluno no arquivo")]
    Empty,
}
```

E a aplicação:

```rust
fn main() -> Result<()> {
    let Some(path) = std::env::args().nth(1) else {
        bail!("uso: notas <arquivo>");
    };
    let notas = notas::load(&path).context("falha ao calcular a média")?;
    let media = notas::average(&notas).context("falha ao calcular a média")?;
    println!("{} alunos, média {media:.1}", notas.len());
    Ok(())
}
```

```bash
cargo run -- exemplos/turma.txt          # 3 alunos, média 82.3
cargo run -- exemplos/nota-invalida.txt  # o erro abaixo, código 1
```

```text
Error: falha ao calcular a média

Caused by:
    0: linha 2: nota inválida
    1: invalid digit found in string
```

O que mudou em relação ao original:

- **O chamador pode reagir.** Os testes usam `matches!` em cada variante, coisa impossível com `Box<dyn Error>` contendo uma `String`.
- **A causa não se perde.** No original, `map_err(|e| format!("...: {e}"))` transformava o `ParseIntError` em texto. Agora ele fica guardado no campo `source`, e o `anyhow` mostra a cadeia.
- **A mensagem não se repete.** O original tinha a causa no texto. Se você só trocasse o `format!` por `#[error("linha {line}: nota inválida: {source}")]`, o teste `cadeia_sem_mensagem_duplicada` falharia com `...nota inválida: invalid digit found in string: invalid digit found in string`.
- **`#[source]`, e não `#[from]`.** `Io` e `BadGrade` têm campos além da causa (`path`, `line`), que um `From` não teria de onde tirar. Por isso o `map_err` constrói a variante no ponto onde esses dados existem.

A biblioteca não expõe nada do `anyhow`. O `anyhow` aparece só no `main.rs` e no teste que confere o formato `{:#}`.

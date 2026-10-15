# 04 — Tipos de erro próprios

**Objetivos.** Ao final, você vai saber:

- por que `String` é um tipo de erro ruim, e como modelar os erros com um enum;
- implementar `Display`, `Error` e `From` no seu tipo de erro (como receita);
- percorrer a cadeia de causas com `source()`;
- quando `Box<dyn Error>` basta.

## O problema: duas causas, um tipo de retorno

Na lição 02, esta função não compilou:

```rust
// ❌ NÃO COMPILA (o erro E0277 está na lição 02)
fn read_port(path: &str) -> Result<u16, io::Error> {
    let text = fs::read_to_string(path)?;  // falha com io::Error
    let port = text.trim().parse::<u16>()?; // falha com ParseIntError
    Ok(port)
}
```

Ela tem duas formas de falhar, com dois tipos de erro, e o `Result` só tem lugar para um. A primeira ideia costuma ser usar `String` para tudo:

```rust
// ❌ NÃO COMPILA
fn read_port(text: &str) -> Result<u16, String> {
    let port: u16 = text.trim().parse()?;
    Ok(port)
}
```

```text
error[E0277]: `?` couldn't convert the error to `String`
 --> src/main.rs:2:40
  |
1 | fn read_port(text: &str) -> Result<u16, String> {
  |                             ------------------- expected `String` because of this
2 |     let port: u16 = text.trim().parse()?;
  |                                 -------^ the trait `From<ParseIntError>` is not implemented for `String`
...
```

Dá para resolver com `.map_err(|e| e.to_string())?` em cada chamada, e o Rust Book começa assim no capítulo do minigrep. Funciona, mas tem três problemas:

1. **Quem chama não consegue distinguir os casos.** Para saber se foi "arquivo não existe" ou "porta inválida", teria que comparar textos.
2. **A causa original se perde.** O `io::Error` (com o seu `kind()`) vira uma frase.
3. **`String` não implementa a *trait* `Error`**, então não se encaixa nas ferramentas que vamos ver (cadeia de causas, `Box<dyn Error>`, `anyhow`).

## Um enum com uma variante por causa

O jeito idiomático é um enum, com uma variante para cada forma de falhar. As variantes carregam os dados úteis, inclusive o erro original:

```rust
use std::io;
use std::num::ParseIntError;

#[derive(Debug)]
enum ConfigError {
    /// Não deu para ler o arquivo.
    Io(io::Error),
    /// O conteúdo não é um número de 0 a 65535.
    BadPort(ParseIntError),
    /// O número é válido, mas é uma porta reservada.
    PortTooLow(u16),
}
```

Agora o chamador pode fazer `match` e reagir a cada caso, e o erro de baixo continua disponível. Falta ensinar ao Rust três coisas sobre esse tipo: como ele aparece para humanos (`Display`), que ele é um erro e qual é a sua causa (`Error`), e como converter os erros de baixo nele (`From`, para o `?` funcionar).

> [!NOTE]
> Os três blocos a seguir são **implementações de *traits***. Uma *trait* é um conjunto de métodos que um tipo promete ter (parecido com uma interface). A sintaxe é `impl NomeDaTrait for SeuTipo { fn ... }`. O [Módulo 06](../06-traits-e-generics/README.md) explica tudo isso em detalhe. Aqui, trate os três blocos como uma **receita**: copie a forma e troque as variantes. Na lição 05, a crate `thiserror` vai escrever essa receita por você.

### Receita 1: `Display`, a mensagem para humanos

```rust
use std::fmt;

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigError::Io(_) => write!(f, "não consegui ler o arquivo de configuração"),
            ConfigError::BadPort(_) => write!(f, "porta inválida"),
            ConfigError::PortTooLow(p) => write!(f, "a porta {p} é reservada (use 1024 ou mais)"),
        }
    }
}
```

O método `fmt` recebe o erro (`self`) e escreve o texto com `write!`, que funciona como `format!`. É ele que roda quando você imprime o erro com `{}` ou chama `.to_string()`.

Siga o estilo da biblioteca padrão para a mensagem: **minúsculas, sem ponto final, curta**, e descrevendo só esta camada do erro. Repare que `Io(_)` e `BadPort(_)` **não** incluem o erro de dentro no texto. Ele vai aparecer pelo `source()`, logo abaixo.

### Receita 2: `Error`, que liga o erro à causa

```rust
use std::error::Error;

impl Error for ConfigError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            ConfigError::Io(e) => Some(e),
            ConfigError::BadPort(e) => Some(e),
            ConfigError::PortTooLow(_) => None,
        }
    }
}
```

`std::error::Error` é a trait que todo tipo de erro da biblioteca padrão implementa. O único método que interessa aqui é `source()`: ele devolve o erro que **causou** este, se houver. `PortTooLow` não tem causa (é uma regra sua), então devolve `None`.

O tipo de retorno assusta, mas leia como "talvez uma referência para algum erro". O `dyn Error` quer dizer "qualquer tipo que implemente `Error`", e o `'static` é um detalhe que o Módulo 06 explica. Você não precisa mudar essa linha.

`Error` exige que o tipo já tenha `Debug` (o `#[derive(Debug)]`) e `Display`. Se faltar o `Display`, o compilador avisa:

```rust
// ❌ NÃO COMPILA
#[derive(Debug)]
enum ConfigError {
    PortTooLow(u16),
}

impl std::error::Error for ConfigError {}
```

```text
error[E0277]: `ConfigError` doesn't implement `std::fmt::Display`
  --> src/main.rs:6:28
   |
 6 | impl std::error::Error for ConfigError {}
   |                            ^^^^^^^^^^^ unsatisfied trait bound
   |
help: the trait `std::fmt::Display` is not implemented for `ConfigError`
...
note: required by a bound in `std::error::Error`
...
59 | pub trait Error: Debug + Display {
   |                          ^^^^^^^ required by this bound in `Error`
```

Repare também no bloco vazio `{}`: todos os métodos de `Error` têm uma implementação padrão (`source()` devolve `None`), então você só escreve o que quer mudar.

### Receita 3: `From`, para o `?` converter sozinho

```rust
impl From<io::Error> for ConfigError {
    fn from(e: io::Error) -> Self {
        ConfigError::Io(e)
    }
}

impl From<ParseIntError> for ConfigError {
    fn from(e: ParseIntError) -> Self {
        ConfigError::BadPort(e)
    }
}
```

Cada bloco diz "sei criar um `ConfigError` a partir de um `io::Error`" (ou de um `ParseIntError`). Lembra que o `?` chama `From::from(e)` antes de devolver o erro? É aqui que ele encontra a conversão. Com os três blocos, a função original compila sem nenhum `map_err`:

```rust
fn read_port(path: &str) -> Result<u16, ConfigError> {
    let text = fs::read_to_string(path)?; // io::Error -> ConfigError::Io
    let port: u16 = text.trim().parse()?; // ParseIntError -> ConfigError::BadPort
    if port < 1024 {
        return Err(ConfigError::PortTooLow(port));
    }
    Ok(port)
}
```

## A cadeia de causas

Como cada erro pode ter uma causa, que pode ter outra causa, os erros formam uma lista encadeada. Para mostrar tudo ao usuário, percorra a cadeia com `source()`:

```rust
fn report(e: &dyn Error) {
    eprintln!("erro: {e}");
    let mut cause = e.source();
    while let Some(c) = cause {
        eprintln!("  causado por: {c}");
        cause = c.source();
    }
}
```

O código completo, com um `main` que devolve `ExitCode` e testes, está em [`codigo/erro-manual`](codigo/erro-manual/). Rodando com cada arquivo de exemplo:

```bash
cargo run -- exemplos/porta-invalida.txt
cargo run -- exemplos/porta-baixa.txt
cargo run -- nao-existe.txt
```

```text
erro: porta inválida
  causado por: invalid digit found in string
erro: a porta 80 é reservada (use 1024 ou mais)
erro: não consegui ler o arquivo de configuração
  causado por: No such file or directory (os error 2)
```

```mermaid
flowchart LR
    A["ConfigError::Io<br/>&quot;não consegui ler o arquivo<br/>de configuração&quot;"] -->|"source()"| B["io::Error<br/>&quot;No such file or directory<br/>(os error 2)&quot;"]
    B -->|"source()"| C["None"]
```

É por isso que o `Display` de cada camada não deve repetir a causa: quem imprime a cadeia já mostra cada uma uma vez. Se `BadPort` escrevesse `"porta inválida: {e}"`, a mensagem de baixo apareceria duas vezes. A lição 05 mostra essa duplicação acontecendo de verdade.

Compare com Python, que tem a mesma ideia com `raise ... from`:

```python
class ConfigError(Exception):
    pass

try:
    port = int(text)
except ValueError as e:
    raise ConfigError("porta inválida") from e   # e vira __cause__
```

O `__cause__` do Python é o `source()` do Rust, e o traceback com "The above exception was the direct cause of the following exception" é o equivalente ao `report` acima. Em C++, o paralelo é `std::throw_with_nested` com `std::rethrow_if_nested`, bem menos usado na prática.

## `Box<dyn Error>`: qualquer erro

Às vezes você não precisa que o chamador distinga os casos: só quer propagar o erro e mostrar a mensagem. Para isso existe `Box<dyn Error>`, que você viu na sugestão do compilador na lição 02. Leia como "uma caixa que guarda qualquer erro". O `?` converte qualquer tipo que implemente `Error` para ele, e até uma `String` vira `Box<dyn Error>` com `.into()`:

```rust
use std::error::Error;
use std::fs;

fn read_port(path: &str) -> Result<u16, Box<dyn Error>> {
    let text = fs::read_to_string(path)?; // io::Error vira Box<dyn Error>
    let port: u16 = text.trim().parse()?; // ParseIntError também
    if port < 1024 {
        return Err(format!("a porta {port} é reservada").into()); // e String também
    }
    Ok(port)
}
```

Sem enum e sem receita. O preço: o tipo concreto fica escondido. O chamador ainda pode perguntar "é um `ParseIntError`?" com `downcast_ref`, mas precisa saber o que procurar, e o compilador não ajuda a cobrir os casos:

```rust
use std::error::Error;

let e: Box<dyn Error> = "abc".parse::<u16>().unwrap_err().into();
if let Some(pe) = e.downcast_ref::<std::num::ParseIntError>() {
    println!("era um ParseIntError: {:?}", pe.kind()); // era um ParseIntError: InvalidDigit
}
```

A regra prática:

| Situação | Tipo de erro |
|---|---|
| biblioteca, ou código cujo chamador precisa reagir a casos diferentes | enum próprio |
| aplicação, `main`, scripts, protótipos: só importa propagar e mostrar | `Box<dyn Error>` (ou `anyhow`, na lição 05) |

Se o erro precisar atravessar threads (Módulo 11), o tipo usado é `Box<dyn Error + Send + Sync>`.

## Comparação

| Python | C++ | Rust |
|---|---|---|
| `class ConfigError(Exception)` | `class ConfigError : public std::runtime_error` | `enum ConfigError` + `Display` + `Error` |
| mensagem no construtor / `__str__` | `what()` | `Display` |
| `raise ... from e` (`__cause__`) | `std::throw_with_nested` | `source()` |
| `except Exception as e` | `catch (const std::exception& e)` | `Box<dyn Error>` |
| subclasses de exceção | hierarquia de classes | variantes do enum (com `match` exaustivo) |

O que se ganha com o enum: o `match` é exaustivo, então, se você acrescentar uma variante, o compilador mostra todo lugar que precisa tratar o caso novo. Uma hierarquia de exceções não avisa nada. O que se perde: escrever a receita à mão é trabalhoso. É exatamente o problema que a lição 05 resolve.

## Armadilhas comuns

**`String` como erro de biblioteca.** O chamador não consegue casar variantes e não tem acesso à causa. Use um enum.

**Repetir a causa no `Display`.** `write!(f, "porta inválida: {e}")` mais `source()` devolvendo `e` faz a mensagem de baixo aparecer duas vezes para quem percorre a cadeia. Escolha um: ou a causa vai no texto, ou vai no `source()` (o recomendado).

**`Box<dyn Error>` na API pública de uma biblioteca.** Quem usa sua biblioteca perde a capacidade de reagir a casos específicos sem `downcast`. Em biblioteca, enum.

**Esquecer de implementar `From` e espalhar `map_err`.** Se um tipo de erro aparece em vários `?`, um `impl From` resolve todos de uma vez. Use `map_err` quando precisar acrescentar informação que o erro de baixo não tem (como o caminho do arquivo, na lição 05).

## Resumo

- `String` como erro é simples, mas impede o chamador de distinguir casos e joga fora a causa.
- Um enum com uma variante por causa é o tipo de erro idiomático. As variantes carregam o erro original.
- A receita: `#[derive(Debug)]`, `impl Display` (mensagem curta, em minúsculas, sem a causa), `impl Error` (com `source()`) e um `impl From` para cada erro de baixo.
- `source()` forma uma cadeia de causas, que você percorre com `while let`.
- `Box<dyn Error>` aceita qualquer erro e é ótimo para aplicações. Em bibliotecas, prefira o enum.

## Exercícios

Veja o exercício 4 em [`exercicios.md`](exercicios.md).

## Leitura complementar

- [Documentação de `std::error`](https://doc.rust-lang.org/std/error/index.html), em especial ["Common Message Styles"](https://doc.rust-lang.org/std/error/index.html#common-message-styles)
- [Documentação da trait `Error`](https://doc.rust-lang.org/std/error/trait.Error.html)
- [The Rust Programming Language, 12.3 (o minigrep com `Box<dyn Error>`)](https://doc.rust-lang.org/stable/book/ch12-03-improving-error-handling-and-modularity.html)
- [Rust API Guidelines: C-GOOD-ERR](https://rust-lang.github.io/api-guidelines/interoperability.html#error-types-are-meaningful-and-well-behaved-c-good-err)
- [Effective Rust, Item 4: Prefer idiomatic `Error` types](https://effective-rust.com/errors.html)

---

[← Lição anterior](03-unwrap-expect-e-combinadores.md) · [Índice do módulo](README.md) · [Próxima lição →](05-thiserror-e-anyhow.md)

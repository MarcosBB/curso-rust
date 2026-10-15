# Solução — Projeto 1: `minigrep` melhorado

Enunciado em [`projetos/01-minigrep/README.md`](../../projetos/01-minigrep/README.md).

```bash
cd solucoes/projeto-01-minigrep
cargo test                         # 25 unitários + 10 de integração
cargo run -q -- -n -i how poem.txt
```

Os arquivos têm os mesmos tipos, assinaturas e testes do projeto [`inicial/`](../../projetos/01-minigrep/inicial/). Só os corpos das funções mudam.

| Arquivo | O que tem |
|---|---|
| [`src/search.rs`](src/search.rs) | `Match<'a>`, `SearchOptions`, `search` (etapas 1 e 2) |
| [`src/config.rs`](src/config.rs) | `Input`, `OutputOptions`, `Config`, `ArgsError`, `Config::from_args` (etapa 3) |
| [`src/lib.rs`](src/lib.rs) | `RunError`, `read_input`, `render` (etapas 4 e 5) |
| [`src/main.rs`](src/main.rs) | `main` e `run`: ambiente, saída, códigos de saída, pipe fechado (etapa 6) |
| [`tests/cli.rs`](tests/cli.rs) | testes de integração do executável |

## Decisões

**A biblioteca devolve dados.** Nenhuma função da lib imprime, lê o ambiente ou encerra o processo. `render` devolve `Vec<String>` em vez de escrever, e `from_args` recebe `ignore_case_env: bool` em vez de chamar `env::var`. O custo é pequeno (o `main` tem um laço a mais), e o ganho é que 25 testes rodam sem tocar em stdout, stderr ou variáveis de ambiente. A versão em que `run` recebe um `&mut impl Write` fica para o Módulo 06, quando você souber o que é `impl Trait`.

**`&[String]` e cópias.** `from_args` recebe uma fatia emprestada, como no Book, e por isso copia o padrão (`to_string`), a opção desconhecida (`clone`) e os caminhos (`PathBuf::from`). São poucas strings, uma vez por execução. A versão que move as `String` em vez de copiar consome o iterador de `env::args()` e fica para o Módulo 07.

**Lifetime só em `contents`.** `search<'a>(pattern: &str, contents: &'a str, ..) -> Vec<Match<'a>>`. O teste `pattern_can_be_dropped_before_results` existe para garantir que ninguém "conserte" a assinatura seguindo o `help` do E0106, que amarraria o padrão ao resultado sem necessidade.

**Padrão vazio casa com tudo.** `"abc".contains("")` é `true`, e o `grep ''` faz o mesmo. Não há caso especial no código, só um teste que documenta o comportamento.

**`IGNORE_CASE` só liga.** A variável e o `-i` fazem a mesma coisa, e não existe opção para desligar. Então não há conflito de precedência: `ignore_case = env || flag`. Um `--no-ignore-case` exigiria decidir quem vence, e a escolha natural seria a linha de comando.

**Erros à mão, sem `thiserror`.** Os três `impl` (`Display`, `Error`, `source`) são a receita do Módulo 05, e o projeto fica sem dependências. Com `thiserror`, `ArgsError` e `RunError` encolhem para os `#[error("...")]` em cima de cada variante. Numa biblioteca publicada, eu usaria `thiserror`. O `anyhow` não ajudaria aqui: o `main` precisa distinguir o erro de argumento (código 2 com a linha de uso) do erro de escrita (pipe fechado vira 0), e isso fica mais claro com tipos concretos.

**A causa dentro do `Display` de `RunError`.** O Módulo 05 recomenda não repetir no `Display` a causa que já sai por `source()`. Aqui o `main` imprime só `{e}`, e a mensagem precisa dizer o motivo (`nao-existe.txt: No such file or directory (os error 2)`). A troca está comentada no código. `source()` continua devolvendo o `io::Error`, para quem quiser inspecionar a causa.

**Erros de leitura não são fatais.** `run` reporta o `RunError` no stderr e segue com `continue`. Só um erro de **escrita** interrompe o laço (via `?`), porque, se o stdout não aceita mais nada, não adianta continuar. O código de saída é 2 se qualquer entrada falhou, mesmo que outras tenham casado, como no `grep`.

**Pipe fechado.** `println!` dá panic quando a escrita falha. `run` usa `writeln!` num `io::stdout().lock()` e propaga o `io::Error`, e o `main` trata `ErrorKind::BrokenPipe` como fim normal (código 0, nada no stderr). O `stdout.flush()?` no fim de `run` é uma garantia: hoje o stdout esvazia o buffer a cada `\n`, mas, se um dia a saída ganhar um buffer maior (um `BufWriter`, por exemplo), um erro na última escrita só apareceria no *flush*, e sem ele se perderia em silêncio. O teste `closed_pipe_is_not_a_panic` fecha a leitura da saída antes de mandar a entrada. Com `println!` no lugar de `writeln!`, ele falha com código 101 (o código de um panic).

**`ExitCode` com constantes.** `FOUND`, `NOT_FOUND` e `ERROR` deixam o `main` legível e evitam números soltos. `run` devolve um `u8` para que a lógica dos códigos fique num lugar só, e o `main` só converte com `ExitCode::from`.

**Laços `for` em vez de iteradores.** A busca, o parser e os testes usam `for` e `push`, que é o que o Módulo 01 apresentou. A mesma lógica com `filter`, `map` e `collect` é o primeiro exercício de retomada no Módulo 07.

## Comparação com o minigrep do Book

| | Book (cap. 12) | Esta solução |
|---|---|---|
| Entradas | um arquivo | vários arquivos, `-` e stdin |
| Opções | só `IGNORE_CASE` | `-i -n -v -c --` e `IGNORE_CASE` |
| Resultado da busca | `Vec<&'a str>` | `Vec<Match<'a>>`, com o número da linha |
| Erros | `&'static str` e `Box<dyn Error>` | `ArgsError` e `RunError` |
| Arquivo com problema | encerra o programa | aviso e continua |
| Código de saída | sempre 1 em erro | 0, 1 ou 2 |
| `\| head` | panic | termina normalmente |
| Testes | 2 unitários | 25 unitários + 10 de integração |

---

[Enunciado do projeto](../../projetos/01-minigrep/README.md) · [Índice do curso](../../README.md)

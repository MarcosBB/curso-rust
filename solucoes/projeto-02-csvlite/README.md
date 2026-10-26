# Solução — Projeto 2: `csvlite`

Enunciado em [`projetos/02-csvlite/README.md`](../../projetos/02-csvlite/README.md).

```bash
cd solucoes/projeto-02-csvlite
cargo test --workspace                         # unitários, api.rs, cli.rs e doctests
cargo run -q -p csvq -- -d ';' -c nome,nota alunos.csv
cargo bench -p csvlite --bench parse -- --warm-up-time 1 --measurement-time 3
```

Os arquivos têm os mesmos tipos, assinaturas e testes do [`inicial/`](../../projetos/02-csvlite/inicial/). Mudam os corpos, o `FromRecord for Person` de `tests/api.rs` (que o aluno escreve na etapa 6) e a documentação da etapa 8.

| Arquivo | O que tem |
|---|---|
| [`csvlite/src/reader.rs`](csvlite/src/reader.rs) | `Options`, `Reader` e a máquina de estados (etapas 1 a 3) |
| [`csvlite/src/error.rs`](csvlite/src/error.rs) | `Error`, `ErrorKind`, `Display`, `Error::invalid_field` (etapas 3 e 6) |
| [`csvlite/src/record.rs`](csvlite/src/record.rs), [`table.rs`](csvlite/src/table.rs) | `Record`, `IntoIterator for &Record`, `Table` (etapas 4 e 6) |
| [`csvlite/src/writer.rs`](csvlite/src/writer.rs) | `write_record` (etapa 5) |
| [`csvlite/tests/api.rs`](csvlite/tests/api.rs) | propriedade de ida e volta e `FromRecord` (etapas 5 e 6) |
| [`csvq/src/main.rs`](csvq/src/main.rs), [`csvq/tests/cli.rs`](csvq/tests/cli.rs) | a CLI e seus testes (etapa 7) |
| [`csvlite/benches/parse.rs`](csvlite/benches/parse.rs) | benchmark (etapa 9) |

## Decisões

**Uma função de "fim de campo".** `end_of_field(c)` decide se um caractere já consumido termina o campo (separador, `\n`, `\r\n`) e é usada tanto depois de um campo sem aspas quanto depois de fechar aspas. Assim, `\r\n` é tratado num lugar só, e o erro `CharAfterQuote` é simplesmente "não era fim de campo".

**Posição só no `bump`.** Todo caractere passa por `bump`, que atualiza `line` e `column`. As posições dos erros saem de dois `let (line, column) = ...` guardados no momento certo: antes da aspa de abertura (`UnterminatedQuote`) e antes do caractere depois das aspas (`CharAfterQuote`). `FieldCount` aponta para o início do registro, coluna 1.

**Linha em branco é um registro com um campo vazio.** O Python devolve `[]`. Aqui, `""` vira `[""]`, o que mantém a ida e volta simples: `write_record(&[""])` escreve só `\n`, que é lido de volta como `[""]`. No modo estrito, uma linha em branco no meio de um CSV de 3 colunas é um `FieldCount`, o que costuma ser o que se quer.

**`"` no meio de um campo sem aspas é texto.** Mais permissivo que a RFC, igual ao Python. O teste `quote_in_the_middle` documenta isso.

**`pub(crate)` nos campos de `Record`.** O `Reader` monta o registro direto, e os testes das etapas 1 a 3 leem `record.fields` antes de existirem os métodos da etapa 4. Para quem usa a crate, os campos continuam privados.

**`IntoIter` com ponteiro de função.** `Map<slice::Iter<'a, String>, fn(&String) -> &str>` é o tipo mais simples que tem nome na stable; `impl Trait` em tipo associado ainda é instável (E0658).

**Mensagens.** Os campos aparecem contando a partir de 1 (`campo 2`, o `index: 1`), porque é assim que uma pessoa conta. O `Display` pluraliza ("1 campo", "2 campos"). `Error::invalid_field` existe para que toda implementação de `FromRecord` produza erros no mesmo formato, com a linha do registro.

**A `csvq` traduz, não decide.** Toda a lógica de CSV está na biblioteca. A CLI só lê argumentos, lê a entrada e transforma `CliError` em mensagem e código 2. O `run` recebe o texto e um `&mut impl Write`, e por isso tem testes unitários sem rodar o executável. O pipe fechado vira código 0, como no Projeto 1.

**Sem `unwrap` fora dos testes.** O `.get(i).unwrap_or_default()` de `column` e de `run` cobre linhas curtas no modo flexível com `""`.

**Proptest com 64 casos.** `ProptestConfig::with_cases(64)` deixa o teste em cerca de 0,1 s e ainda acha os dois bugs da etapa 5 na primeira execução.

**Benchmark.** `reader_iter` e `table_parse` dão cerca de 2,39 ms cada (10 mil registros de 3 campos). O custo está nas 30 mil `String` alocadas, e não em guardar os registros. O `Cow` do Módulo 10 é o próximo passo.

**Workspace dentro do curso.** O curso não tem workspace na raiz do repositório. Este é independente: não há `Cargo.toml` acima dele, então `cargo` rodado aqui dentro trata a pasta como um projeto só.

---

[Enunciado do projeto](../../projetos/02-csvlite/README.md) · [Índice do curso](../../README.md)

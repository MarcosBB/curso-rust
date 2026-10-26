# Solução — Projeto 3: `tarefas`

Enunciado em [`projetos/03-tarefas/README.md`](../../projetos/03-tarefas/README.md).

```bash
cd solucoes/projeto-03-tarefas
cargo test                                         # 46 unitários + 6 do executável
cargo run -q -- --arquivo /tmp/t.json add Estudar o Módulo 09 -p alta
cargo run -q -- --arquivo /tmp/t.json list
```

Os arquivos têm os mesmos tipos, assinaturas e testes do [`inicial/`](../../projetos/03-tarefas/inicial/), mais os três testes que o enunciado pede para o aluno escrever, marcados com o comentário "Escrito pelo aluno" (`salvar_substitui_o_arquivo_anterior`, o caso `so_prioridade` e `estatisticas_nao_grava`).

| Arquivo | O que tem |
|---|---|
| [`src/tarefa.rs`](src/tarefa.rs) | `Prioridade`, `Tarefa`, atributos do `serde` (etapas 1 e 2) |
| [`src/lista.rs`](src/lista.rs) | `ListaDeTarefas` (etapa 3) |
| [`src/armazenamento.rs`](src/armazenamento.rs) | trait `Armazenamento`, `ArquivoJson`, `Memoria` (etapa 4) |
| [`src/comando.rs`](src/comando.rs) | `Comando::de_args` com slice patterns (etapa 5) |
| [`src/app.rs`](src/app.rs) | `executar` (etapa 6) |
| [`src/main.rs`](src/main.rs), [`tests/cli.rs`](tests/cli.rs) | o executável e seus testes (etapa 7) |

## Decisões

**`executar` na biblioteca, com tudo por parâmetro.** Arquivo (`&mut impl Armazenamento`), saída (`&mut impl Write`) e relógio (`agora: u64`). Por isso a regra de "grava só quando muda" tem teste unitário (`memoria.gravacoes`), sem processo nem disco. O `main` ficou com o que não dá para evitar: ler argumentos e ambiente, escolher o caminho, ler o relógio e traduzir o erro.

**`impl Trait`, e não `dyn`.** Só existe um armazenamento por execução, escolhido na compilação de cada chamador (o `main` usa `ArquivoJson`; os testes, `Memoria`). Dispatch estático basta. Um `Box<dyn Armazenamento>` só valeria a pena se o tipo fosse escolhido em tempo de execução (por exemplo, `--memoria`).

**Erros com `thiserror` e causa separada.** O `Display` de `Io` e `Json` diz o contexto (qual arquivo), e o `#[source]` guarda a causa. O `main` percorre `source()` e monta `tarefas: contexto: causa`. Assim a mensagem do `serde_json`, que já traz linha e coluna, aparece uma vez só. O `thiserror` mostra um `PathBuf` com `.caminho.display()` dentro do `#[error(...)]`.

**Códigos de saída.** `Erro::Uso` sai com 2 e a linha de uso; o resto com 1. Um `Erro::Saida` de pipe fechado sai com 0 em silêncio, como no Projeto 1.

**O comando antes do caminho.** `rodar` interpreta o comando antes de procurar `TAREFAS_ARQUIVO` e a pasta pessoal. Um erro de uso aparece mesmo sem `HOME`, e nunca cria arquivo (o teste `uso_errado_sai_com_2` confere).

**`env::home_dir()` da std.** Ele não está mais depreciado e devolve `$HOME` quando a variável existe. Isso dispensa a crate `dirs` e permite o teste `sem_arquivo_usa_a_pasta_pessoal`, que aponta `HOME` para uma pasta temporária com `Command::env`, só no processo filho.

**Gravação.** `fs::write` no `.tmp` e `fs::rename` por cima, na mesma pasta. O único `expect` de produção está no `to_string_pretty`, com um comentário: a serialização desses tipos não tem como falhar.

**Data em segundos UNIX.** `u64`, sem `chrono` nem `jiff`. A data legível fica em "Para ir além".

**`clippy::unwrap_used` + `allow-unwrap-in-tests`.** O lint vale para o código de produção. No `tests/cli.rs`, as funções auxiliares não são `#[test]`, e por isso o arquivo tem um `#![allow(clippy::unwrap_used, reason = ...)]` explícito.

**Ordem da listagem.** `sort_by_key(|t| (t.feita, Reverse(t.prioridade), t.id))`: uma chave, nenhuma closure de comparação.

**Limitações conhecidas.** Dois processos gravando ao mesmo tempo: o último ganha. Um arquivo editado à mão com `proximo_id` menor que algum id existente geraria ids repetidos; validar isso ao carregar é uma boa extensão.

---

[Enunciado do projeto](../../projetos/03-tarefas/README.md) · [Índice do curso](../../README.md)

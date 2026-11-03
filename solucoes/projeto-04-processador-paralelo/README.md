# Solução — Projeto 4: `contapal`

Enunciado em [`projetos/04-processador-paralelo/README.md`](../../projetos/04-processador-paralelo/README.md).

```bash
cd solucoes/projeto-04-processador-paralelo
cargo test                                   # 24 unitários + 3 do executável
cargo run -q --release -- -n 5 --tempo src
cargo run -q --release -- -m canais -j 4 --arquivos src
```

Os arquivos têm os mesmos tipos, assinaturas e testes do [`inicial/`](../../projetos/04-processador-paralelo/inicial/); só os corpos mudam.

| Arquivo | O que tem |
|---|---|
| [`src/contagem.rs`](src/contagem.rs) | `palavras`, `contar`, `juntar`, `mais_frequentes` (etapa 1) |
| [`src/arquivos.rs`](src/arquivos.rs) | `ErroArquivo`, `percorrer` com `walkdir`, `ler`, `processar` (etapa 2) |
| [`src/relatorio.rs`](src/relatorio.rs) | `Relatorio::registrar`, `juntar`, `ordenado`, `escrever` (etapa 3) |
| [`src/processar.rs`](src/processar.rs) | `sequencial` (3), `com_canais` (4), `com_rayon` (5), e os testes que comparam os três |
| [`src/config.rs`](src/config.rs), [`src/main.rs`](src/main.rs), [`tests/cli.rs`](tests/cli.rs) | parser, executável e seus testes (prontos no inicial) |

## Decisões

**Um molde para os três modos.** Os três fazem `percorrer → and_then(processar) → registrar num Relatorio → ordenado`. O sequencial usa `Iterator::fold`; o rayon, `fold(Relatorio::default, ..)` + `reduce(Relatorio::default, Relatorio::juntar)`; o pipeline faz o `registrar` no agregador. Com a mesma forma, a diferença entre eles é só a concorrência, e um teste com `==` prova que dão o mesmo resultado.

**`processar(PathBuf) -> Result<(PathBuf, Contagem), ErroArquivo>`.** O resultado carrega o caminho junto, e por isso atravessa um canal sozinho (o agregador não precisa saber de onde veio). O erro também já tem o caminho dentro.

**`thread::scope` no modo `canais`.** As threads emprestam `raizes` e `&Mutex<Receiver>`, sem `Arc` nem `'static`, e o `scope` devolve o relatório. Se um worker entrar em pânico, o `scope` repassa o pânico ao terminar, em vez de o resultado sair incompleto sem aviso.

**Fila limitada só na entrada.** `sync_channel(64)` entre o primeiro estágio e os workers, porque listar é muito mais rápido que ler. Os resultados vão por `channel()` sem limite: o agregador é rápido, e um limite ali só arriscaria travar os workers.

**`loop` + `let ... else` nos workers.** O guarda do `lock()` morre no fim do `let`, antes de o arquivo ser lido. Com `while let`, ele ficaria preso o corpo inteiro: a variante foi medida (681 ms contra 150 ms com 10 workers, o tempo do sequencial) e dá o resultado certo, então nenhum teste de resultado a pega.

**`send` que falha encerra o laço.** Se quem recebe sumiu, não adianta continuar produzindo. Não há `unwrap` em `send`.

**Pool próprio no rayon.** `ThreadPoolBuilder::new().num_threads(n).build()` + `install`, para o `-j` e os testes escolherem o tamanho (o pool global só pode ser configurado uma vez por processo). `num_threads(0)` é o padrão do rayon (uma por núcleo), e a mesma convenção vale para os workers do `canais` (`threads_automatico`). O único `expect` de produção está no `build()`, comentado.

**`Vec` antes do `par_iter`.** Listar leva 15 ms no corpus de medição e divide bem. A variante com `par_bridge` (sem o `Vec`) mediu 167 ms contra 182 ms: um pouco melhor aqui, e fica como alternativa no enunciado.

**`walkdir::Error` → `into_io_error()`.** O `io::Error::from(e)` mantém o texto do walkdir, que repetiria o caminho na mensagem. A variante sem `io::Error` (laço de links) não acontece sem `follow_links`, mas tem uma mensagem própria em vez de `unwrap`.

**Comparar relatórios.** O `io::Error` não é `PartialEq`, então os testes comparam um `resumo` (a lista de arquivos, o total e os erros como texto) depois de `ordenado()`.

**Medições.** Números e método estão no enunciado (etapa 6). O resultado inesperado, o pipeline mais rápido que o rayon, foi mantido: é o que esta máquina mediu.

**Limitações conhecidas.** O arquivo inteiro vai para a memória; cada palavra vira uma `String` (é o maior custo); só UTF-8 é aceito; nada de `.gitignore`. Todas aparecem em "Para ir além".

---

[Enunciado do projeto](../../projetos/04-processador-paralelo/README.md) · [Índice do curso](../../README.md)

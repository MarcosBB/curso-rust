# Solução — Projeto 5: `chat`

Enunciado em [`projetos/05-chat-tcp/README.md`](../../projetos/05-chat-tcp/README.md).

```bash
cd solucoes/projeto-05-chat-tcp
cargo test                      # 15 unitários + 15 de integração
cargo run -q                    # servidor em 127.0.0.1:7878
nc 127.0.0.1 7878               # em outro terminal (ou: cargo run -q --bin cliente)
```

Os arquivos têm os mesmos tipos, assinaturas e testes do [`inicial/`](../../projetos/05-chat-tcp/inicial/); só os corpos mudam.

| Arquivo | O que tem |
|---|---|
| [`src/protocolo.rs`](src/protocolo.rs) | `interpretar`, `validar_apelido`, `ErroApelido` (etapa 1) |
| [`src/sala.rs`](src/sala.rs) | `Sala` com `broadcast`, `Mutex` da std e `AtomicU64` (etapa 2) |
| [`src/conexao.rs`](src/conexao.rs) | `atender`, `conversar` com `select!` de três ramos, `tratar`, `receber` (etapas 3–5) |
| [`src/servidor.rs`](src/servidor.rs) | `rodar`: `accept` + `select!` com o token, `TaskTracker` (etapas 3 e 5) |
| [`src/main.rs`](src/main.rs), [`tests/chat.rs`](tests/chat.rs) | executável com Ctrl+C e os testes de integração (prontos no inicial) |
| [`src/bin/cliente.rs`](src/bin/cliente.rs) | o cliente opcional (etapa 6) |

## Decisões

**`broadcast` no lugar de um `mpsc` por cliente.** O exemplo `chat.rs` do tokio guarda um `mpsc::UnboundedSender` por cliente num `HashMap` e manda cada mensagem num laço. Funciona, mas a fila de um cliente que não lê cresce sem limite. Com `broadcast` (capacidade 64), o cliente lento perde as mensagens mais antigas e recebe um aviso, e o servidor não cresce. A medição está no enunciado (etapa 4): 190 849 mensagens perdidas pelo lento, nenhum efeito nos outros.

**Filtro por autor na conexão.** Cada `Evento` leva o id de quem o gerou, e a conexão pula os seus. A sala não precisa saber quem é cada receptor, e um anúncio "para todos, menos eu" é um `send` só.

**Métodos síncronos na `Sala`.** O `std::sync::Mutex` é travado e solto dentro de cada método, então nenhum guarda pode atravessar um `.await`: a assinatura garante isso. É por isso que `quem` devolve um `Vec<String>` copiado, e não uma referência para dentro do conjunto.

**`Framed` + `LinesCodec::new_with_max_length(1024)`.** O limite protege a memória do servidor; o `Framed` serve para ler e escrever; e o codec tira o `\r` do telnet. A justificativa completa está na etapa 3 do enunciado.

**Linha longa encerra a conexão.** A ideia original era avisar e continuar. Mas o `Framed` para de ler depois de qualquer erro do codec (o próximo `next()` é `None`; está no código-fonte de `framed_impl.rs`), e o teste mostrou isso. Continuar exigiria chamar o codec à mão, sem o `Framed`.

**Limpeza fora do `?`.** `atender` chama `conversar`, guarda o resultado, e só depois libera o apelido e anuncia a saída. Toda saída de `conversar` (cliente fechou, `/sair`, erro de E/S, desligamento) passa pela limpeza. Uma alternativa seria um tipo com `Drop` que libera o apelido; aqui a função extra é mais simples de ler.

**Sem "saiu" no desligamento.** Com o anúncio, a ordem das linhas que cada cliente recebe no Ctrl+C dependia de qual task acordava primeiro, e um teste falhou com `*** bob saiu` chegando antes de `*** servidor encerrando`. Como todos estão saindo ao mesmo tempo, o anúncio era só ruído.

**`CancellationToken` + `TaskTracker`.** É o desenho do guia de *graceful shutdown* do Tokio. O token entra no `select!` do `accept` e no de cada conexão; o tracker espera as tasks sem acumular resultados (o `JoinSet` guardaria um por conexão até alguém buscar).

**`rodar` recebe o listener.** O `bind` fica no `main` (onde um erro vira mensagem e código 1) e nos testes (com `127.0.0.1:0`). Assim os testes rodam em paralelo, cada um com o seu servidor, sem disputar porta.

**Erro de `accept` não derruba o servidor.** Ele é registrado e o laço continua. Um caso real é faltar descritor de arquivo (muitas conexões abertas); um servidor de produção esperaria um pouco antes de tentar de novo.

**Testes sem esperar o silêncio.** Para conferir que algo não chegou, o teste manda `/quem` e exige que a próxima linha seja a resposta. Toda leitura tem tempo limite de 2 s. O único `sleep` (100 ms em `desligar_no_meio_de_uma_linha`) existe para o servidor ler a meia linha antes do Ctrl+C simulado; sem ele, o socket fecharia com bytes não lidos, o TCP mandaria RST, e o teste ficava instável.

**Cliente com `into_split` e `BufReader::lines`.** Para mostrar o outro caminho (sem `Framed`), e porque o `next_line` também é cancel safe. A precondição `, if teclado_aberto` evita mandar `/sair` em laço depois do Ctrl+D.

**Limitações conhecidas.** Uma sala só; sem mensagens privadas; um cliente que não lê pode atrasar o desligamento (o `send` dele fica esperando); sem limite de taxa nem de conexões. Todas estão em "Para ir além".

---

[Enunciado do projeto](../../projetos/05-chat-tcp/README.md) · [Índice do curso](../../README.md)

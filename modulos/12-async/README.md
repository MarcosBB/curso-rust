# Módulo 12 — Programação assíncrona

> Última revisão: 2026-11-02 (tokio 1.54)

O Módulo 11 resolveu a concorrência de quem **calcula**: uma thread por núcleo. Este módulo trata da concorrência de quem **espera**: milhares de conexões, timers e respostas de rede, cada uma parada a maior parte do tempo. Você vai entender o modelo de `async`/`await` do Rust (futures preguiçosas, sem runtime embutido), usar o tokio para criar tasks, timers, E/S e canais, e aprender os cuidados que só existem aqui: não bloquear o runtime, o `Send` através de `.await`, o cancelamento e o `Pin`.

**Pré-requisitos:** Módulos 06 (traits, `dyn` e *dyn compatibility*), 07 (closures e `Fn`/`FnMut`), 08 (dependências e *features*) e 11 (threads, `Send`/`Sync`, canais e `Mutex`).

**Objetivos.** Ao final do módulo, você vai saber:

- explicar o que é uma future, por que ela não faz nada sem `.await` e quando async compensa;
- criar tasks com `tokio::spawn` e `JoinSet`, combinar futures com `join!` e ler os erros de `Send` e `'static`;
- usar timers, E/S de rede e canais do tokio, sem bloquear o runtime;
- escolher entre `std::sync::Mutex` e `tokio::sync::Mutex`;
- usar `select!` sabendo o que é cancelado, e escrever código *cancellation safe*;
- fixar futures com `pin!` e `Box::pin`, e usar `async fn` em traits e async closures.

## Lições

| # | Lição | Assunto |
|---|---|---|
| 01 | [Futures e `async`/`await`](01-futures-e-async-await.md) | E/S x CPU, futures preguiçosas, sem runtime embutido, a trait `Future`, `asyncio` e coroutines do C++20 |
| 02 | [Tokio: runtime, `join!` e tasks](02-tokio-e-tasks.md) | `#[tokio::main]`, features, `join!`/`try_join!`, `tokio::spawn`, `Send` através de `.await`, `JoinSet`, threads x tasks |
| 03 | [Timers e o pecado de bloquear o runtime](03-tempo-e-bloqueio.md) | `sleep`, `timeout`, `interval`, bloqueio medido, `spawn_blocking` |
| 04 | [E/S assíncrona: um servidor de eco](04-io-assincrona.md) | `TcpListener`, uma task por conexão, `AsyncReadExt`/`AsyncWriteExt`, testes na porta 0, `tokio::fs` |
| 05 | [Canais e estado compartilhado](05-canais-e-estado.md) | `mpsc`, `oneshot`, `broadcast`, `watch`, task dona do recurso, `std::sync::Mutex` x `tokio::sync::Mutex` |
| 06 | [`select!` e cancelamento](06-select-e-cancelamento.md) | cancelar = descartar, `biased;`, *cancellation safety*, parada limpa |
| 07 | [`Pin` e `Unpin`: o essencial](07-pin-e-unpin.md) | futures auto-referentes, `pin!`, `Box::pin`, uma future escrita à mão |
| 08 | [`async fn` em traits e async closures](08-async-em-traits-e-closures.md) | `Send` na future do método, `dyn` e `async-trait`, `AsyncFn`/`AsyncFnMut` |

## Exercícios

[`exercicios.md`](exercicios.md): 5 exercícios. Soluções em [`solucoes/12-async`](../../solucoes/12-async/).

## Código

Os exemplos completos estão em [`codigo/`](codigo/). Todos dependem do tokio (`features = ["full"]`):

- [`tasks-e-join`](codigo/tasks-e-join/): futures preguiçosas, sequencial x `join!`, `spawn`, `JoinSet` e 100 000 tasks (lições 01 e 02);
- [`tempo-e-bloqueio`](codigo/tempo-e-bloqueio/): `timeout`, `interval`, o efeito de bloquear o runtime e `spawn_blocking` (lição 03);
- [`eco-tcp`](codigo/eco-tcp/): servidor de eco com uma task por conexão, com testes de 50 clientes simultâneos (lição 04);
- [`canais-e-estado`](codigo/canais-e-estado/): os quatro canais, um contador dono do estado e os dois `Mutex` (lição 05);
- [`select-e-cancelamento`](codigo/select-e-cancelamento/): `select!`, a perda de mensagens e a correção, `pin!`, parada com `oneshot` e uma future à mão (lições 06 e 07);
- [`traits-e-closures`](codigo/traits-e-closures/): `async fn` em traits (genérica, `Send` e `dyn`) e async closures (lição 08).

Para rodar:

```bash
cd codigo/eco-tcp
cargo run          # e, em outro terminal: nc 127.0.0.1 7878
cargo test
```

Como no Módulo 11, a ordem das linhas de saída pode mudar de uma execução para outra, e os tempos medidos variam um pouco com a máquina. As lições avisam quando isso acontece.

## Referências principais

- [Tutorial do Tokio](https://tokio.rs/tokio/tutorial) (a referência prática principal deste módulo)
- [Documentação do tokio](https://docs.rs/tokio/latest/tokio/)
- [The Rust Programming Language, cap. 17: Fundamentals of Asynchronous Programming](https://doc.rust-lang.org/stable/book/ch17-00-async-await.html) (usa a crate didática `trpl`)
- [Async Book](https://rust-lang.github.io/async-book/) (em reescrita: use a Parte 1 e os capítulos de referência sobre *pinning*, cancelamento e *structured concurrency*)
- Documentação de [`std::future`](https://doc.rust-lang.org/std/future/index.html), [`std::pin`](https://doc.rust-lang.org/std/pin/index.html) e [`std::task`](https://doc.rust-lang.org/std/task/index.html)
- Blog de [Alice Ryhl](https://ryhl.io/), mantenedora do tokio (*What is blocking?*, *Actors with Tokio*)

O próximo passo é o [Projeto 5](../../projetos/05-chat-tcp/README.md): um chat TCP assíncrono, com muitas conexões ao mesmo tempo, mensagens repassadas a todos os participantes e um desligamento limpo. Ele junta quase tudo deste módulo: tasks por conexão, E/S por linhas, canais e `select!`.

---

[← Anterior: Projeto 4](../../projetos/04-processador-paralelo/README.md) · [Índice do curso](../../README.md) · [Próximo: Projeto 5 →](../../projetos/05-chat-tcp/README.md)

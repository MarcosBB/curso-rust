# Módulo 11 — Concorrência

As regras que você aprendeu até aqui (um dono, "muitos `&` ou um `&mut`", `Rc` e `RefCell` para os casos especiais) foram pensadas para uma thread. Este módulo mostra que elas valem, quase sem mudança, para várias: o mesmo compilador que recusa dois `&mut` recusa duas threads escrevendo no mesmo dado. Você vai criar threads, mandar mensagens entre elas, compartilhar estado com `Mutex` e atômicos, fazer estado global sem `static mut` e paralelizar coleções com o `rayon`.

**Pré-requisitos:** Módulos 02 e 03 (posse, empréstimos, `'static`), 07 (closures e `move`) e 10 (`Rc`, `RefCell`, guardas e o escopo de temporários na edition 2024).

**Objetivos.** Ao final do módulo, você vai saber:

- criar threads, esperar por elas e emprestar dados locais com `thread::scope`;
- ler os erros de `Send` e `Sync` e escolher o tipo certo para várias threads;
- trocar mensagens com canais `mpsc` e montar um pipeline;
- compartilhar estado com `Arc<Mutex<T>>`, `RwLock` e atômicos, e evitar deadlocks;
- fazer estado global seguro com `LazyLock` e `OnceLock`;
- paralelizar iteradores com `rayon` e medir se compensou.

## Lições

| # | Lição | Assunto |
|---|---|---|
| 01 | [Threads: `spawn`, `join`, `move` e `scope`](01-threads-e-scope.md) | `JoinHandle`, E0373, `'static`, panic numa thread, `thread::scope`, `chunks_mut` |
| 02 | [`Send` e `Sync`](02-send-e-sync.md) | data race em C++, auto traits, `Rc` → `Arc`, `RefCell` → `Mutex`, `Box<dyn Error + Send + Sync>`, GIL |
| 03 | [Canais](03-canais.md) | `mpsc`, vários produtores, o `drop(tx)`, `sync_channel`, `recv_timeout`, pipeline |
| 04 | [`Mutex` e `RwLock`](04-mutex-e-rwlock.md) | `Arc<Mutex<T>>`, guardas, `let _ =`, *poisoning*, `RwLock`, deadlocks e o `match` na edition 2024 |
| 05 | [Atômicos](05-atomicos.md) | `fetch_add`, `compare_exchange`, flag de parada, `Ordering` na prática |
| 06 | [Estado global seguro](06-estado-global.md) | `static` e `Sync`, `LazyLock`, `OnceLock`, `static mut` na edition 2024 |
| 07 | [Paralelismo de dados com `rayon`](07-rayon.md) | `par_iter`, `fold`/`reduce`, medições, quando não compensa |

## Exercícios

[`exercicios.md`](exercicios.md): 5 exercícios. Soluções em [`solucoes/11-concorrencia`](../../solucoes/11-concorrencia/).

## Código

Os exemplos completos estão em [`codigo/`](codigo/):

- [`threads-e-scope`](codigo/threads-e-scope/): `spawn`, `join`, panic numa thread, `Builder`, soma e escala com `thread::scope` (lição 01);
- [`canais`](codigo/canais/): vários produtores, `recv_timeout` e um pipeline de três estágios (lição 03);
- [`estado-compartilhado`](codigo/estado-compartilhado/): contagem com `Arc<Mutex>`, `RwLock`, *poisoning*, contador atômico, flag de parada e `Release`/`Acquire` (lições 04 e 05);
- [`estado-global`](codigo/estado-global/): atômico e `Mutex` em `static`, `LazyLock` e `OnceLock` (lição 06);
- [`rayon-basico`](codigo/rayon-basico/): primos em paralelo, frequências com `fold`/`reduce` e `par_sort`, com medições (lição 07).

Para rodar:

```bash
cd codigo/canais
cargo run
cargo test
```

Programas concorrentes não têm saída determinística: a ordem das linhas pode mudar de uma execução para outra. As lições avisam quando isso acontece; os resultados (somas, contagens) não mudam.

## Referências principais

- [The Rust Programming Language, cap. 16: Fearless Concurrency](https://doc.rust-lang.org/stable/book/ch16-00-concurrency.html)
- Documentação de [`std::thread`](https://doc.rust-lang.org/std/thread/index.html) e [`std::sync`](https://doc.rust-lang.org/std/sync/index.html)
- [Rust Atomics and Locks](https://marabos.nl/atomics/), de Mara Bos (gratuito online)
- [Comprehensive Rust: Concurrency](https://google.github.io/comprehensive-rust/concurrency/welcome.html)
- [Rustonomicon: Concurrency](https://doc.rust-lang.org/nomicon/concurrency.html)
- [Documentação do `rayon`](https://docs.rs/rayon/latest/rayon/)

O próximo passo é o [Projeto 4](../../projetos/04-processador-paralelo/README.md), que usa threads, canais e `rayon` para processar muitos arquivos em paralelo. Depois, o Módulo 12 trata do outro tipo de concorrência: milhares de tarefas esperando E/S, com `async`/`await`.

---

[← Anterior: Módulo 10](../10-smart-pointers/README.md) · [Índice do curso](../../README.md) · [Próximo: Projeto 4 →](../../projetos/04-processador-paralelo/README.md)

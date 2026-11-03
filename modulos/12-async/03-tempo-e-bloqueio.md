# 03 — Timers e o pecado de bloquear o runtime

**Objetivos.** Ao final, você vai saber:

- esperar, pôr prazo e repetir com `sleep`, `timeout` e `interval`;
- reconhecer código que **bloqueia** o runtime, e medir o estrago;
- mandar trabalho pesado ou síncrono para `spawn_blocking` (ou para o `rayon`).

## `sleep`, `timeout` e `interval`

Os timers do tokio ficam em `tokio::time`, e todos são futures:

```rust
use tokio::time::{Duration, Instant, interval, sleep, timeout};

let r = timeout(Duration::from_millis(100), sleep(Duration::from_secs(5))).await;
println!("{r:?}");
let r = timeout(Duration::from_millis(100), async { 42 }).await;
println!("{r:?}");

let inicio = Instant::now();
let mut tick = interval(Duration::from_millis(100));
for _ in 0..3 {
    tick.tick().await;
    println!("tick em {} ms", inicio.elapsed().as_millis());
}
```

```text
Err(Elapsed(()))
Ok(42)
tick em 1 ms
tick em 102 ms
tick em 203 ms
```

- `sleep(d)` termina depois de `d`, sem ocupar a thread.
- `timeout(d, fut)` devolve `Ok(valor)` se `fut` terminou a tempo, ou `Err(Elapsed)`. Quando o prazo vence, `fut` é **descartada**, ou seja, cancelada (lição 06). Qualquer future pode ganhar um prazo assim, sem que ela precise saber disso.
- `interval(d)` dispara a cada `d`. O **primeiro** `tick()` termina na hora, o que surpreende muita gente. Diferente de `sleep` em laço, o `interval` desconta o tempo gasto entre os ticks, então o ritmo não escorrega.

Em Python: `asyncio.sleep`, `asyncio.wait_for` (ou `asyncio.timeout`, desde o 3.11) e um laço com `sleep`.

## Bloquear o runtime

Lembre da lição 02: uma task só larga a thread num `.await`. Um código que **bloqueia** (dorme, espera E/S síncrona ou calcula por muito tempo) sem passar por um `.await` prende a thread inteira, e todas as outras tasks que dependiam dela ficam paradas.

O exemplo mais simples é usar o `sleep` errado:

```rust
async fn trabalho_ruim(id: u32) {
    std::thread::sleep(Duration::from_millis(100)); // bloqueia a thread inteira
    println!("ruim {id} terminou");
}

async fn trabalho_bom(id: u32) {
    tokio::time::sleep(Duration::from_millis(100)).await; // cede a vez enquanto espera
    println!("bom {id} terminou");
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let inicio = Instant::now();
    tokio::join!(trabalho_ruim(1), trabalho_ruim(2), trabalho_ruim(3));
    println!("std::thread::sleep: {:?}", inicio.elapsed());

    let inicio = Instant::now();
    tokio::join!(trabalho_bom(1), trabalho_bom(2), trabalho_bom(3));
    println!("tokio::time::sleep: {:?}", inicio.elapsed());
}
```

```text
ruim 1 terminou
ruim 2 terminou
ruim 3 terminou
std::thread::sleep: 312.864875ms
bom 2 terminou
bom 3 terminou
bom 1 terminou
tokio::time::sleep: 102.001583ms
```

As três versões "ruins" rodaram uma depois da outra, porque cada uma segurou a única thread por 100 ms. As "boas" esperaram juntas. (A ordem em que as boas terminaram varia; o que importa é o tempo.)

O compilador não acusa nada: `std::thread::sleep` dentro de `async fn` é perfeitamente válido. É uma das armadilhas mais caras do async, porque num servidor ela aparece como "às vezes tudo fica lento", sem erro nenhum. O runtime multi-thread disfarça o problema com mais threads, mas basta bloquear tantas quantos núcleos você tem para travar tudo.

O que bloqueia, na prática:

| Bloqueia (evite numa task) | Versão assíncrona ou alternativa |
|---|---|
| `std::thread::sleep` | `tokio::time::sleep(...).await` |
| `std::fs::read_to_string` e afins | `tokio::fs::read_to_string(...).await` |
| `std::net::TcpStream` | `tokio::net::TcpStream` |
| `rx.recv()` de um `std::sync::mpsc` | canais de `tokio::sync` (lição 05) |
| cálculo longo (comprimir, hash de senha, parse enorme) | `spawn_blocking` ou `rayon` |
| biblioteca síncrona (um driver de banco antigo, por exemplo) | `spawn_blocking` |

Uma regra prática citada pela comunidade: entre dois `.await`, uma task não deveria passar mais que algo entre 10 e 100 microssegundos sem ceder a vez.

O Python tem exatamente o mesmo problema: `time.sleep` dentro de uma corrotina trava o *event loop*, e o remédio é `await asyncio.sleep` ou `await asyncio.to_thread(funcao)`.

## `spawn_blocking`

`tokio::task::spawn_blocking` roda uma closure **síncrona** num conjunto separado de threads, reservado para bloqueio, e devolve um `JoinHandle` que você pode esperar com `.await`:

```rust
fn primos_ate(n: u64) -> usize {
    (2..n)
        .filter(|&k| (2..).take_while(|d| d * d <= k).all(|d| k % d != 0))
        .count()
}

async fn primos_em_segundo_plano(n: u64) -> usize {
    tokio::task::spawn_blocking(move || primos_ate(n))
        .await
        .expect("a thread de bloqueio não deve dar panic")
}
```

Com o runtime de uma thread só, o cálculo pesado não impede o resto de andar:

```rust
let inicio = Instant::now();
let pesado = tokio::spawn(primos_em_segundo_plano(3_000_000));
sleep(Duration::from_millis(20)).await;
println!("o runtime continua livre: {:?}", inicio.elapsed());
let n = pesado.await.unwrap();
println!("{n} primos em {:?}", inicio.elapsed());
```

```text
o runtime continua livre: 23.231208ms
216816 primos em 301.42ms
```

Sem o `spawn_blocking`, a linha "continua livre" só sairia depois dos 300 ms de cálculo.

A closure precisa ser `Send + 'static`, como a de `thread::spawn`. O conjunto de threads de bloqueio cresce sob demanda (até 512, por padrão), então ele é pensado para **esperas** síncronas. Para **cálculo** em grande quantidade, o melhor é o `rayon` do Módulo 11, que usa uma thread por núcleo. Os dois se combinam com um canal `oneshot` (lição 05): a task manda o trabalho para o rayon e espera a resposta sem bloquear.

> [!TIP]
> Na dúvida sobre se algo bloqueia, meça. Uma task que demora para responder quando outra está ocupada é o sintoma. O `tokio-console` (uma ferramenta de diagnóstico do projeto tokio) mostra quanto tempo cada task passa sem ceder a vez.

Código completo em [`codigo/tempo-e-bloqueio`](codigo/tempo-e-bloqueio/src/main.rs).

## Armadilhas comuns

- **`std::thread::sleep` numa task.** Compila, roda e serializa tudo. Use `tokio::time::sleep(...).await`.
- **Ler arquivos ou falar com a rede pela std dentro de async.** Mesma coisa: use `tokio::fs`, `tokio::net`, ou `spawn_blocking`.
- **Cálculo pesado direto na task.** Não há `.await` no meio, então a thread fica presa. `spawn_blocking` (pouco cálculo, ocasional) ou `rayon` (muito cálculo).
- **Esquecer que o primeiro tick de `interval` é imediato.** Se você queria esperar antes da primeira execução, chame `tick().await` uma vez antes do laço, ou use `interval_at`.

## Resumo

- `sleep`, `timeout` e `interval` são futures; `timeout` cancela a future que estourou o prazo.
- Uma task só cede a thread no `.await`. Bloquear dentro dela trava as outras.
- `std::thread::sleep`, E/S da std e cálculo longo são as formas comuns de bloquear.
- `spawn_blocking` leva código síncrono para threads separadas; para cálculo em massa, `rayon`.

## Exercícios

Veja o exercício 3 em [`exercicios.md`](exercicios.md).

## Leitura complementar

- [Documentação de `tokio::time`](https://docs.rs/tokio/latest/tokio/time/index.html)
- [Documentação de `spawn_blocking`](https://docs.rs/tokio/latest/tokio/task/fn.spawn_blocking.html)
- [Async Book: IO and issues with blocking](https://rust-lang.github.io/async-book/part-guide/io.html)
- [Alice Ryhl, *Async: What is blocking?*](https://ryhl.io/blog/async-what-is-blocking/) (de uma das mantenedoras do tokio)

---

[← Lição anterior](02-tokio-e-tasks.md) · [Índice do módulo](README.md) · [Próxima lição →](04-io-assincrona.md)

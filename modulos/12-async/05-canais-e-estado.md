# 05 — Canais e estado compartilhado em async

**Objetivos.** Ao final, você vai saber:

- usar os canais do tokio (`mpsc`, `oneshot`, `broadcast`, `watch`) e escolher entre eles;
- montar uma task dona de um recurso, que responde a pedidos por mensagens;
- escolher entre `std::sync::Mutex` e `tokio::sync::Mutex`, e entender o erro do guarda que atravessa um `.await`.

## Por que outros canais?

O `std::sync::mpsc` do Módulo 11 tem um `recv()` que **bloqueia** a thread. Numa task, isso é o pecado da lição 03. Os canais de `tokio::sync` têm a mesma ideia, mas o `recv` é uma future: a task dorme sem prender a thread, e o waker a acorda quando chega uma mensagem.

```rust
use tokio::sync::mpsc;

let (tx, mut rx) = mpsc::channel::<String>(2);
for id in 1..=3 {
    let tx = tx.clone();
    tokio::spawn(async move {
        for i in 1..=2 {
            tx.send(format!("task {id}: msg {i}")).await.unwrap();
        }
    });
}
drop(tx); // sem isto, o while abaixo nunca termina
while let Some(msg) = rx.recv().await {
    println!("{msg}");
}
println!("todos os produtores terminaram");
```

```text
task 1: msg 1
task 1: msg 2
task 3: msg 1
task 2: msg 1
task 2: msg 2
task 3: msg 2
todos os produtores terminaram
```

Quase tudo é igual ao `mpsc` da std: vários `Sender` clonados, um `Receiver`, ordem preservada por produtor, e o `recv` devolve `None` quando **todos** os `Sender` sumiram. O `drop(tx)` continua não sendo enfeite.

As diferenças:

- **`mpsc::channel(n)` é sempre limitado.** O `send` é `async` e espera quando a fila está cheia: é o *backpressure* do `sync_channel` da std, só que sem bloquear a thread. Para uma fila sem limite, existe `mpsc::unbounded_channel()`, com `send` síncrono (e o risco de a memória crescer sem parar).
- **`recv` é `&mut self`**, por isso o `let mut rx`.
- **Funciona dos dois lados.** Uma thread comum pode mandar com `tx.blocking_send(...)` e receber com `rx.blocking_recv()`. Isso liga o mundo síncrono ao assíncrono (por exemplo, uma thread do `rayon` devolvendo resultado para uma task).

## Os quatro canais

| Canal | Produtores → consumidores | Para quê |
|---|---|---|
| `mpsc` | muitos → um | fila de trabalho, pedidos para uma task |
| `oneshot` | um → um, **uma** mensagem | devolver uma resposta |
| `broadcast` | muitos → muitos, **cada** consumidor recebe tudo | avisos para todos os interessados |
| `watch` | muitos → muitos, só o **valor mais recente** | estado ou configuração que muda |

```rust
let (tx, mut rx1) = broadcast::channel::<&str>(16);
let mut rx2 = tx.subscribe();
tx.send("aviso geral").unwrap();
println!("rx1: {}", rx1.recv().await.unwrap());
println!("rx2: {}", rx2.recv().await.unwrap());
```

```text
rx1: aviso geral
rx2: aviso geral
```

No `broadcast`, cada receptor (criado com `subscribe()`) recebe uma cópia de cada mensagem, então o tipo precisa ser `Clone`. A capacidade (16) é por receptor: quem fica para trás mais que isso perde as mensagens antigas e recebe `Err(RecvError::Lagged(n))` dizendo quantas perdeu.

No `watch`, o receptor não vê uma fila, só o último valor. `changed().await` espera a próxima mudança:

```rust
let (tx, mut rx) = watch::channel("iniciando");
let observador = tokio::spawn(async move {
    while rx.changed().await.is_ok() {
        println!("estado agora: {}", *rx.borrow_and_update());
    }
});
for estado in ["rodando", "parando"] {
    sleep(Duration::from_millis(10)).await;
    tx.send(estado).unwrap();
}
drop(tx);
observador.await.unwrap();
```

```text
estado agora: rodando
estado agora: parando
```

`changed()` devolve `Err` quando o transmissor é descartado, o que encerra o laço.

## Uma task dona do recurso

Um padrão muito usado com tokio: em vez de compartilhar um recurso com `Arc<Mutex<_>>`, uma task é **dona** dele, e as outras mandam pedidos por um `mpsc`. Quando o pedido precisa de resposta, ele leva junto um `oneshot::Sender`:

```rust
enum Comando {
    Somar(u64),
    Total(oneshot::Sender<u64>),
}

fn iniciar_contador() -> mpsc::Sender<Comando> {
    let (tx, mut rx) = mpsc::channel(16);
    tokio::spawn(async move {
        let mut total = 0;
        while let Some(cmd) = rx.recv().await {
            match cmd {
                Comando::Somar(n) => total += n,
                Comando::Total(resp) => {
                    let _ = resp.send(total); // quem perguntou pode ter desistido
                }
            }
        }
    });
    tx
}

async fn perguntar_total(contador: &mpsc::Sender<Comando>) -> u64 {
    let (resp_tx, resp_rx) = oneshot::channel();
    contador.send(Comando::Total(resp_tx)).await.unwrap();
    resp_rx.await.unwrap()
}
```

```text
total: 42
```

(depois de mandar `Somar(10)`, `Somar(20)` e `Somar(12)`). O `total` é uma variável comum da task: ninguém mais tem acesso a ela, então não há lock nenhum. A task processa um comando por vez, na ordem de chegada. É o modelo de *atores*, e funciona bem quando o recurso é algo que não pode ser compartilhado de verdade (uma conexão, um arquivo aberto) ou quando a lógica em volta dele é complicada.

O `oneshot::Sender::send` não é `async` (só cabe uma mensagem, não há fila para encher) e devolve `Err` se o receptor já foi descartado. O `oneshot::Receiver` é ele mesmo uma future: espera-se a resposta com `resp_rx.await`.

## `Mutex`: qual dos dois?

O Módulo 11 usou `Arc<Mutex<T>>` para estado compartilhado entre threads. Em async, ele continua sendo a ferramenta certa na maioria dos casos, desde que o guarda **não atravesse um `.await`**. Veja o que acontece quando atravessa:

```rust
// ❌ NÃO COMPILA
use std::sync::{Arc, Mutex};
use tokio::time::{Duration, sleep};

async fn registrar(log: Arc<Mutex<Vec<String>>>, msg: String) {
    let mut v = log.lock().unwrap();
    sleep(Duration::from_millis(10)).await;
    v.push(msg);
}

#[tokio::main]
async fn main() {
    let log = Arc::new(Mutex::new(Vec::new()));
    let h = tokio::spawn(registrar(log.clone(), "oi".into()));
    h.await.unwrap();
    println!("{:?}", log.lock().unwrap());
}
```

```text
error: future cannot be sent between threads safely
   --> src/main.rs:13:26
    |
 13 |     let h = tokio::spawn(registrar(log.clone(), "oi".into()));
    |                          ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ future returned by `registrar` is not `Send`
    |
    = help: within `impl Future<Output = ()>`, the trait `Send` is not implemented for `std::sync::MutexGuard<'_, Vec<String>>`
note: future is not `Send` as this value is used across an await
   --> src/main.rs:6:38
    |
  5 |     let mut v = log.lock().unwrap();
    |         ----- has type `std::sync::MutexGuard<'_, Vec<String>>` which is not `Send`
  6 |     sleep(Duration::from_millis(10)).await;
    |                                      ^^^^^ await occurs here, with `mut v` maybe used later
...
```

O `MutexGuard` da std não é `Send` (Módulo 11, lição 02): ele precisa ser destravado na mesma thread que travou. Uma task que guarda o guarda num `.await` pode acordar em outra thread. O compilador recusa, e ainda bem, porque o problema seria pior que esse: enquanto a task dorme segurando o lock, qualquer outra task que tente `lock()` **bloqueia a thread** dela (é um lock da std), e se as duas estiverem na mesma thread, é deadlock.

### O `drop` que não basta

A correção óbvia seria soltar o guarda antes do `.await`:

```rust
// ❌ NÃO COMPILA (ainda)
async fn registrar(log: Arc<Mutex<Vec<String>>>, msg: String) {
    let mut v = log.lock().unwrap();
    v.push(msg);
    drop(v);
    sleep(Duration::from_millis(10)).await;
}
```

```text
error: future cannot be sent between threads safely
...
    = help: within `impl Future<Output = ()>`, the trait `Send` is not implemented for `std::sync::MutexGuard<'_, Vec<String>>`
note: future is not `Send` as this value is used across an await
   --> src/main.rs:8:38
    |
  5 |     let mut v = log.lock().unwrap();
    |         ----- has type `std::sync::MutexGuard<'_, Vec<String>>` which is not `Send`
...
  8 |     sleep(Duration::from_millis(10)).await;
    |                                      ^^^^^ await occurs here, with `mut v` maybe used later
```

Surpresa: o erro é o mesmo. A análise que decide o que entra no estado da future é conservadora com variáveis que foram **emprestadas**. O `v.push(msg)` pega um `&mut` de `v` (via `DerefMut`), e a partir daí o compilador não confia mais no `drop` e trata `v` como viva até o fim do escopo. (Um guarda que nunca foi usado passaria com o `drop`, mas um guarda que nunca foi usado não serve para nada.) É uma limitação conhecida do compilador. O jeito que funciona é um bloco que termina antes do `.await`:

```rust
// ✅
async fn contar_visita(visitas: Arc<Mutex<HashMap<String, u32>>>, pagina: String) {
    {
        let mut mapa = visitas.lock().unwrap();
        *mapa.entry(pagina).or_insert(0) += 1;
    } // o guarda morre aqui
    sleep(Duration::from_millis(1)).await;
}
```

Ou, mais simples quando cabe numa linha, um guarda temporário que morre no fim da instrução: `visitas.lock().unwrap().push(x);`. Melhor ainda é esconder o `Mutex` atrás de métodos síncronos (`fn contar(&self, pagina: &str)`): um método sem `async` não tem como segurar o lock através de um `.await`.

### Quando o guarda precisa atravessar o `.await`

Às vezes é exatamente isso que você quer: segurar o recurso enquanto espera (escrever num arquivo sem que ninguém intercale, por exemplo). Para isso existe `tokio::sync::Mutex`, cujo `lock()` é `async` e cujo guarda é `Send`:

```rust
async fn registrar_devagar(log: Arc<tokio::sync::Mutex<Vec<String>>>, msg: String) {
    let mut v = log.lock().await;
    sleep(Duration::from_millis(5)).await; // simula uma escrita lenta
    v.push(msg);
}
```

Quem espera pelo lock dorme sem prender a thread. O custo é ser mais lento que o da std, e o efeito colateral é que **tudo** o que depende dele espera a vez, inclusive durante a E/S lenta. A recomendação do tutorial do tokio:

| Situação | Use |
|---|---|
| seção crítica curta e síncrona (atualizar um mapa, um contador) | `std::sync::Mutex` (solto antes do `.await`) |
| guarda precisa atravessar `.await` | `tokio::sync::Mutex` |
| recurso de E/S compartilhado, lógica complexa | uma task dona do recurso + canais |
| contador ou flag simples | atômicos (Módulo 11) |

O mesmo raciocínio vale para `RwLock` (`std::sync::RwLock` x `tokio::sync::RwLock`). O tokio tem também `Semaphore` (limitar a N operações simultâneas, por exemplo 10 conexões ao banco) e `Notify` (acordar tasks sem mandar dados).

Código completo, com testes, em [`codigo/canais-e-estado`](codigo/canais-e-estado/src/main.rs).

## Comparando com Python

O `asyncio` tem `asyncio.Queue` (equivale ao `mpsc`, com `maxsize`), `asyncio.Lock` (equivale ao `tokio::sync::Mutex`, só funciona entre corrotinas) e `asyncio.Event`. Como o `asyncio` roda numa thread só, um `threading.Lock` segurado através de um `await` não dá erro: ele só trava o *event loop* inteiro se outra corrotina tentar pegá-lo. Em Rust, o mesmo erro vira um erro de compilação.

## Armadilhas comuns

- **`std::sync::mpsc::Receiver::recv` numa task.** Bloqueia a thread. Use `tokio::sync::mpsc`.
- **Esquecer o `drop(tx)`.** O `while let Some(..) = rx.recv().await` nunca termina, igual ao Módulo 11.
- **`drop(guarda)` antes do `.await` não resolve o erro de `Send`** se o guarda foi usado (e ele sempre é). Use um bloco `{ }`.
- **Trocar tudo por `tokio::sync::Mutex` para "fazer compilar".** Compila, mas pode serializar o programa: o lock fica preso durante a espera. Primeiro tente soltar o guarda antes do `.await`.
- **`broadcast` lento.** Um receptor que não acompanha recebe `Lagged`. Trate esse erro, não faça `unwrap`.

## Resumo

- Em tasks, use os canais de `tokio::sync`: o `recv` (e o `send` do `mpsc`) são futures.
- `mpsc` para filas, `oneshot` para respostas, `broadcast` para todos receberem tudo, `watch` para o valor mais recente.
- Uma task dona do recurso, com `mpsc` de pedidos e `oneshot` de respostas, dispensa locks.
- `std::sync::Mutex` é o padrão, desde que o guarda morra antes do `.await` (num bloco, não com `drop`).
- `tokio::sync::Mutex` só quando o guarda precisa atravessar o `.await`.

## Exercícios

Veja o exercício 2 em [`exercicios.md`](exercicios.md).

## Leitura complementar

- [Tutorial do Tokio: Shared state](https://tokio.rs/tokio/tutorial/shared-state) e [Channels](https://tokio.rs/tokio/tutorial/channels)
- [Documentação de `tokio::sync`](https://docs.rs/tokio/latest/tokio/sync/index.html) (inclui a discussão sobre qual `Mutex` usar)
- [Async Book: Channels, locking, and synchronization](https://rust-lang.github.io/async-book/part-guide/sync.html)
- [Alice Ryhl, *Actors with Tokio*](https://ryhl.io/blog/actors-with-tokio/)

---

[← Lição anterior](04-io-assincrona.md) · [Índice do módulo](README.md) · [Próxima lição →](06-select-e-cancelamento.md)

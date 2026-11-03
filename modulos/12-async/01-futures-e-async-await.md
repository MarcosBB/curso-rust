# 01 — Futures e `async`/`await`

**Objetivos.** Ao final, você vai saber:

- quando async ajuda (esperar E/S) e quando não ajuda (calcular);
- o que uma `async fn` devolve, e por que nada acontece até o `.await`;
- por que Rust não traz um runtime embutido, e o que é a trait `Future`;
- como isso se compara com o `asyncio` do Python e as coroutines do C++20.

## O problema: esperar sai caro com threads

No Módulo 11, cada trabalho concorrente ganhou uma thread. Isso funciona bem quando o trabalho é **calcular**: cada thread ocupa um núcleo. Mas um servidor passa a maior parte do tempo **esperando**: o próximo pacote da rede, a resposta do banco, o disco. Uma thread parada esperando ainda custa uma pilha (alguns MB de memória virtual) e uma entrada no escalonador do sistema operacional.

Com 10 000 conexões abertas, uma thread por conexão começa a pesar. Nesta máquina, criar 10 000 threads que só dormem 100 ms nem chegou a funcionar:

```text
thread 'main' (214949) panicked at .../std/src/thread/functions.rs:131:29:
failed to spawn thread: Os { code: 35, kind: WouldBlock, message: "Resource temporarily unavailable" }
```

O sistema recusou novas threads antes de chegar a 10 000 (com 2 000, funcionou). Já 100 000 *tasks* assíncronas fazendo a mesma coisa terminaram em 133 ms (lição 02).

A ideia do async: em vez de bloquear a thread enquanto espera, a função **devolve o controle** e diz "me acorde quando os dados chegarem". Uma única thread consegue então alternar entre milhares de esperas. Isso é concorrência (muitas coisas em andamento) sem precisar de paralelismo (muitas coisas executando no mesmo instante).

| Gargalo | Exemplo | Ferramenta |
|---|---|---|
| CPU | comprimir arquivos, contar primos | threads, `rayon` (Módulo 11) |
| espera por E/S | servidor web, cliente HTTP, chat | async |

## `async fn` devolve uma future

```rust
async fn saudacao() -> String {
    String::from("oi")
}
```

Apesar da assinatura dizer `-> String`, chamar `saudacao()` **não** devolve uma `String`. Devolve um valor de um tipo anônimo que implementa `Future<Output = String>`: uma *future* (futuro), a promessa de uma `String` quando a computação terminar. Para obter o valor, você usa `.await`:

```rust
let s = saudacao().await;
```

O `.await` pergunta à future se ela já terminou. Se não terminou (está esperando a rede, por exemplo), a função atual **também** devolve o controle a quem a chamou, e assim por diante, até o *runtime* (o executor). Quando os dados chegam, o runtime retoma a função exatamente do ponto do `.await`.

Por isso, `.await` só pode aparecer dentro de algo `async`:

```rust
// ❌ NÃO COMPILA
async fn dobro(x: u32) -> u32 {
    x * 2
}

fn main() {
    let y = dobro(21).await;
    println!("{y}");
}
```

```text
error[E0728]: `await` is only allowed inside `async` functions and blocks
 --> src/main.rs:6:23
  |
5 | fn main() {
  | --------- this is not `async`
6 |     let y = dobro(21).await;
  |                       ^^^^^ only allowed inside `async` functions and blocks
```

Uma função comum não tem como "devolver o controle e ser retomada depois". Essa capacidade é justamente o que o `async` acrescenta. Além de `async fn`, existem os blocos `async { ... }`, que criam uma future no meio de uma função:

```rust
let f = async {
    let s = saudacao().await;
    s.len()
}; // f: impl Future<Output = usize>, ainda não executou nada
```

## Futures são preguiçosas

Uma future não faz nada até alguém dar `.await` nela (ou entregá-la a um runtime). Criar a future só prepara o trabalho:

```rust
async fn diga(texto: &str) {
    println!("{texto}");
}

#[tokio::main]
async fn main() {
    diga("um");
    let f = diga("dois");
    println!("antes do await");
    f.await;
}
```

```text
warning: unused implementer of `Future` that must be used
 --> src/main.rs:7:5
  |
7 |     diga("um");
  |     ^^^^^^^^^^
  |
  = note: futures do nothing unless you `.await` or poll them
  = note: `#[warn(unused_must_use)]` (part of `#[warn(unused)]`) on by default
```

```text
antes do await
dois
```

O `"um"` nunca aparece: a future foi criada e descartada sem rodar. E o `"dois"` só sai depois do `"antes do await"`, porque o corpo de `diga` só executa no `.await`. (O `#[tokio::main]` é o runtime; ele aparece já, já.)

Esse é o oposto do `Promise` do JavaScript, que começa a executar assim que é criado. É também um dos erros mais comuns de quem começa: esquecer um `.await` e a operação simplesmente não acontecer. O aviso do compilador está lá para isso; não o ignore.

O Python se comporta igual, mas só avisa em tempo de execução:

```python
async def diga(texto):
    print(texto)

async def main():
    diga("um")
    f = diga("dois")
    print("antes do await")
    await f

asyncio.run(main())
```

```text
lazy.py:7: RuntimeWarning: coroutine 'diga' was never awaited
  diga("um")
antes do await
dois
```

## Sem runtime embutido

No Python, o `asyncio` vem com a linguagem: `asyncio.run(main())` cria o *event loop* (o laço que decide qual corrotina roda) e pronto. Em Rust, a linguagem só define a sintaxe (`async`, `.await`) e a trait `Future`. **Quem executa as futures é uma biblioteca**. Sem ela, nem o `main` pode ser `async`:

```rust
// ❌ NÃO COMPILA
async fn saudacao() -> String {
    String::from("oi")
}

async fn main() {
    let s = saudacao().await;
    println!("{s}");
}
```

```text
error[E0752]: `main` function is not allowed to be `async`
 --> src/main.rs:5:1
  |
5 | async fn main() {
  | ^^^^^^^^^^^^^^^ `main` function is not allowed to be `async`
```

Alguém precisa ficar chamando as futures até elas terminarem, e o `main` é onde esse alguém começa. Por que não embutir um? Porque não existe um runtime certo para todo mundo: um servidor quer várias threads e E/S de rede, um microcontrolador não tem nem sistema operacional (lá se usa `embassy`), e uma aplicação pequena pode preferir algo mínimo (`smol`). Deixar o runtime numa crate permite que cada um escolha, sem pagar pelo que não usa.

Na prática, o padrão do ecossistema é o **tokio**, e é ele que o curso usa daqui em diante. No `Cargo.toml`:

```toml
[dependencies]
tokio = { version = "1.54", features = ["full"] }
```

> [!NOTE]
> O capítulo 17 do Rust Book usa uma crate didática, `trpl`, que embrulha o tokio e a crate `futures` para esconder detalhes. Se você estiver lendo o livro em paralelo, `trpl::run(fut)` faz o papel do `#[tokio::main]`. Aqui vamos direto ao tokio, que é o que você vai encontrar em projetos reais.

## O que é uma future, por dentro

A trait é pequena:

```rust
pub trait Future {
    type Output;
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output>;
}

pub enum Poll<T> {
    Ready(T),
    Pending,
}
```

O runtime chama `poll`. A future responde `Ready(valor)` se terminou, ou `Pending` se ainda está esperando. No caso `Pending`, ela guarda o *waker* que veio no `Context` e o entrega a quem vai produzir o dado (o sistema operacional, um timer, um canal). Quando o dado chega, o waker avisa o runtime: "pode chamar `poll` de novo nesta". O runtime não fica perguntando em laço; ele só volta a quem foi acordado.

```mermaid
stateDiagram-v2
    [*] --> Criada: chamar a async fn
    Criada --> Executando: runtime chama poll
    Executando --> Pendente: precisa esperar (Poll::Pending)
    Pendente --> Executando: waker avisa, poll de novo
    Executando --> Pronta: terminou (Poll::Ready)
    Pronta --> [*]
```

O `Pin` na assinatura tem sua própria lição (07), junto com uma future escrita à mão. Você quase nunca implementa `poll`: o compilador faz isso para cada `async fn`.

### Uma `async fn` vira uma máquina de estados

O compilador transforma cada `async fn` num `enum` com um estado por `.await`: "ainda não comecei", "parado no primeiro `.await`, com estas variáveis locais", "parado no segundo", "terminei". As variáveis que precisam sobreviver a um `.await` viram campos desse enum. Cada `poll` avança de um estado para o próximo até encontrar algo que ainda não está pronto.

Duas consequências práticas:

- **Não há alocação escondida.** A future é um valor comum, com tamanho conhecido na compilação. Ela pode ficar na pilha, dentro de outra future, ou numa `Box` se você quiser.
- **O que está vivo num `.await` faz parte da future.** Se uma variável `Rc` atravessa um `.await`, a future contém um `Rc` e herda as restrições dele (por exemplo, não ser `Send`). A lição 02 volta a isso.

## Comparando com C++20

O C++20 também separou as coisas: a linguagem ganhou `co_await`, `co_return` e `co_yield`, mas a biblioteca padrão não trouxe nenhum tipo de *task* nem executor. Você usa uma biblioteca (cppcoro, Asio, folly) ou escreve o seu tipo de retorno com `promise_type`. O C++26 acrescenta `std::execution` (senders/receivers), um modelo de execução padronizado.

As diferenças práticas:

- Em C++, o estado da coroutine (o *frame*) costuma ir para o **heap**, e o compilador pode otimizar essa alocação em alguns casos. Em Rust, a future é um valor de tamanho fixo, sem alocação obrigatória.
- Em C++, uma referência a uma variável local que sobrevive ao `co_await` pode ficar pendurada sem aviso. Em Rust, o *borrow checker* analisa a future como qualquer outra struct: o empréstimo precisa valer enquanto a future existir.

| | Python `asyncio` | C++20 coroutines | Rust |
|---|---|---|---|
| sintaxe | `async def`, `await` | `co_await`, `co_return` | `async fn`, `.await` |
| runtime | embutido (`asyncio.run`) | biblioteca | biblioteca (tokio) |
| preguiçosa? | sim (corrotina), `create_task` agenda | depende do tipo de retorno | sim |
| alocação | objeto no heap | frame no heap (geralmente) | valor de tamanho fixo |
| esquecer o `await` | `RuntimeWarning` | depende da biblioteca | aviso de compilação |

## Armadilhas comuns

- **Esquecer o `.await`.** A operação simplesmente não acontece. Leia o aviso `unused implementer of Future`.
- **Achar que async deixa cálculo mais rápido.** Async não cria paralelismo para trabalho de CPU; ele só aproveita o tempo de espera. Para calcular, use threads ou `rayon` (e a lição 03 mostra como misturar os dois).
- **`async fn main` sem runtime.** É o E0752. Ponha `#[tokio::main]` em cima.

## Resumo

- Async serve para muitas esperas por E/S; para cálculo, continue com threads.
- Uma `async fn` devolve uma future. Nada executa até o `.await`.
- `.await` só existe dentro de `async fn` ou de bloco `async`.
- A linguagem não traz runtime; o tokio é o padrão do ecossistema.
- Por dentro, uma future é uma máquina de estados com `poll`, `Ready`/`Pending` e um waker.

## Exercícios

Os exercícios começam na lição 02. Veja [`exercicios.md`](exercicios.md).

## Leitura complementar

- [The Rust Programming Language, cap. 17: Fundamentals of Asynchronous Programming](https://doc.rust-lang.org/stable/book/ch17-00-async-await.html)
- [Async Book: Async and await](https://rust-lang.github.io/async-book/part-guide/async-await.html)
- [Documentação de `std::future::Future`](https://doc.rust-lang.org/std/future/trait.Future.html)
- [Tutorial do Tokio: Hello Tokio](https://tokio.rs/tokio/tutorial/hello-tokio)

---

[← Índice do módulo](README.md) · [Próxima lição →](02-tokio-e-tasks.md)

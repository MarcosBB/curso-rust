# 07 — `Pin` e `Unpin`: o essencial

**Objetivos.** Ao final, você vai saber:

- por que algumas futures não podem mudar de lugar na memória depois de começar;
- o que `Pin` e `Unpin` garantem, sem entrar nos detalhes de implementação;
- fixar uma future com `pin!` (na pilha) ou `Box::pin` (no heap), e quando cada um aparece;
- escrever uma future à mão, para ver `poll`, `Pin` e o waker funcionando.

## O problema: futures que apontam para si mesmas

Lembre que uma `async fn` vira uma máquina de estados, e as variáveis vivas num `.await` viram campos dela (lição 01). Agora imagine:

```rust
async fn ler_cabecalho(socket: &mut TcpStream) -> io::Result<()> {
    let mut buf = [0u8; 64];
    let parte = &mut buf[..16];       // referência para `buf`
    socket.read(parte).await?;        // `buf` e `parte` atravessam o .await
    Ok(())
}
```

No `.await`, a future guarda os dois: o array `buf` e a referência `parte`, que aponta **para dentro da própria future**. É uma estrutura auto-referente. Se alguém mover a future para outro endereço depois desse ponto (com um `let f2 = f;`, ou ao crescer um `Vec` de futures), `buf` vai junto, mas `parte` continua apontando para o endereço antigo. Seria uma referência pendurada.

Em código síncrono, o *borrow checker* nunca deixa isso acontecer: uma struct não pode emprestar de si mesma. Em async, o compilador gera essas estruturas o tempo todo, e precisava de outra garantia: **depois que a future começa a rodar, ela não se move mais**. Essa garantia é o `Pin`.

## `Pin` e `Unpin`

`Pin<P>` envolve um ponteiro (`Pin<&mut T>`, `Pin<Box<T>>`) e promete que o valor apontado **não vai ser movido** até ser destruído. Por isso a assinatura de `poll` recebe `self: Pin<&mut Self>`: o runtime só pode fazer poll numa future que está fixada.

A maior parte dos tipos não se importa em ser movida: `i32`, `String`, `Vec<T>`, `TcpStream`, um `oneshot::Receiver`. Eles implementam `Unpin`, uma *auto trait* (como `Send`) que diz "pode me mover mesmo pinado, eu não tenho auto-referências". Para tipos `Unpin`, o `Pin` não restringe nada.

Quem **não** é `Unpin` são justamente as futures geradas por `async fn` e blocos `async`, porque o compilador não sabe se elas vão conter auto-referências.

Na prática, você só esbarra no `Pin` em dois casos:

1. quando precisa fazer poll na **mesma** future várias vezes através de uma referência (`&mut fut` no `select!` em laço);
2. quando precisa guardar futures de tipos diferentes juntas (`Box<dyn Future>`), o que exige `Pin<Box<...>>`.

## Caso 1: `&mut fut` exige `Unpin`

O exemplo da lição 06, sem o `pin!`:

```rust
// ❌ NÃO COMPILA
use tokio::time::{Duration, sleep};

#[tokio::main]
async fn main() {
    let lenta = async {
        sleep(Duration::from_millis(300)).await;
        "pronto"
    };
    let mut tick = tokio::time::interval(Duration::from_millis(100));
    loop {
        tokio::select! {
            v = &mut lenta => { println!("{v}"); break; }
            _ = tick.tick() => println!("ainda esperando..."),
        }
    }
}
```

```text
error[E0277]: `&mut {async block@src/main.rs:5:17: 5:22}` is not a future
  --> src/main.rs:11:9
   |
11 | /         tokio::select! {
12 | |             v = &mut lenta => { println!("{v}"); break; }
13 | |             _ = tick.tick() => println!("ainda esperando..."),
14 | |         }
   | |         ^
   | |         |
   | |_________`&mut {async block@src/main.rs:5:17: 5:22}` is not a future
   |           required by a bound introduced by this call
   |
   = help: the trait `Unpin` is not implemented for `{async block@src/main.rs:5:17: 5:22}`
   = note: &mut {async block@src/main.rs:5:17: 5:22} must be a future or must implement `IntoFuture` to be awaited
   = note: required for `&mut {async block@src/main.rs:5:17: 5:22}` to implement `Future`
...
```

O compilador diz que `&mut F` só é uma future se `F: Unpin`. Faz sentido: com um `&mut` simples, nada impediria você de, entre duas voltas do laço, mover a future com `std::mem::swap`, depois de ela já ter começado. A correção é fixá-la:

```rust
// ✅
use std::pin::pin;

let mut lenta = pin!(async {
    sleep(Duration::from_millis(300)).await;
    "pronto"
});
```

`std::pin::pin!` (estável desde a 1.68) move a future para uma variável na pilha que ninguém mais consegue acessar diretamente, e devolve um `Pin<&mut F>`. Um `Pin<&mut F>` é `Unpin` (é só um ponteiro), então `&mut lenta` passa a ser uma future. O tutorial do tokio usa a macro `tokio::pin!`, que faz a mesma coisa.

## Caso 2: `Box::pin` e futures de tipos diferentes

Cada `async fn` e cada bloco `async` tem um tipo próprio e anônimo. Para guardar futures diferentes no mesmo `Vec`, ou devolver uma ou outra conforme uma condição, você precisa de um *trait object*: `Box<dyn Future<Output = T>>`. E, como o poll exige `Pin`, o tipo usado é `Pin<Box<dyn Future<Output = T>>>`:

```rust
use std::future::Future;
use std::pin::Pin;

fn escolher(rapido: bool) -> Pin<Box<dyn Future<Output = &'static str> + Send>> {
    if rapido {
        Box::pin(async { "na hora" })
    } else {
        Box::pin(async {
            sleep(Duration::from_millis(50)).await;
            "depois"
        })
    }
}
```

`Box::pin` põe a future no heap e a fixa lá. Mover a `Box` só move o ponteiro; a future fica no mesmo endereço. Esse tipo vai voltar na lição 08, como a forma de usar `dyn` com métodos assíncronos.

| | `pin!(fut)` | `Box::pin(fut)` |
|---|---|---|
| onde fica | pilha (na função atual) | heap |
| custo | nenhum | uma alocação |
| pode sair da função? | não | sim (é um valor comum) |
| uso típico | `&mut fut` no `select!` | `dyn Future`, guardar em struct ou `Vec` |

## Uma future escrita à mão

Para ver as peças juntas, aqui está uma future que responde `Pending` algumas vezes antes de ficar pronta:

```rust
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

struct Contagem {
    restante: u32,
}

impl Future for Contagem {
    type Output = &'static str;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.restante == 0 {
            Poll::Ready("fim")
        } else {
            println!("poll: faltam {}", self.restante);
            self.restante -= 1;
            cx.waker().wake_by_ref(); // "me chame de novo"
            Poll::Pending
        }
    }
}

#[tokio::main]
async fn main() {
    println!("{}", Contagem { restante: 3 }.await);
}
```

```text
poll: faltam 3
poll: faltam 2
poll: faltam 1
fim
```

- `Contagem` só tem um `u32`, então é `Unpin`, e `self.restante -= 1` funciona direto através do `Pin<&mut Self>`. Numa future que não é `Unpin`, mexer nos campos exige *pin projection* (a crate `pin-project-lite` faz isso com segurança), assunto de quem escreve bibliotecas.
- **Antes de devolver `Pending`, a future precisa garantir que alguém vai chamar o waker**, senão o runtime nunca mais faz poll nela e ela fica parada para sempre. Aqui ela mesma chama `wake_by_ref()` na hora, o que equivale a dizer "tente de novo já". Uma future de verdade entrega o waker ao timer, ao socket ou ao canal, que o chamam quando o dado chega.

Isso é tudo o que o compilador gera para você em cada `async fn`. Escrever futures à mão é raro fora de bibliotecas.

Código completo em [`codigo/select-e-cancelamento`](codigo/select-e-cancelamento/src/main.rs).

## Comparando com C++

No C++20, o *frame* de uma coroutine fica, em geral, no heap, alocado pelo compilador, e o objeto que você manipula é um *handle* para ele. Como o frame nunca se move, o problema da auto-referência não aparece, ao custo de uma alocação por coroutine (que o compilador às vezes consegue eliminar). Rust escolheu o caminho oposto: futures como valores comuns, sem alocação obrigatória, e o `Pin` para proibir o movimento só depois que ele passa a ser perigoso.

## Armadilhas comuns

- **`&mut fut` no `select!` sem `pin!`.** O erro fala em `Unpin`. Ponha `let mut fut = pin!(...)` antes do laço.
- **Tentar entender `Pin` pela implementação antes de precisar.** Para usar async, basta saber: futures de `async` não são `Unpin`; `pin!` fixa na pilha, `Box::pin` no heap.
- **Esquecer o waker numa future manual.** Devolver `Pending` sem providenciar um `wake` trava aquela future para sempre.

## Resumo

- Futures geradas por `async` podem conter referências para si mesmas, e por isso não podem se mover depois de começar.
- `Pin<P>` garante que o valor apontado não se move; `Unpin` marca os tipos para os quais isso não importa (a maioria).
- `pin!` fixa na pilha (para `&mut fut` em laço); `Box::pin` fixa no heap (para `dyn Future`).
- `poll` devolve `Ready` ou `Pending`; com `Pending`, alguém precisa chamar o waker.

## Exercícios

Veja o exercício 5 em [`exercicios.md`](exercicios.md), que usa `Pin<Box<dyn Future>>`.

## Leitura complementar

- [Documentação de `std::pin`](https://doc.rust-lang.org/std/pin/index.html)
- [Async Book: Pinning](https://rust-lang.github.io/async-book/part-reference/pinning.html)
- [The Rust Programming Language, 17.5 A Closer Look at the Traits for Async](https://doc.rust-lang.org/stable/book/ch17-05-traits-for-async.html)
- [Tutorial do Tokio: Async in depth](https://tokio.rs/tokio/tutorial/async)

---

[← Lição anterior](06-select-e-cancelamento.md) · [Índice do módulo](README.md) · [Próxima lição →](08-async-em-traits-e-closures.md)

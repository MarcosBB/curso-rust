# 06 — `select!` e cancelamento

**Objetivos.** Ao final, você vai saber:

- esperar a **primeira** de várias futures com `select!`, e o que acontece com as outras;
- o que cancelar significa em Rust (descartar a future) e onde isso acontece sem você pedir;
- reconhecer código que **não** é *cancellation safe* e reescrevê-lo;
- parar uma task de forma limpa com um sinal.

## `join!` espera todas, `select!` espera a primeira

```rust
async fn tarefa(nome: &'static str, ms: u64) -> &'static str {
    let _a = Avisa(nome); // imprime "drop: nome" quando for descartado
    println!("{nome} começou");
    sleep(Duration::from_millis(ms)).await;
    println!("{nome} terminou");
    nome
}

tokio::select! {
    v = tarefa("rapida", 50) => println!("venceu: {v}"),
    v = tarefa("lenta", 500) => println!("venceu: {v}"),
}
println!("depois do select");
```

```text
rapida começou
lenta começou
rapida terminou
drop: rapida
drop: lenta
venceu: rapida
depois do select
```

Cada braço tem a forma `padrão = future => código`. O `select!` faz *poll* em todas as futures; quando a primeira termina, ele executa o código do braço dela e **descarta as outras**. A `lenta` nunca imprimiu "terminou": no meio do `sleep`, ela foi destruída, e o `Drop` das suas variáveis locais rodou (o `drop: lenta`).

Duas regras do `select!` do tokio:

- **A ordem de verificação é aleatória** a cada execução, para nenhum braço ganhar sempre por estar em primeiro. Rodando de novo, a saída começou com `lenta começou`. Se você quer a ordem do código, ponha `biased;` como primeira linha do `select!` (exemplo mais abaixo).
- **Se o padrão não casar**, o braço é desativado e o `select!` continua com os outros. Com `Some(msg) = rx.recv()`, um `None` desativa o braço. Se todos forem desativados, executa-se o braço `else => ...` (sem ele, é *panic*).

Os casos de uso típicos são "isto ou um prazo", "isto ou um sinal de parada" e "o que chegar primeiro entre duas fontes":

```rust
loop {
    tokio::select! {
        Some(msg) = rx.recv() => println!("mensagem: {msg}"),
        _ = tick.tick() => println!("ainda vivo"),
        _ = tokio::signal::ctrl_c() => break,
    }
}
```

## Cancelar é descartar

Em Rust, uma future só avança quando alguém faz *poll* nela. Então **cancelar é simplesmente parar de fazer poll e descartá-la**. Não há exceção, nem sinal, nem *flag*: a future é destruída onde estava, no último `.await` em que parou, e os destrutores das variáveis vivas rodam (fechando arquivos, soltando locks, devolvendo conexões ao pool).

Isso acontece em mais lugares do que parece:

| Onde | O que é cancelado |
|---|---|
| `select!` | os braços que não venceram |
| `timeout(d, fut)` | `fut`, se o prazo vencer |
| `JoinHandle::abort()` | a task |
| `JoinSet` descartado | todas as tasks que ainda rodam |
| `main` retornando | todas as tasks |
| uma future qualquer descartada sem `.await` | ela mesma |

Compare com o Python: lá, `task.cancel()` faz um `CancelledError` ser **lançado** no ponto do `await`, e a corrotina pode capturá-lo, limpar e até ignorar o cancelamento. Em Rust, a future não é avisada e não pode recusar; a limpeza fica nos `Drop`. O resultado é mais simples e mais previsível, mas impõe um cuidado novo.

## Cancellation safety

Uma future é *cancellation safe* (segura para cancelar) se descartá-la no meio, num `.await`, **não perde nada**: dá para criar outra igual e continuar de onde parou. O problema aparece no `select!` dentro de um laço, porque a cada volta os braços que perderam são descartados e recriados.

Veja uma função que lê duas mensagens e devolve o par:

```rust
/// Lê duas mensagens e devolve o par. NÃO é cancellation safe: se for
/// cancelada entre os dois `recv`, a primeira mensagem se perde.
async fn ler_par(rx: &mut mpsc::Receiver<u32>) -> Option<(u32, u32)> {
    let a = rx.recv().await?;
    let b = rx.recv().await?;
    Some((a, b))
}

async fn pares_com_perda(mut rx: mpsc::Receiver<u32>, tick_ms: u64) -> Vec<(u32, u32)> {
    let mut tick = tokio::time::interval(Duration::from_millis(tick_ms));
    let mut pares = Vec::new();
    loop {
        tokio::select! {
            par = ler_par(&mut rx) => match par {
                Some(p) => pares.push(p),
                None => return pares,
            },
            _ = tick.tick() => {}
        }
    }
}
```

Um produtor manda os números de 1 a 6, um a cada 30 ms, e o `tick` dispara a cada 50 ms:

```text
com perda: [(3, 4)]
```

Quatro dos seis números sumiram. O `ler_par` recebe o `1`, fica esperando o segundo `recv`, o `tick` dispara, o `select!` descarta o `ler_par`... e o `1`, guardado na variável local `a`, vai junto. O código compila e não dá erro: o dado simplesmente desaparece.

A correção é deixar no `select!` só operações que são seguras para cancelar, e guardar o estado parcial **fora** do `select!`, numa variável que sobrevive às voltas do laço:

```rust
async fn pares_sem_perda(mut rx: mpsc::Receiver<u32>, tick_ms: u64) -> Vec<(u32, u32)> {
    let mut tick = tokio::time::interval(Duration::from_millis(tick_ms));
    let mut pares = Vec::new();
    let mut pendente: Option<u32> = None;
    loop {
        tokio::select! {
            msg = rx.recv() => match (msg, pendente.take()) {
                (Some(b), Some(a)) => pares.push((a, b)),
                (Some(a), None) => pendente = Some(a),
                (None, _) => return pares,
            },
            _ = tick.tick() => {}
        }
    }
}
```

```text
sem perda: [(1, 2), (3, 4), (5, 6)]
```

O `rx.recv()` é seguro para cancelar: ou ele devolveu uma mensagem (e o braço dele venceu), ou não tirou nada do canal. Quando outro braço vence, nada se perde.

Outra correção comum: tirar o trabalho lento de dentro da future disputada e pô-lo no **corpo** do braço. O corpo (depois do `=>`) nunca é cancelado pelo `select!`: ele só começa depois que o braço venceu.

```rust
tokio::select! {
    Some(n) = rx.recv() => processar(n).await, // o corpo roda até o fim
    _ = tick.tick() => {}
}
```

### O que é seguro, segundo o tokio

A documentação do `select!` lista:

| Seguro para cancelar | Não é seguro |
|---|---|
| `mpsc::Receiver::recv`, `broadcast::Receiver::recv`, `watch::Receiver::changed` | `AsyncReadExt::read_exact`, `read_to_end`, `read_to_string` (perdem o que já leram) |
| `TcpListener::accept` | `AsyncWriteExt::write_all` (não dá para saber quanto foi escrito) |
| `AsyncReadExt::read`, `AsyncWriteExt::write` | `Mutex::lock`, `Semaphore::acquire`, `Notify::notified` (perdem o lugar na fila) |
| `sleep`, `interval.tick()` | qualquer `async fn` sua que guarde estado entre dois `.await` |

A última linha é a mais importante: uma `async fn` sua não é segura para cancelar, a menos que você verifique. Documente as que forem.

## Reaproveitando a mesma future entre voltas

Às vezes você quer o contrário: uma operação longa que **não** deve ser recriada a cada volta, só acompanhada. Para isso, crie a future antes do laço e passe `&mut fut` ao `select!`. Mas o `select!` exige que a future esteja *pinned* (fixada):

```rust
let mut lenta = pin!(async {
    sleep(Duration::from_millis(300)).await;
    "pronto"
});
let mut tick = tokio::time::interval(Duration::from_millis(100));
loop {
    tokio::select! {
        v = &mut lenta => { println!("{v}"); break; }
        _ = tick.tick() => println!("ainda esperando..."),
    }
}
```

```text
ainda esperando...
ainda esperando...
ainda esperando...
pronto
```

(Com o primeiro tick imediato, o número de "ainda esperando" pode ser 3 ou 4, conforme o empate em 300 ms.) Sem o `pin!`, o erro fala em `Unpin`. O que é isso e por que é preciso está na próxima lição.

## Parando uma task de forma limpa

`abort()` cancela uma task em qualquer `.await`, no meio do que ela estiver fazendo. Para uma parada **limpa** (terminar o item atual, gravar o estado), mande um sinal e deixe a própria task decidir onde parar:

```rust
async fn trabalhar_ate_parar(mut parar: oneshot::Receiver<()>) -> u32 {
    let mut ciclos = 0;
    loop {
        tokio::select! {
            biased; // confere a parada primeiro, sempre
            _ = &mut parar => return ciclos,
            _ = sleep(Duration::from_millis(10)) => ciclos += 1,
        }
    }
}

let (parar_tx, parar_rx) = oneshot::channel();
let trabalhador = tokio::spawn(trabalhar_ate_parar(parar_rx));
sleep(Duration::from_millis(55)).await;
parar_tx.send(()).unwrap();
println!("ciclos completos: {}", trabalhador.await.unwrap());
```

```text
ciclos completos: 4
```

O `biased;` garante que o sinal de parada é conferido antes do trabalho a cada volta. Aqui o `oneshot::Receiver` não precisou de `pin!` porque ele é `Unpin` (lição 07). Para avisar **várias** tasks, use um `watch` (lição 05), ou o `CancellationToken` da crate `tokio-util`, feito exatamente para isso.

Código completo, com testes, em [`codigo/select-e-cancelamento`](codigo/select-e-cancelamento/src/main.rs).

## Armadilhas comuns

- **`async fn` com estado entre dois `.await` dentro de `select!` em laço.** Perde dados sem erro. Deixe no `select!` só operações seguras e guarde o estado fora.
- **`read_exact` ou `write_all` disputando com um timer.** Mesmo problema. Use `read` (e acumule num buffer fora do laço), ou ponha a operação inteira no corpo do braço.
- **Achar que o braço perdedor "continua em segundo plano".** Ele foi destruído. Se precisa continuar, crie-o como task (`tokio::spawn`) e faça `select!` no `JoinHandle`.
- **`select!` sem `else` com todos os braços desativados.** É *panic*. Inclua um `else => break`.
- **Depender da ordem dos braços.** É aleatória, a menos que você use `biased;`.

## Resumo

- `select!` espera a primeira future; as outras são descartadas.
- Cancelar é descartar a future no último `.await`; os `Drop` rodam, mas o estado local se perde.
- `select!`, `timeout`, `abort`, `JoinSet` e o fim do `main` cancelam futures.
- *Cancellation safe* = pode ser descartada e recriada sem perder dados. `recv` é; `read_exact`, `write_all` e as suas `async fn` com estado, em geral, não.
- Para uma parada limpa, mande um sinal (`oneshot`, `watch`, `CancellationToken`) e use `biased;`.

## Exercícios

Veja os exercícios 3 e 4 em [`exercicios.md`](exercicios.md).

## Leitura complementar

- [Tutorial do Tokio: Select](https://tokio.rs/tokio/tutorial/select)
- [Documentação de `tokio::select!`](https://docs.rs/tokio/latest/tokio/macro.select.html), em especial a seção *Cancellation safety*
- [Async Book: Cancellation and cancellation safety](https://rust-lang.github.io/async-book/part-reference/cancellation.html)
- [Tokio: Graceful Shutdown](https://tokio.rs/tokio/topics/shutdown)

---

[← Lição anterior](05-canais-e-estado.md) · [Índice do módulo](README.md) · [Próxima lição →](07-pin-e-unpin.md)

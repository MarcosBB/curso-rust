# Soluções — Módulo 12

Enunciados em [`modulos/12-async/exercicios.md`](../../modulos/12-async/exercicios.md).

Cada solução é um projeto Cargo com os testes do enunciado. Para conferir:

```bash
cd nome-do-projeto
cargo test
```

Os testes usam `#[tokio::test(start_paused = true)]`: o relógio do tokio é virtual e avança sozinho quando todas as tasks estão esperando um timer. Por isso os tempos medidos são exatos (`100ms`, não `100.3ms`) e os testes terminam na hora, mesmo os que "esperam" segundos.

## 1. Buscar tudo ao mesmo tempo

Projeto: [`ex1-buscar-todos/`](ex1-buscar-todos/src/lib.rs).

Com o código inicial, o teste de tempo falha com a duração real da versão sequencial, 50 × 100 ms:

```text
---- tests::roda_ao_mesmo_tempo stdout ----

thread 'tests::roda_ao_mesmo_tempo' (264484) panicked at src/lib.rs:42:9:
5s
```

A solução cria todas as tasks primeiro e só depois espera:

```rust
pub async fn buscar_todos(ids: &[u32]) -> Vec<String> {
    let handles: Vec<_> = ids.iter().map(|&id| tokio::spawn(buscar(id))).collect();
    let mut saida = Vec::with_capacity(handles.len());
    for h in handles {
        saida.push(h.await.expect("buscar não dá panic"));
    }
    saida
}
```

O `collect` é o que importa: ele percorre o iterador inteiro, criando as 50 tasks, antes do primeiro `.await`. As tasks já estão rodando enquanto o `for` espera a primeira. Esperar os handles na ordem dos ids mantém a ordem do resultado: se o handle 3 terminar antes do 1, o resultado dele fica guardado no `JoinHandle` até a vez dele. Um `JoinSet` daria a ordem de término, e você teria de reordenar depois.

Escrever `for &id in ids { saida.push(tokio::spawn(buscar(id)).await...) }` cairia no mesmo problema do código inicial: cria uma task e espera por ela antes de criar a próxima.

Uma alternativa sem tasks é rodar todas as futures dentro da mesma task, com `futures::future::join_all` (crate `futures`). Para E/S, o resultado é o mesmo; as tasks têm a vantagem de poder usar várias threads.

`buscar_dois_grupos` é um `tokio::join!(buscar_todos(a), buscar_todos(b))`: dois grupos conhecidos ao escrever o código, rodando ao mesmo tempo. O teste confere que o total fica abaixo de 150 ms.

## 2. O guarda que atravessa o `.await`

Projeto: [`ex2-salvar-todos/`](ex2-salvar-todos/src/lib.rs).

O primeiro erro é o da lição 05:

```text
error: future cannot be sent between threads safely
   --> src/lib.rs:19:35
    |
 19 |         handles.push(tokio::spawn(salvar(log.clone(), nome)));
    |                                   ^^^^^^^^^^^^^^^^^^^^^^^^^ future returned by `salvar` is not `Send`
    |
    = help: within `impl Future<Output = ()>`, the trait `Send` is not implemented for `std::sync::MutexGuard<'_, Vec<String>>`
note: future is not `Send` as this value is used across an await
   --> src/lib.rs:8:38
    |
  6 |     let mut v = log.lock().unwrap();
    |         ----- has type `std::sync::MutexGuard<'_, Vec<String>>` which is not `Send`
  7 |     v.push(format!("começando {item}"));
  8 |     sleep(Duration::from_millis(10)).await; // simula E/S
    |                                      ^^^^^ await occurs here, with `mut v` maybe used later
```

O guarda `v` está vivo durante o `sleep`. A correção é travar duas vezes, cada uma por uma instrução só, com o guarda temporário morrendo no `;`:

```rust
async fn salvar(log: Arc<Mutex<Vec<String>>>, item: String) {
    log.lock().unwrap().push(format!("começando {item}"));
    sleep(Duration::from_millis(10)).await; // simula E/S
    log.lock().unwrap().push(format!("salvo {item}"));
}
```

Um bloco `{ let mut v = log.lock().unwrap(); v.push(...); }` antes do `sleep` também funciona. Um `drop(v)` antes do `sleep`, não (lição 05).

Com isso, o `cargo build` passa, mas o `cargo test` mostra o segundo erro, porque os testes chamam `tokio::spawn(salvar_todos(...))`:

```text
error: future cannot be sent between threads safely
   --> src/lib.rs:34:32
    |
 34 |         let log = tokio::spawn(salvar_todos(itens)).await.unwrap();
    |                                ^^^^^^^^^^^^^^^^^^^ future returned by `salvar_todos` is not `Send`
    |
    = help: within `impl Future<Output = Vec<String>>`, the trait `Send` is not implemented for `Rc<String>`
note: future is not `Send` as this value is used across an await
   --> src/lib.rs:21:11
    |
 14 |     let prefixo = Rc::new(String::from("item-"));
    |         ------- has type `Rc<String>` which is not `Send`
...
 21 |         h.await.unwrap();
    |           ^^^^^ await occurs here, with `prefixo` maybe used later
```

O `Rc` nunca vai para outra task (só é usado no `format!`), mas continua vivo, no escopo de `salvar_todos`, durante o `h.await`. A correção é `Arc` (ou nem usar ponteiro: um `&str` constante bastaria). Repare que o erro só aparece quando alguém tenta mandar a future de `salvar_todos` para uma task: a própria função compila. Uma biblioteca que quer ser usada com tokio deveria ter testes assim.

### Por que não o `tokio::sync::Mutex`?

Trocar o `Mutex` e manter o guarda durante o `sleep` compila, mas o teste `nao_serializa` falha:

```text
thread 'tests::nao_serializa' (257873) panicked at src/lib.rs:48:9:
100ms
```

Dez itens levaram 100 ms em vez de 10 ms: cada task segura o lock durante a "E/S" e as outras esperam na fila. O código ficou correto e sequencial. A regra da lição 05 vale: solte o guarda antes do `.await`, e só use o `Mutex` do tokio quando segurar o lock durante a espera for exatamente o que você quer.

## 3. A resposta mais rápida

Projeto: [`ex3-mais-rapida/`](ex3-mais-rapida/src/lib.rs).

```rust
pub async fn mais_rapida(servidores: Vec<Servidor>, prazo: Duration) -> Result<String, Erro> {
    let mut set = JoinSet::new();
    for s in servidores {
        set.spawn(consultar(s));
    }
    let corrida = async {
        while let Some(r) = set.join_next().await {
            if let Ok(Ok(resposta)) = r {
                return Ok(resposta);
            }
        }
        Err(Erro::TodosFalharam)
    };
    // Ao sair daqui, o `set` é descartado e cancela as consultas restantes.
    timeout(prazo, corrida).await.unwrap_or(Err(Erro::Prazo))
}
```

- O `JoinSet` entrega os resultados por ordem de término, que é o que "a primeira resposta" pede.
- `Ok(Ok(resposta))`: o externo é o `JoinError` (a task deu *panic* ou foi cancelada), o interno é a resposta do servidor. Os dois tipos de falha são ignorados, e o laço continua.
- Quando `join_next` devolve `None`, todas as tasks terminaram sem nenhuma resposta boa: `TodosFalharam`. Isso cobre também a lista vazia.
- O `timeout` envolve a corrida inteira. Se o prazo vence, a future `corrida` é cancelada (descartada) e `timeout` devolve `Err(Elapsed)`, que o `unwrap_or` converte em `Erro::Prazo`.
- Em todos os casos, ao retornar, o `set` sai de escopo e o `Drop` dele aborta as consultas que ainda estavam rodando. Nada fica pendurado em segundo plano.

O bloco `async` empresta `set` mutavelmente; isso funciona porque ele é esperado (pelo `timeout`) ali mesmo, dentro da função. Uma variante com `tokio::select!` entre o laço e um `sleep(prazo)` também resolve, e é equivalente.

Os testes conferem os tempos exatos: o caso "ganha a mais rápida" termina em `100ms` (a lenta, de 300 ms, foi cancelada), e o do prazo, em exatamente `500ms`.

## 4. O coletor que perde mensagens

Projeto: [`ex4-coletor/`](ex4-coletor/src/lib.rs).

Com o código inicial:

```text
thread 'tests::nenhuma_mensagem_se_perde' (265341) panicked at src/lib.rs:60:9:
assertion `left == right` failed
  left: 0
 right: 10
```

`receber_e_processar` faz `rx.recv().await` e depois `processar(n).await`, que leva 5 ms. O `tick` dispara a cada 2 ms, então ele sempre vence enquanto o processamento ainda está no `sleep`. O `select!` descarta a future de `receber_e_processar`, e a mensagem, já tirada do canal e guardada na variável local `n`, vai junto. Nas voltas seguintes, a próxima mensagem tem o mesmo destino. Nenhuma é processada, e não há erro nenhum: o canal fecha e o coletor devolve um relatório vazio.

A correção põe só o `recv`, que é seguro para cancelar, na disputa, e o processamento no corpo do braço:

```rust
tokio::select! {
    msg = rx.recv() => match msg {
        Some(n) => {
            r.total += processar(n).await;
            r.mensagens += 1;
        }
        None => return r,
    },
    _ = tick.tick() => r.ticks += 1,
}
```

O corpo de um braço só começa depois que o braço venceu, e o `select!` espera o corpo terminar: ele nunca é cancelado. Os ticks que deveriam ter acontecido durante o processamento não se perdem: o `interval` do tokio, por padrão, dispara os atrasados logo em seguida (`MissedTickBehavior::Burst`).

A outra correção da lição 06 (guardar a mensagem recebida numa variável fora do laço e processá-la numa volta seguinte) também passa nos testes, mas é mais complicada aqui sem ganho nenhum.

## 5. Notificadores com `dyn`

Projeto: [`ex5-notificadores/`](ex5-notificadores/src/lib.rs).

O código inicial dá E0038 em cada uso de `dyn Notificador`:

```text
error[E0038]: the trait `Notificador` is not dyn compatible
  --> src/lib.rs:43:19
   |
43 |     canais: &[Box<dyn Notificador>],
   |                   ^^^^^^^^^^^^^^^ `Notificador` is not dyn compatible
   |
note: for a trait to be dyn compatible it needs to allow building a vtable
      for more information, visit <https://doc.rust-lang.org/reference/items/traits.html#dyn-compatibility>
  --> src/lib.rs:5:14
   |
 4 | pub trait Notificador {
   |           ----------- this trait is not dyn compatible...
 5 |     async fn enviar(&self, msg: &str) -> Result<(), String>;
   |              ^^^^^^ ...because method `enviar` is `async`
   = help: consider moving `enviar` to another trait
   = help: the following types implement `Notificador`:
             Email
             Sms
           consider defining an enum where each variant holds one of these types,
           implementing `Notificador` for this new enum and using it instead
```

A solução segue a lição 08:

```rust
pub type FutureBox<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

pub trait Notificador: Send + Sync {
    fn enviar<'a>(&'a self, msg: &'a str) -> FutureBox<'a, Result<(), String>>;
}

impl Notificador for Sms {
    fn enviar<'a>(&'a self, msg: &'a str) -> FutureBox<'a, Result<(), String>> {
        Box::pin(async move {
            sleep(Duration::from_millis(10)).await;
            if msg.chars().count() > self.limite {
                Err(format!("SMS: mensagem com mais de {} caracteres", self.limite))
            } else {
                Ok(())
            }
        })
    }
}
```

- Todas as implementações devolvem o mesmo tipo, `Pin<Box<dyn Future ...>>`, então o método cabe numa vtable.
- `async move` leva `self` e `msg` (que são referências, `&'a`) para dentro da future. O `+ 'a` diz que a future não vive mais que esses empréstimos.
- O `+ Send` na future e o `Send + Sync` como supertraits são o que o teste `roda_dentro_de_spawn` exige. O `Vec<Box<dyn Notificador>>` vai para dentro de uma task, então precisa ser `Send`, e `Box<dyn Notificador>` só é `Send` se a trait garantir isso. A future de `notificar_todos` guarda um `&Box<dyn Notificador>` através do `.await`, o que exige `dyn Notificador: Sync`. E a future de cada `enviar` também atravessa esse `.await`, por isso o `+ Send` nela.
- O `Email` usa um `std::sync::Mutex` dentro do bloco `async`, numa instrução só, sem atravessar o `.await`. Por isso a future continua `Send`.

Com a crate `async-trait`, a trait e as implementações voltariam a ser escritas com `async fn`, com `#[async_trait::async_trait]` em cima de cada uma, e a macro geraria exatamente esse código.

A alternativa sugerida pelo próprio compilador, um `enum Canal { Email(Email), Sms(Sms) }` com um `match`, também resolve se a lista de tipos for fechada. Aqui o `Espiao` dos testes mostra o motivo de usar `dyn`: um tipo de fora (o teste, ou outra crate) precisa poder implementar a trait.

---

[← Exercícios](../../modulos/12-async/exercicios.md) · [Índice do módulo](../../modulos/12-async/README.md)

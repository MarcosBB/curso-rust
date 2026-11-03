# Exercícios — Módulo 12

Soluções em [`solucoes/12-async`](../../solucoes/12-async/).

Em todos os exercícios, crie o projeto com `cargo new --lib nome`, acrescente `rust-version = "1.96"` no `Cargo.toml`, adicione o tokio e cole o código inicial em `src/lib.rs`:

```bash
cargo add tokio@1.54 --features full
cargo add --dev tokio@1.54 --features test-util
```

O objetivo final é sempre o mesmo: `cargo test` passando e `CARGO_BUILD_WARNINGS=deny cargo clippy --all-targets` sem avisos.

> [!NOTE]
> Os testes usam `#[tokio::test(start_paused = true)]` (por isso a feature `test-util`, só para os testes). Com ele, o relógio do tokio começa **parado** e, sempre que todas as tasks estão esperando um timer, avança direto para o próximo. Um teste que "dorme" 5 segundos termina em microssegundos, e o tempo medido com `tokio::time::Instant` é exato. É assim que os testes conferem se o seu código roda as coisas **ao mesmo tempo**.

## 1. Buscar tudo ao mesmo tempo (fácil, passe nos testes)

Lição: [02](02-tokio-e-tasks.md).

`cargo new --lib buscas`. `buscar_todos` funciona, mas faz uma busca de cada vez: com 50 ids, demora 5 segundos. Faça as buscas rodarem ao mesmo tempo, **mantendo a ordem dos ids** no resultado, e implemente `buscar_dois_grupos`.

```rust
use tokio::time::{Duration, sleep};

/// Simula uma consulta pela rede: demora 100 ms.
pub async fn buscar(id: u32) -> String {
    sleep(Duration::from_millis(100)).await;
    format!("usuario-{id}")
}

/// Busca todos os ids ao mesmo tempo e devolve na ordem dos ids.
pub async fn buscar_todos(ids: &[u32]) -> Vec<String> {
    let mut saida = Vec::new();
    for &id in ids {
        saida.push(buscar(id).await);
    }
    saida
}

/// Busca as duas listas ao mesmo tempo.
pub async fn buscar_dois_grupos(a: &[u32], b: &[u32]) -> (Vec<String>, Vec<String>) {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::Instant;

    // `start_paused`: o relógio do tokio fica parado e avança sozinho quando
    // todas as tasks estão esperando um timer. O teste roda em microssegundos
    // e o tempo medido é exato.
    #[tokio::test(start_paused = true)]
    async fn devolve_na_ordem() {
        let r = buscar_todos(&[3, 1, 2]).await;
        assert_eq!(r, ["usuario-3", "usuario-1", "usuario-2"]);
    }

    #[tokio::test(start_paused = true)]
    async fn roda_ao_mesmo_tempo() {
        let inicio = Instant::now();
        let ids: Vec<u32> = (0..50).collect();
        assert_eq!(buscar_todos(&ids).await.len(), 50);
        assert!(
            inicio.elapsed() < Duration::from_millis(150),
            "{:?}",
            inicio.elapsed()
        );
    }

    #[tokio::test(start_paused = true)]
    async fn dois_grupos_ao_mesmo_tempo() {
        let inicio = Instant::now();
        let (a, b) = buscar_dois_grupos(&[1, 2], &[3]).await;
        assert_eq!(a, ["usuario-1", "usuario-2"]);
        assert_eq!(b, ["usuario-3"]);
        assert!(
            inicio.elapsed() < Duration::from_millis(150),
            "{:?}",
            inicio.elapsed()
        );
    }

    #[tokio::test(start_paused = true)]
    async fn vazio() {
        assert!(buscar_todos(&[]).await.is_empty());
    }
}
```

<details>
<summary>Dica</summary>

O teste `roda_ao_mesmo_tempo` falha com `5s`. Crie uma task por id com `tokio::spawn` e guarde os `JoinHandle` num `Vec`, **na ordem dos ids**; só depois espere cada um. (Um `JoinSet` entregaria na ordem de término, não dos ids.) `buscar(id)` recebe o `id` por valor, então a future é `'static`.

Para `buscar_dois_grupos`, as duas chamadas são conhecidas ao escrever o código: é o caso do `join!`.
</details>

## 2. O guarda que atravessa o `.await` (médio, faça compilar e passe nos testes)

Lição: [05](05-canais-e-estado.md) (e [02](02-tokio-e-tasks.md)).

`cargo new --lib salvar`. O código não compila. Conserte-o **sem trocar o `std::sync::Mutex` pelo do tokio** e mantendo os itens sendo salvos ao mesmo tempo.

```rust
// ❌ NÃO COMPILA
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use tokio::time::{Duration, sleep};

async fn salvar(log: Arc<Mutex<Vec<String>>>, item: String) {
    let mut v = log.lock().unwrap();
    v.push(format!("começando {item}"));
    sleep(Duration::from_millis(10)).await; // simula E/S
    v.push(format!("salvo {item}"));
}

/// Salva todos os itens ao mesmo tempo e devolve o log.
pub async fn salvar_todos(itens: Vec<String>) -> Vec<String> {
    let log = Arc::new(Mutex::new(Vec::new()));
    let prefixo = Rc::new(String::from("item-"));
    let mut handles = Vec::new();
    for item in itens {
        let nome = format!("{prefixo}{item}");
        handles.push(tokio::spawn(salvar(log.clone(), nome)));
    }
    for h in handles {
        h.await.unwrap();
    }
    log.lock().unwrap().clone()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::Instant;

    #[tokio::test(start_paused = true)]
    async fn registra_tudo() {
        let itens: Vec<String> = ["a", "b", "c"].map(String::from).to_vec();
        let log = tokio::spawn(salvar_todos(itens)).await.unwrap();
        assert_eq!(log.len(), 6);
        for x in ["a", "b", "c"] {
            assert!(log.contains(&format!("começando item-{x}")));
            assert!(log.contains(&format!("salvo item-{x}")));
        }
    }

    #[tokio::test(start_paused = true)]
    async fn nao_serializa() {
        let itens: Vec<String> = (0..10).map(|i| i.to_string()).collect();
        let inicio = Instant::now();
        tokio::spawn(salvar_todos(itens)).await.unwrap();
        assert!(
            inicio.elapsed() < Duration::from_millis(30),
            "{:?}",
            inicio.elapsed()
        );
    }
}
```

<details>
<summary>Dica</summary>

São dois erros, e o segundo só aparece depois que o primeiro some (e só no `cargo test`, que chama `salvar_todos` dentro de um `tokio::spawn`). Leia a nota *used across an await* de cada um: ela diz qual valor está vivo durante qual `.await`.

Para o guarda: lembre que `drop(v)` antes do `.await` não basta. Para o outro: qual é a versão `Send` daquele ponteiro?

Por que não trocar pelo `tokio::sync::Mutex`? Experimente: compila, mas o teste `nao_serializa` mostra o efeito.
</details>

## 3. A resposta mais rápida (médio, passe nos testes)

Lições: [02](02-tokio-e-tasks.md), [03](03-tempo-e-bloqueio.md) e [06](06-select-e-cancelamento.md).

`cargo new --lib corrida`. Implemente `mais_rapida`: consulte todos os servidores ao mesmo tempo e devolva a **primeira resposta `Ok`**. Respostas `Err` são ignoradas. Se todos falharem, `Err(Erro::TodosFalharam)`; se o `prazo` vencer antes de alguma resposta boa, `Err(Erro::Prazo)`. Quando a função retornar, as consultas que ainda estiverem rodando devem ser canceladas.

```rust
use tokio::task::JoinSet;
use tokio::time::{Duration, sleep, timeout};

#[derive(Debug, PartialEq)]
pub enum Erro {
    TodosFalharam,
    Prazo,
}

/// Um servidor de teste: responde depois de `atraso_ms`, com sucesso ou não.
#[derive(Clone)]
pub struct Servidor {
    pub nome: &'static str,
    pub atraso_ms: u64,
    pub funciona: bool,
}

pub async fn consultar(s: Servidor) -> Result<String, String> {
    sleep(Duration::from_millis(s.atraso_ms)).await;
    if s.funciona {
        Ok(format!("resposta de {}", s.nome))
    } else {
        Err(format!("{} falhou", s.nome))
    }
}

/// Consulta todos ao mesmo tempo e devolve a primeira resposta `Ok`.
/// Se todos falharem, `TodosFalharam`; se o prazo vencer antes, `Prazo`.
pub async fn mais_rapida(servidores: Vec<Servidor>, prazo: Duration) -> Result<String, Erro> {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::Instant;

    fn s(nome: &'static str, atraso_ms: u64, funciona: bool) -> Servidor {
        Servidor {
            nome,
            atraso_ms,
            funciona,
        }
    }

    const PRAZO: Duration = Duration::from_millis(500);

    #[tokio::test(start_paused = true)]
    async fn ganha_a_mais_rapida() {
        let inicio = Instant::now();
        let r = mais_rapida(vec![s("a", 300, true), s("b", 100, true)], PRAZO).await;
        assert_eq!(r, Ok("resposta de b".into()));
        assert_eq!(inicio.elapsed(), Duration::from_millis(100));
    }

    #[tokio::test(start_paused = true)]
    async fn ignora_as_que_falham() {
        let r = mais_rapida(vec![s("a", 50, false), s("b", 200, true)], PRAZO).await;
        assert_eq!(r, Ok("resposta de b".into()));
    }

    #[tokio::test(start_paused = true)]
    async fn todas_falham() {
        let inicio = Instant::now();
        let r = mais_rapida(vec![s("a", 50, false), s("b", 80, false)], PRAZO).await;
        assert_eq!(r, Err(Erro::TodosFalharam));
        assert_eq!(inicio.elapsed(), Duration::from_millis(80));
    }

    #[tokio::test(start_paused = true)]
    async fn prazo_vence() {
        let inicio = Instant::now();
        let r = mais_rapida(vec![s("a", 900, true)], PRAZO).await;
        assert_eq!(r, Err(Erro::Prazo));
        assert_eq!(inicio.elapsed(), PRAZO);
    }

    #[tokio::test(start_paused = true)]
    async fn lista_vazia() {
        assert_eq!(mais_rapida(vec![], PRAZO).await, Err(Erro::TodosFalharam));
    }
}
```

<details>
<summary>Dica</summary>

Um `JoinSet` com uma task por servidor, e `join_next().await` em laço: os resultados chegam por ordem de término. O resultado de cada `join_next` é um `Result<Result<String, String>, JoinError>`.

Para o prazo, ponha o laço inteiro dentro de um bloco `async` e passe-o ao `tokio::time::timeout`. E o cancelamento das que sobraram vem de graça: o que acontece com um `JoinSet` descartado?
</details>

## 4. O coletor que perde mensagens (médio, passe nos testes)

Lição: [06](06-select-e-cancelamento.md).

`cargo new --lib coletor`. O código compila, mas o teste `nenhuma_mensagem_se_perde` falha: das 10 mensagens enviadas, o coletor processa **zero**. Descubra por quê e conserte, sem mudar `processar` nem os testes.

```rust
use tokio::sync::mpsc;
use tokio::time::{Duration, interval, sleep};

#[derive(Debug, Default, PartialEq)]
pub struct Relatorio {
    pub total: u64,
    pub mensagens: usize,
    pub ticks: u32,
}

/// Simula um processamento lento (uma escrita no disco, por exemplo).
async fn processar(n: u64) -> u64 {
    sleep(Duration::from_millis(5)).await;
    n * 10
}

/// Recebe uma mensagem e já a processa.
async fn receber_e_processar(rx: &mut mpsc::Receiver<u64>) -> Option<u64> {
    let n = rx.recv().await?;
    Some(processar(n).await)
}

/// Recebe e processa até o canal fechar, contando os ticks do relógio.
pub async fn coletar(mut rx: mpsc::Receiver<u64>, intervalo: Duration) -> Relatorio {
    let mut tick = interval(intervalo);
    let mut r = Relatorio::default();
    loop {
        tokio::select! {
            v = receber_e_processar(&mut rx) => match v {
                Some(x) => {
                    r.total += x;
                    r.mensagens += 1;
                }
                None => return r,
            },
            _ = tick.tick() => r.ticks += 1,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn produtor(n: u64, pausa: Duration) -> mpsc::Receiver<u64> {
        let (tx, rx) = mpsc::channel(4);
        tokio::spawn(async move {
            for i in 1..=n {
                tx.send(i).await.unwrap();
                sleep(pausa).await;
            }
        });
        rx
    }

    #[tokio::test(start_paused = true)]
    async fn nenhuma_mensagem_se_perde() {
        let rx = produtor(10, Duration::from_millis(3));
        let r = coletar(rx, Duration::from_millis(2)).await;
        assert_eq!(r.mensagens, 10);
        assert_eq!(r.total, 550);
        assert!(r.ticks > 0);
    }

    #[tokio::test(start_paused = true)]
    async fn canal_vazio_termina() {
        let (tx, rx) = mpsc::channel(1);
        drop(tx);
        let r = coletar(rx, Duration::from_millis(2)).await;
        assert_eq!((r.total, r.mensagens), (0, 0));
    }

    #[tokio::test(start_paused = true)]
    async fn tick_lento() {
        let rx = produtor(5, Duration::from_millis(1));
        let r = coletar(rx, Duration::from_secs(1)).await;
        assert_eq!(r.total, 150);
        assert_eq!(r.ticks, 1); // o primeiro tick de `interval` é imediato
    }
}
```

<details>
<summary>Dica</summary>

`receber_e_processar` tem dois `.await`. O que acontece com a mensagem já recebida quando o `tick` vence o `select!` durante o `processar`? A lição 06 mostra duas saídas: guardar o estado fora do `select!`, ou deixar o trabalho lento no **corpo** do braço, que nunca é cancelado.
</details>

## 5. Notificadores com `dyn` (difícil, faça compilar e passe nos testes)

Lições: [07](07-pin-e-unpin.md) e [08](08-async-em-traits-e-closures.md).

`cargo new --lib notificadores`. A trait `Notificador` tem um método `async`, e `notificar_todos` recebe uma lista de `Box<dyn Notificador>`. Não compila. Torne a trait dyn-compatível **sem usar a crate `async-trait`**, de modo que os testes (inclusive o que roda `notificar_todos` dentro de `tokio::spawn`) passem. O `Espiao` dos testes também implementa a trait: adapte-o, sem mudar o que ele faz.

```rust
// ❌ NÃO COMPILA
use std::sync::Mutex;
use tokio::time::{Duration, sleep};

pub trait Notificador {
    async fn enviar(&self, msg: &str) -> Result<(), String>;
}

/// Guarda as mensagens "enviadas" (num servidor de verdade, seria SMTP).
#[derive(Default)]
pub struct Email {
    pub enviados: Mutex<Vec<String>>,
}

impl Notificador for Email {
    async fn enviar(&self, msg: &str) -> Result<(), String> {
        sleep(Duration::from_millis(20)).await;
        self.enviados.lock().unwrap().push(msg.to_string());
        Ok(())
    }
}

/// Recusa mensagens maiores que `limite` caracteres.
pub struct Sms {
    pub limite: usize,
}

impl Notificador for Sms {
    async fn enviar(&self, msg: &str) -> Result<(), String> {
        sleep(Duration::from_millis(10)).await;
        if msg.chars().count() > self.limite {
            Err(format!(
                "SMS: mensagem com mais de {} caracteres",
                self.limite
            ))
        } else {
            Ok(())
        }
    }
}

/// Envia `msg` por todos os canais, um depois do outro, e devolve os resultados.
pub async fn notificar_todos(
    canais: &[Box<dyn Notificador>],
    msg: &str,
) -> Vec<Result<(), String>> {
    let mut resultados = Vec::with_capacity(canais.len());
    for c in canais {
        resultados.push(c.enviar(msg).await);
    }
    resultados
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    /// Um notificador de teste que compartilha o registro com o teste.
    struct Espiao(Arc<Mutex<Vec<String>>>);

    impl Notificador for Espiao {
        async fn enviar(&self, msg: &str) -> Result<(), String> {
            self.0.lock().unwrap().push(msg.to_string());
            Ok(())
        }
    }

    #[tokio::test(start_paused = true)]
    async fn email_e_sms() {
        let canais: Vec<Box<dyn Notificador>> =
            vec![Box::new(Email::default()), Box::new(Sms { limite: 5 })];
        let r = notificar_todos(&canais, "oi").await;
        assert_eq!(r, [Ok(()), Ok(())]);
        let r = notificar_todos(&canais, "mensagem longa").await;
        assert_eq!(r[0], Ok(()));
        assert_eq!(r[1], Err("SMS: mensagem com mais de 5 caracteres".into()));
    }

    #[tokio::test(start_paused = true)]
    async fn roda_dentro_de_spawn() {
        let registro = Arc::new(Mutex::new(Vec::new()));
        let canais: Vec<Box<dyn Notificador>> = vec![Box::new(Espiao(registro.clone()))];
        let r = tokio::spawn(async move { notificar_todos(&canais, "olá").await })
            .await
            .unwrap();
        assert_eq!(r, [Ok(())]);
        assert_eq!(*registro.lock().unwrap(), ["olá"]);
    }
}
```

<details>
<summary>Dica</summary>

O método precisa devolver um tipo só para todas as implementações: `Pin<Box<dyn Future<Output = ...> + Send + 'a>>`. Um `type` ajuda a não repetir isso. O `'a` liga a future a `&self` e a `msg`.

O teste com `tokio::spawn` exige que `Vec<Box<dyn Notificador>>` seja `Send`. `Box<dyn Notificador>` só é `Send` se o compilador souber que todo `Notificador` é `Send`: declare `Send + Sync` como *supertraits* (Módulo 06).
</details>

---

[← Última lição](08-async-em-traits-e-closures.md) · [Índice do módulo](README.md) · [Soluções →](../../solucoes/12-async/README.md)

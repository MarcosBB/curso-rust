# 05 — Atômicos: contadores e flags sem lock

**Objetivos.** Ao final, você vai saber:

- usar `AtomicU64`, `AtomicUsize` e `AtomicBool` para contadores e flags compartilhados;
- as operações básicas: `load`, `store`, `fetch_add` e `compare_exchange`;
- escolher o `Ordering` certo nos casos práticos (`Relaxed`, `Release`/`Acquire`, `SeqCst`);
- decidir entre um atômico e um `Mutex`.

## O contador, de novo

O contador da lição 04 usou `Arc<Mutex<i32>>`. Para um número só, há uma ferramenta mais leve: os tipos atômicos de `std::sync::atomic`. Uma operação atômica é indivisível: nenhuma outra thread vê o valor "pela metade", e o "ler, somar, gravar" do `fetch_add` acontece de uma vez, direto no processador.

```rust
use std::sync::atomic::{AtomicU64, Ordering};
use std::thread;

static CONTADOR: AtomicU64 = AtomicU64::new(0);

fn main() {
    thread::scope(|s| {
        for _ in 0..8 {
            s.spawn(|| {
                for _ in 0..100_000 {
                    CONTADOR.fetch_add(1, Ordering::Relaxed);
                }
            });
        }
    });
    println!("{}", CONTADOR.load(Ordering::Relaxed));
}
```

```text
800000
```

É o programa em C++ da lição 02, que dava 258547, 285687, 172723. Aqui, sempre 800000.

Repare em duas coisas. Primeiro, `fetch_add` recebe `&self`: um atômico é **mutabilidade interior** (Módulo 10), como o `Cell`, só que segura entre threads. Por isso ele pode ficar num `static` (lição 06) ou ser compartilhado por `&` dentro de um `thread::scope`, sem `Mutex` e sem `Arc`. Segundo, todo método recebe um `Ordering`, que é o assunto da segunda metade desta lição.

## Os tipos e as operações

Há atômicos para os inteiros (`AtomicU8` a `AtomicU64`, `AtomicI8` a `AtomicI64`, `AtomicUsize`, `AtomicIsize`), para `bool` (`AtomicBool`) e para ponteiros (`AtomicPtr`). Eles não são genéricos: não existe `Atomic<String>`, porque o processador só faz operações atômicas em valores pequenos.

| Operação | O que faz |
|---|---|
| `load(ord)` | lê o valor |
| `store(v, ord)` | grava `v` |
| `swap(v, ord)` | grava `v` e devolve o antigo |
| `fetch_add(n, ord)`, `fetch_sub`, `fetch_max`, `fetch_min`, `fetch_or`... | aplica a operação e devolve o valor **anterior** |
| `compare_exchange(atual, novo, ord_ok, ord_falha)` | grava `novo` só se o valor ainda for `atual`; devolve `Ok(anterior)` ou `Err(valor_visto)` |
| `fetch_update(ord, ord, f)` | laço de `compare_exchange` pronto, com uma closure |
| `into_inner()`, `get_mut()` | acesso sem atomicidade, quando você é dono do atômico (ou tem `&mut`) |

### Uma flag de parada

`AtomicBool` é o jeito simples de avisar uma thread que ela deve parar:

```rust
let parar = AtomicBool::new(false);
let voltas = AtomicU64::new(0);
thread::scope(|s| {
    s.spawn(|| {
        while !parar.load(Ordering::Relaxed) {
            voltas.fetch_add(1, Ordering::Relaxed);
            thread::sleep(Duration::from_millis(1));
        }
    });
    thread::sleep(Duration::from_millis(20));
    parar.store(true, Ordering::Relaxed);
});
```

Quantas voltas a thread dá depende do escalonador e varia a cada execução; o que é garantido é que ela para.

### Ler, decidir, gravar: `compare_exchange`

`fetch_add` resolve "somar um". Quando a decisão depende do valor atual, como "dê o próximo número, mas não passe de 3", ler com `load` e gravar com `store` separados reabre a corrida: outra thread pode mudar o valor entre os dois. O `compare_exchange` junta as duas coisas: grava só se ninguém mexeu desde a leitura. Se alguém mexeu, você recebe o valor novo e tenta de novo.

```rust
static PROXIMO: AtomicU32 = AtomicU32::new(1);

/// Devolve ids de 1 a 3; depois disso, None. Nunca passa do limite.
fn reservar_vaga() -> Option<u32> {
    let mut atual = PROXIMO.load(Ordering::Relaxed);
    loop {
        if atual > 3 {
            return None;
        }
        match PROXIMO.compare_exchange(atual, atual + 1, Ordering::Relaxed, Ordering::Relaxed) {
            Ok(_) => return Some(atual),
            Err(visto) => atual = visto,
        }
    }
}
```

Cinco threads chamando `reservar_vaga`, duas execuções:

```text
thread 0: Some(1)        thread 1: Some(2)
thread 1: Some(2)        thread 3: None
thread 2: Some(3)        thread 4: None
thread 3: None           thread 2: Some(3)
thread 4: None           thread 0: Some(1)
```

A ordem muda, e até qual thread pega qual vaga pode mudar; mas são sempre três `Some` distintos e dois `None`. O `fetch_update` escreve esse laço para você; o laço explícito mostra o mecanismo.

## `Ordering` na prática

O processador e o compilador reordenam leituras e escritas para ganhar velocidade. Numa thread só, você nunca percebe. Entre threads, uma thread pode ver as escritas da outra **em outra ordem**. O `Ordering` diz quanto dessa reordenação é permitida em volta de cada operação atômica. O modelo é o mesmo do C++ (`std::memory_order`), com os mesmos nomes.

Para o dia a dia, três casos cobrem quase tudo:

**`Relaxed`: o atômico é o dado.** Contadores, estatísticas, geradores de id, uma flag de parada que não "publica" nada. A operação é atômica e não se perde, mas não ordena nada em volta dela. Todos os exemplos acima usam `Relaxed`, e estão corretos.

**`Release` + `Acquire`: o atômico avisa que outro dado está pronto.** Uma thread prepara um dado e depois liga uma flag com `Release`; outra lê a flag com `Acquire` e, se vir `true`, tem a garantia de ver também tudo o que a primeira escreveu antes da flag.

```rust
let dado = AtomicU64::new(0);
let pronto = AtomicBool::new(false);
thread::scope(|s| {
    s.spawn(|| {
        dado.store(42, Ordering::Relaxed);
        pronto.store(true, Ordering::Release);   // "publica" o que veio antes
    });
    while !pronto.load(Ordering::Acquire) {      // "recebe" o que veio antes
        std::hint::spin_loop();
    }
    println!("{}", dado.load(Ordering::Relaxed)); // sempre 42
});
```

Com `Relaxed` na flag, a thread principal poderia ver `pronto == true` e ainda ler `dado == 0`. Os nomes ajudam a lembrar: `Release` (soltar) é como destravar um mutex, `Acquire` (adquirir) é como travar. É assim que os próprios locks são implementados por baixo.

**`SeqCst`: na dúvida.** É a ordenação mais forte: todas as operações `SeqCst` de todas as threads parecem acontecer numa única ordem global. É o padrão do C++ quando você não escreve nada. Raramente é **necessária**, mas nunca é a escolha errada para a correção, só um pouco mais lenta em alguns processadores.

> [!TIP]
> Se você precisa de mais de um atômico para manter um invariante ("o saldo e o histórico mudam juntos"), use um `Mutex`. Atômicos garantem cada operação isolada; combinar várias corretamente é difícil, e é o tema de um livro inteiro (*Rust Atomics and Locks*, gratuito online).

## Atômico ou `Mutex`?

| Situação | Use |
|---|---|
| um contador, um id, uma estatística | `AtomicU64`/`AtomicUsize` com `Relaxed` |
| uma flag de parada ou "já inicializado" | `AtomicBool` |
| um valor maior que 8 bytes, ou vários valores que mudam juntos | `Mutex<T>` |
| "ler, decidir, gravar" num número | `compare_exchange` ou `fetch_update` |
| "ler, decidir, gravar" numa estrutura | `Mutex<T>` |

O `Arc` que você usou nas lições anteriores é, por dentro, exatamente isto: um `AtomicUsize` com o contador de donos, que `clone` incrementa e `drop` decrementa.

## Armadilhas comuns

- **`load` seguido de `store` como se fosse uma operação só.** `if x.load(..) < 10 { x.store(x.load(..) + 1, ..) }` tem a mesma corrida do C++: duas threads leem 9 e as duas gravam 10. Use `fetch_add`, `compare_exchange` ou `fetch_update`.
- **Esquecer que `fetch_add` devolve o valor anterior.** `let id = PROXIMO.fetch_add(1, ..)` dá o id **antes** do incremento. É exatamente o que um gerador de id quer.
- **Laço de espera ocupada.** O `while !pronto.load(..)` do exemplo gasta CPU enquanto espera. Serve para esperas curtíssimas; para esperas longas, use um canal, `join` ou `Condvar`.
- **Achar que `Relaxed` é perigoso por definição.** Para um contador isolado, ele é correto. O cuidado é só quando o atômico serve para sinalizar **outro** dado.

## Resumo

- Atômicos (`AtomicU64`, `AtomicBool`...) dão mutabilidade interior segura entre threads para valores pequenos, sem lock.
- `fetch_add` e companhia fazem "ler, alterar, gravar" de uma vez e devolvem o valor anterior. `compare_exchange` grava só se ninguém mexeu.
- `Relaxed` para contadores e flags isoladas; `Release` no `store` + `Acquire` no `load` para publicar outro dado; `SeqCst` quando houver dúvida.
- Vários valores que mudam juntos pedem `Mutex`.

## Exercícios

Veja o exercício 4 em [`exercicios.md`](exercicios.md).

## Leitura complementar

- [Documentação de `std::sync::atomic`](https://doc.rust-lang.org/std/sync/atomic/index.html) e de [`Ordering`](https://doc.rust-lang.org/std/sync/atomic/enum.Ordering.html)
- [Rust Atomics and Locks, cap. 2 (Atomics) e 3 (Memory Ordering)](https://marabos.nl/atomics/atomics.html) (Mara Bos; gratuito online)
- [Rustonomicon: Atomics](https://doc.rust-lang.org/nomicon/atomics.html)

---

[← Lição anterior](04-mutex-e-rwlock.md) · [Índice do módulo](README.md) · [Próxima lição →](06-estado-global.md)

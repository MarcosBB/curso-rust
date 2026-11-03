# Exercícios — Módulo 11

Soluções em [`solucoes/11-concorrencia`](../../solucoes/11-concorrencia/).

Em todos os exercícios, crie o projeto com `cargo new --lib nome`, acrescente `rust-version = "1.96"` no `Cargo.toml` e cole o código inicial em `src/lib.rs`. O objetivo final é sempre o mesmo: `cargo test` passando e `CARGO_BUILD_WARNINGS=deny cargo clippy --all-targets` sem avisos.

Os testes de código concorrente rodam muitas vezes com resultados que **não** podem variar (a soma, a contagem, o estoque final), mesmo que a ordem das threads varie. Se um teste passar às vezes e falhar às vezes, há uma corrida no seu código.

## 1. Soma paralela (fácil, faça compilar e passe nos testes)

Lição: [01](01-threads-e-scope.md).

`cargo new --lib soma`. A função `soma_paralela` não compila. Conserte-a **sem copiar os dados** (nada de `to_vec`) e implemente `maximo_paralelo` do mesmo jeito.

```rust
// ❌ NÃO COMPILA
use std::thread;

/// Soma `dados` com até `partes` threads.
pub fn soma_paralela(dados: &[u64], partes: usize) -> u64 {
    let tamanho = dados.len().div_ceil(partes.max(1)).max(1);
    let mut handles = Vec::new();
    for pedaco in dados.chunks(tamanho) {
        handles.push(thread::spawn(move || pedaco.iter().sum::<u64>()));
    }
    handles.into_iter().map(|h| h.join().unwrap()).sum()
}

/// O maior valor de `dados`, com até `partes` threads (`None` se vazio).
pub fn maximo_paralelo(dados: &[i64], partes: usize) -> Option<i64> {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn soma_igual_a_sequencial() {
        let dados: Vec<u64> = (1..=10_000).collect();
        for partes in [1, 2, 3, 8, 100] {
            assert_eq!(soma_paralela(&dados, partes), 50_005_000);
        }
    }

    #[test]
    fn casos_de_borda() {
        assert_eq!(soma_paralela(&[], 4), 0);
        assert_eq!(soma_paralela(&[7], 0), 7);
        assert_eq!(maximo_paralelo(&[], 4), None);
    }

    #[test]
    fn maximo() {
        let dados = [3, -1, 42, 7, 42, -100, 0];
        for partes in 1..=8 {
            assert_eq!(maximo_paralelo(&dados, partes), Some(42));
        }
    }
}
```

<details>
<summary>Dica</summary>

O erro não é o E0373 da lição, porque já há um `move`; leia a linha `argument requires that ... must outlive 'static`. O `move` levou a referência `pedaco`, mas ela continua apontando para `dados`, que é emprestado de quem chamou. Qual ferramenta da lição 01 permite que threads emprestem?

Para `maximo_paralelo`, cada thread devolve `Option<i64>` (um pedaço nunca é vazio, mas o tipo é esse); junte com `filter_map` e `max`.
</details>

## 2. Contagem de palavras com canal (médio, passe nos testes)

Lição: [03](03-canais.md).

`cargo new --lib contagem`. Divida os textos em `workers` lotes (o texto `i` vai para o lote `i % workers`). Cada lote roda numa thread, que conta as palavras do lote num `HashMap` **local** e manda o mapa pelo canal. A função junta os mapas que chegarem e devolve o total. Nada de `Mutex`.

```rust
use std::collections::HashMap;
use std::sync::mpsc;
use std::thread;

/// Conta as palavras (em minúsculas) de um texto.
fn contar(texto: &str) -> HashMap<String, usize> {
    let mut mapa = HashMap::new();
    for p in texto.split_whitespace() {
        *mapa.entry(p.to_lowercase()).or_insert(0) += 1;
    }
    mapa
}

/// `workers == 0` deve funcionar como 1.
pub fn contar_palavras(textos: Vec<String>, workers: usize) -> HashMap<String, usize> {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn textos() -> Vec<String> {
        ["O rato roeu", "a roupa do rei", "de Roma o rato", "", "rei rei"]
            .iter()
            .map(|s| s.to_string())
            .collect()
    }

    #[test]
    fn igual_a_sequencial() {
        let esperado = contar(&textos().join(" "));
        for w in [1, 2, 3, 10] {
            assert_eq!(contar_palavras(textos(), w), esperado);
        }
    }

    #[test]
    fn contagens() {
        let c = contar_palavras(textos(), 3);
        assert_eq!(c["rato"], 2);
        assert_eq!(c["rei"], 3);
        assert_eq!(c["o"], 2);
        assert_eq!(c.get("gato"), None);
    }

    #[test]
    fn sem_textos_e_zero_workers() {
        assert!(contar_palavras(Vec::new(), 4).is_empty());
        assert_eq!(contar_palavras(vec!["a a".into()], 0)["a"], 2);
    }
}
```

<details>
<summary>Dica</summary>

Monte os lotes como `vec![Vec::new(); workers]` e mova cada lote para a sua thread. Se os testes travarem, procure o `Sender` que ainda está vivo (lição 03, "O `drop(tx)` não é enfeite").
</details>

## 3. Estoque compartilhado (médio, faça compilar e conserte o deadlock)

Lições: [02](02-send-e-sync.md) e [04](04-mutex-e-rwlock.md).

`cargo new --lib estoque`. Este `Estoque` foi escrito para uma thread só, e o teste com threads não compila. Faça compilar trocando os tipos. Cuidado: a troca mais direta (cada `borrow_mut()` vira um `lock().unwrap()`) compila, mas um dos testes **trava** e nunca termina (o `cargo test` avisa `has been running for over 60 seconds`; interrompa com Ctrl+C). Descubra qual linha causa o deadlock e conserte.

O teste com 8 threads tem de passar sempre: 800 tentativas de reserva para 500 unidades, e nenhuma venda a mais.

```rust
// ❌ NÃO COMPILA (nos testes)
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

/// Um estoque que pode ser clonado. Os clones compartilham os mesmos itens.
#[derive(Clone, Default)]
pub struct Estoque {
    itens: Rc<RefCell<HashMap<String, u32>>>,
}

impl Estoque {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn repor(&self, item: &str, qtd: u32) {
        *self.itens.borrow_mut().entry(item.to_string()).or_insert(0) += qtd;
    }

    /// Tira `qtd` unidades de `item`, se houver. Um item desconhecido é
    /// cadastrado com 0 unidades. Devolve `true` se a reserva foi feita.
    pub fn reservar(&self, item: &str, qtd: u32) -> bool {
        match self.itens.borrow_mut().get_mut(item) {
            Some(n) if *n >= qtd => {
                *n -= qtd;
                true
            }
            Some(_) => false,
            None => {
                self.itens.borrow_mut().insert(item.to_string(), 0);
                false
            }
        }
    }

    pub fn quantidade(&self, item: &str) -> Option<u32> {
        self.itens.borrow().get(item).copied()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn reserva_simples() {
        let e = Estoque::new();
        e.repor("café", 3);
        assert!(e.reservar("café", 2));
        assert!(!e.reservar("café", 2));
        assert_eq!(e.quantidade("café"), Some(1));
    }

    #[test]
    fn item_desconhecido_e_cadastrado() {
        let e = Estoque::new();
        assert!(!e.reservar("chá", 1));
        assert_eq!(e.quantidade("chá"), Some(0));
    }

    #[test]
    fn varias_threads_nunca_vendem_demais() {
        let e = Estoque::new();
        e.repor("café", 500);
        let mut handles = Vec::new();
        for _ in 0..8 {
            let e = e.clone();
            handles.push(thread::spawn(move || {
                (0..100).filter(|_| e.reservar("café", 1)).count()
            }));
        }
        let vendidos: usize = handles.into_iter().map(|h| h.join().unwrap()).sum();
        assert_eq!(vendidos, 500);
        assert_eq!(e.quantidade("café"), Some(0));
    }
}
```

<details>
<summary>Dica</summary>

A tabela da lição 02 diz o que vira o quê. Para o deadlock: quanto tempo vive o guarda criado na expressão de um `match`? Uma solução trava uma vez, guarda o `MutexGuard` numa variável e usa o mesmo guarda na verificação e na alteração. Por que "copiar a quantidade, soltar o lock e travar de novo para alterar" quebraria o teste com 8 threads?
</details>

## 4. Métricas sem lock (médio, passe nos testes)

Lições: [05](05-atomicos.md) e [06](06-estado-global.md).

`cargo new --lib metricas`. Implemente as funções marcadas com `todo!()` **sem `Mutex`**: só atômicos e `LazyLock`. `proximo_id` usa um contador global; `Metricas` guarda o número de requisições, o total de bytes e a maior requisição; `status_http` consulta uma tabela global com pelo menos os códigos 200, 201, 404 e 500.

```rust
use std::collections::HashMap;
use std::sync::LazyLock;
use std::sync::atomic::{AtomicU64, Ordering};

/// Um id novo a cada chamada, começando em 1, único entre threads.
pub fn proximo_id() -> u64 {
    todo!()
}

/// Métricas de requisições, atualizadas por várias threads.
pub struct Metricas {
    requisicoes: AtomicU64,
    bytes: AtomicU64,
    maior: AtomicU64,
}

impl Metricas {
    pub const fn new() -> Self {
        Self {
            requisicoes: AtomicU64::new(0),
            bytes: AtomicU64::new(0),
            maior: AtomicU64::new(0),
        }
    }

    pub fn registrar(&self, bytes: u64) {
        todo!()
    }

    pub fn requisicoes(&self) -> u64 {
        todo!()
    }

    pub fn bytes(&self) -> u64 {
        todo!()
    }

    pub fn maior(&self) -> u64 {
        todo!()
    }
}

impl Default for Metricas {
    fn default() -> Self {
        Self::new()
    }
}

/// Métricas do processo inteiro.
pub static GLOBAL: Metricas = Metricas::new();

/// A descrição de um código HTTP conhecido ("OK", "Not Found"...).
pub fn status_http(codigo: u16) -> Option<&'static str> {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;
    use std::thread;

    #[test]
    fn ids_unicos_entre_threads() {
        let ids: Vec<u64> = thread::scope(|s| {
            let hs: Vec<_> = (0..8)
                .map(|_| s.spawn(|| (0..1000).map(|_| proximo_id()).collect::<Vec<_>>()))
                .collect();
            hs.into_iter().flat_map(|h| h.join().unwrap()).collect()
        });
        let unicos: HashSet<u64> = ids.iter().copied().collect();
        assert_eq!(unicos.len(), 8000);
        assert!(ids.iter().all(|&id| id >= 1));
    }

    #[test]
    fn metricas_somam_todas_as_threads() {
        let m = Metricas::new();
        thread::scope(|s| {
            for t in 0..8 {
                let m = &m;
                s.spawn(move || {
                    for i in 1..=1000 {
                        m.registrar(i + t);
                    }
                });
            }
        });
        assert_eq!(m.requisicoes(), 8000);
        // soma de (i + t) para i em 1..=1000 e t em 0..8
        assert_eq!(m.bytes(), 8 * 500_500 + 1000 * 28);
        assert_eq!(m.maior(), 1007);
    }

    #[test]
    fn global_e_status() {
        GLOBAL.registrar(10);
        assert!(GLOBAL.requisicoes() >= 1);
        assert_eq!(status_http(404), Some("Not Found"));
        assert_eq!(status_http(418), None);
    }
}
```

<details>
<summary>Dica</summary>

Os imports já dizem o que usar. Para "a maior requisição", há uma operação `fetch_` pronta, sem laço. Como todas as métricas são independentes, `Ordering::Relaxed` basta. Por que o teste `global_e_status` usa `>= 1`, e não `== 1`? (Pense em quantos testes podem estar rodando ao mesmo tempo.)
</details>

## 5. Paralelismo com rayon (médio, passe nos testes)

Lição: [07](07-rayon.md).

`cargo new --lib paralelo` e `cargo add rayon`. Implemente as três funções com iteradores paralelos do rayon. Em `maior_collatz`, o resultado não pode depender de como o rayon dividiu o trabalho: em caso de empate no número de passos, ganha o **menor** número.

```rust
use rayon::prelude::*;

fn eh_primo(n: u64) -> bool {
    n >= 2 && (2..).take_while(|d| d * d <= n).all(|d| !n.is_multiple_of(d))
}

/// Todos os primos menores que `n`, em ordem crescente.
pub fn primos_ate(n: u64) -> Vec<u64> {
    todo!()
}

/// Número de passos da sequência de Collatz de `n` até chegar a 1
/// (par: n / 2; ímpar: 3n + 1).
pub fn passos_collatz(mut n: u64) -> u32 {
    todo!()
}

/// O número em `1..limite` com a sequência de Collatz mais longa, e o número
/// de passos. Em caso de empate, o menor número. `None` se `limite <= 1`.
pub fn maior_collatz(limite: u64) -> Option<(u64, u32)> {
    todo!()
}

/// Soma dos quadrados, em paralelo.
pub fn soma_quadrados(v: &[i64]) -> i64 {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn primos() {
        assert_eq!(primos_ate(30), [2, 3, 5, 7, 11, 13, 17, 19, 23, 29]);
        assert!(primos_ate(2).is_empty());
        let seq: Vec<u64> = (0..50_000).filter(|&x| eh_primo(x)).collect();
        assert_eq!(primos_ate(50_000), seq);
    }

    #[test]
    fn collatz() {
        assert_eq!(passos_collatz(1), 0);
        assert_eq!(passos_collatz(6), 8);
        assert_eq!(maior_collatz(10), Some((9, 19)));
        assert_eq!(maior_collatz(1), None);
        assert_eq!(maior_collatz(1_000_000), Some((837_799, 524)));
    }

    #[test]
    fn quadrados() {
        let v: Vec<i64> = (-100..=100).collect();
        let seq: i64 = v.iter().map(|x| x * x).sum();
        assert_eq!(soma_quadrados(&v), seq);
        assert_eq!(soma_quadrados(&[]), 0);
    }
}
```

<details>
<summary>Dica</summary>

`collect` de um iterador paralelo preserva a ordem. Para o empate, `max_by_key` aceita uma tupla como chave, e `std::cmp::Reverse(n)` inverte a comparação de `n`. Depois de passar nos testes, compare os tempos de `maior_collatz(10_000_000)` com `RAYON_NUM_THREADS=1` e sem a variável, em `cargo test --release`.
</details>

---

[← Lição 07](07-rayon.md) · [Índice do módulo](README.md) · [Soluções →](../../solucoes/11-concorrencia/README.md)

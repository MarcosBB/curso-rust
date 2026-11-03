# Soluções — Módulo 11

Enunciados em [`modulos/11-concorrencia/exercicios.md`](../../modulos/11-concorrencia/exercicios.md).

Cada solução é um projeto Cargo com os testes do enunciado. Para conferir:

```bash
cd nome-do-projeto
cargo test
```

Como os testes usam threads, rode-os algumas vezes (`for i in 1 2 3 4 5; do cargo test -q; done`). Uma corrida costuma aparecer como um teste que falha "às vezes".

## 1. Soma paralela

Projeto: [`ex1-soma-paralela/`](ex1-soma-paralela/src/lib.rs).

O código inicial dá este erro:

```text
error[E0521]: borrowed data escapes outside of function
 --> src/lib.rs:8:22
  |
4 | pub fn soma_paralela(dados: &[u64], partes: usize) -> u64 {
  |                      -----  - let's call the lifetime of this reference `'1`
  |                      |
  |                      `dados` is a reference that is only valid in the function body
...
8 |         handles.push(thread::spawn(move || pedaco.iter().sum::<u64>()));
  |                      ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
  |                      |
  |                      `dados` escapes the function body here
  |                      argument requires that `'1` must outlive `'static`
```

Não é o E0373 da lição 01, porque já há um `move`. O `move` levou `pedaco` para a thread, mas `pedaco` é um `&[u64]` que aponta para dentro de `dados`, e `dados` é emprestado de quem chamou. O `spawn` exige `'static`: a thread poderia continuar rodando depois que a função retornasse e quem chamou liberasse o vetor. O compilador não sabe que os `join` vêm logo depois.

`thread::scope` dá exatamente essa garantia: todas as threads do escopo terminam antes de ele retornar. A correção troca `thread::spawn` por `s.spawn` e põe o laço dentro do escopo:

```rust
thread::scope(|s| {
    let mut handles = Vec::new();
    for pedaco in dados.chunks(tamanho) {
        handles.push(s.spawn(move || pedaco.iter().sum::<u64>()));
    }
    handles.into_iter().map(|h| h.join().unwrap()).sum()
})
```

O `scope` devolve o valor da closure, então a soma sai direto dele. As alternativas que o enunciado proíbe (`dados.to_vec()` e mover a cópia, ou `Arc<Vec<u64>>`) também compilam, mas copiam o vetor inteiro só para satisfazer o `'static`.

`maximo_paralelo` segue o mesmo molde. Cada pedaço devolve `Option<i64>` (o `max` de um iterador), e o resultado final é o `max` dos `Some`:

```rust
handles.into_iter().filter_map(|h| h.join().unwrap()).max()
```

Com `dados` vazio, `chunks` não produz nenhum pedaço, nenhuma thread é criada e o resultado é `None` (ou `0` na soma). O `.max(1)` em `tamanho` evita o *panic* de `chunks(0)`.

## 2. Contagem de palavras com canal

Projeto: [`ex2-contagem-canal/`](ex2-contagem-canal/src/lib.rs).

```rust
let workers = workers.max(1);
let mut lotes: Vec<Vec<String>> = vec![Vec::new(); workers];
for (i, t) in textos.into_iter().enumerate() {
    lotes[i % workers].push(t);
}

let (tx, rx) = mpsc::channel();
for lote in lotes {
    let tx = tx.clone();
    thread::spawn(move || {
        let mut parcial = HashMap::new();
        for texto in &lote {
            for (p, n) in contar(texto) {
                *parcial.entry(p).or_insert(0) += n;
            }
        }
        tx.send(parcial).unwrap();
    });
}
drop(tx);
```

Cada thread é dona do seu lote (movido para ela) e do seu mapa parcial. Nada é compartilhado: o `HashMap` parcial é **movido** pelo canal para a thread que chamou, que soma os mapas num `for parcial in rx`. Por isso não há `Mutex` nem `Arc`.

O `drop(tx)` é o que faz o `for` terminar: depois dele, os únicos `Sender` vivos são os clones das threads, e cada um morre quando a sua thread acaba. Sem ele, o teste trava. Os `JoinHandle` não são guardados: o canal já serve de sincronização, porque o `for` só acaba depois que todas as threads mandaram o seu mapa e terminaram.

Com mais workers que textos (o teste com `w = 10`), alguns lotes ficam vazios e mandam um mapa vazio, o que não muda o total.

## 3. Estoque compartilhado

Projeto: [`ex3-estoque/`](ex3-estoque/src/lib.rs).

O teste com threads não compila:

```text
error[E0277]: `Rc<RefCell<HashMap<String, u32>>>` cannot be sent between threads safely
   --> src/lib.rs:57:40
    |
 57 |               handles.push(thread::spawn(move || {
    |                            ------------- ^------
...
 59 | |             }));
    | |_____________^ `Rc<RefCell<HashMap<String, u32>>>` cannot be sent between threads safely
```

Pela tabela da lição 02, `Rc<RefCell<T>>` vira `Arc<Mutex<T>>`. O `#[derive(Clone, Default)]` continua funcionando: clonar o `Estoque` clona o `Arc`, e os clones compartilham o mesmo mapa.

A troca direta (`borrow_mut()` → `lock().unwrap()`) compila, e o teste `item_desconhecido_e_cadastrado` trava. O culpado é o braço `None`:

```rust
match self.itens.lock().unwrap().get_mut(item) {
    // ...
    None => {
        self.itens.lock().unwrap().insert(item.to_string(), 0); // espera para sempre
        false
    }
}
```

O guarda criado na expressão do `match` é um temporário que vive até o fim do `match` inteiro, inclusive durante o braço `None`. O segundo `lock()` espera o primeiro ser solto, o que nunca acontece (lição 04, "Travar duas vezes na mesma thread"). Com o `RefCell` original, o mesmo braço daria *panic* (`RefCell already borrowed`), mas o código nunca chegou a rodar com threads.

A correção trava uma vez e usa o mesmo guarda em tudo:

```rust
let mut itens = self.itens.lock().unwrap();
match itens.get_mut(item) {
    Some(n) if *n >= qtd => {
        *n -= qtd;
        true
    }
    Some(_) => false,
    None => {
        itens.insert(item.to_string(), 0);
        false
    }
}
```

O empréstimo de `itens.get_mut(item)` acaba antes do braço `None` (não há referência viva ali), então o `insert` no mesmo guarda é permitido.

A outra correção do tipo "copie e solte antes de decidir" (`let atual = self.itens.lock().unwrap().get(item).copied();` e depois travar de novo para subtrair) resolve o deadlock, mas **quebra o teste com 8 threads** de vez em quando: entre a leitura e a escrita, outra thread pode reservar a mesma unidade. Duas threads leem `1`, as duas decidem que há estoque, as duas subtraem. Não é uma data race (cada acesso está protegido), é uma *race condition* (lição 02). A verificação e a alteração precisam acontecer com o mesmo lock.

## 4. Métricas sem lock

Projeto: [`ex4-metricas/`](ex4-metricas/src/lib.rs).

```rust
static PROXIMO_ID: AtomicU64 = AtomicU64::new(1);

pub fn proximo_id() -> u64 {
    PROXIMO_ID.fetch_add(1, Ordering::Relaxed)
}

pub fn registrar(&self, bytes: u64) {
    self.requisicoes.fetch_add(1, Ordering::Relaxed);
    self.bytes.fetch_add(bytes, Ordering::Relaxed);
    self.maior.fetch_max(bytes, Ordering::Relaxed);
}
```

`fetch_add` devolve o valor **anterior**, que é exatamente o próximo id (começando em 1). `fetch_max` faz "grave o maior entre o atual e este" de forma atômica; com `load` + comparação + `store`, duas threads poderiam ler o mesmo máximo antigo e a maior requisição se perderia. As leituras são `load(Ordering::Relaxed)`.

`Relaxed` basta em tudo: cada atômico é um dado independente, e nenhum deles avisa que **outro** dado está pronto (lição 05). Um efeito disso: uma thread lendo as métricas enquanto outras registram pode ver `requisicoes` já incrementado e `bytes` ainda não. Se as três precisassem ser vistas sempre consistentes entre si, a resposta seria um `Mutex<(u64, u64, u64)>`.

A tabela de status é um `LazyLock`, porque um `HashMap` precisa alocar:

```rust
static STATUS: LazyLock<HashMap<u16, &'static str>> = LazyLock::new(|| {
    HashMap::from([
        (200, "OK"),
        (201, "Created"),
        (404, "Not Found"),
        (500, "Internal Server Error"),
    ])
});

pub fn status_http(codigo: u16) -> Option<&'static str> {
    STATUS.get(&codigo).copied()
}
```

`Metricas::new` é `const fn`, por isso `GLOBAL` pode ser um `static` comum, sem `LazyLock`. Sobre a pergunta da dica: o `cargo test` roda os testes em threads paralelas, no mesmo processo, e todos veem o mesmo `GLOBAL` (e o mesmo `PROXIMO_ID`). Um teste não pode supor que é o único a mexer num `static`. É o motivo do `>= 1` e de o teste de ids conferir só que são únicos, não que são `1..=8000`.

## 5. Paralelismo com rayon

Projeto: [`ex5-rayon/`](ex5-rayon/src/lib.rs).

```rust
pub fn primos_ate(n: u64) -> Vec<u64> {
    (0..n).into_par_iter().filter(|&x| eh_primo(x)).collect()
}

pub fn maior_collatz(limite: u64) -> Option<(u64, u32)> {
    (1..limite)
        .into_par_iter()
        .map(|n| (n, passos_collatz(n)))
        .max_by_key(|&(n, p)| (p, Reverse(n)))
}

pub fn soma_quadrados(v: &[i64]) -> i64 {
    v.par_iter().map(|x| x * x).sum()
}
```

`collect` preserva a ordem do intervalo, então os primos já saem em ordem crescente, sem `sort`.

O desempate do `maior_collatz` é o ponto delicado. Com `max_by_key(|&(_, p)| p)`, qual dos empatados ganha depende de detalhes da implementação, e numa versão paralela isso é fácil de errar. A chave `(p, Reverse(n))` torna o resultado único: maior número de passos e, entre esses, o maior `Reverse(n)`, que é o **menor** `n`. Assim a resposta é a mesma com 1 thread ou com 10. `passos_collatz` é um laço simples (`n / 2` se par, `3 * n + 1` se ímpar, contando os passos até 1).

Na máquina de teste (10 núcleos, compilado em `--release`), `maior_collatz(10_000_000)` devolveu `Some((8400511, 685))` em 1,6 s com `RAYON_NUM_THREADS=1` e em 0,2 s sem a variável. Os números exatos dependem do seu processador.

# 01 — Threads: `spawn`, `join`, `move` e `scope`

**Objetivos.** Ao final, você vai saber:

- criar threads com `thread::spawn`, esperar por elas com `join` e receber o valor que devolvem;
- entender por que `spawn` exige `move` e `'static`, e o que esses dois requisitos significam de verdade;
- tratar o *panic* de uma thread sem derrubar o programa;
- usar `thread::scope` para emprestar dados locais a várias threads, sem `Arc` e sem cópias.

## Criar uma thread e esperar por ela

Uma *thread* (linha de execução) roda uma função ao mesmo tempo que o resto do programa. Em Rust, as threads da std são threads do sistema operacional, como `std::thread` no C++ e `threading.Thread` no Python. `thread::spawn` recebe uma closure e devolve um `JoinHandle`:

```rust
use std::thread;
use std::time::Duration;

fn main() {
    let h = thread::spawn(|| {
        for i in 1..=5 {
            println!("thread: {i}");
            thread::sleep(Duration::from_millis(1));
        }
    });
    for i in 1..=2 {
        println!("main: {i}");
        thread::sleep(Duration::from_millis(1));
    }
    h.join().unwrap();
    println!("fim");
}
```

```text
main: 1
thread: 1
main: 2
thread: 2
thread: 3
thread: 4
thread: 5
fim
```

Esta foi **uma** execução. Quem decide qual thread roda quando é o sistema operacional, então a ordem das linhas de `main` e `thread` pode mudar de uma execução para outra. Isso vale para quase toda saída deste módulo: quando a ordem depender do escalonador, o texto avisa.

`h.join()` bloqueia até a thread terminar. Sem ele, o que acontece quando `main` acaba primeiro? O processo termina, e as outras threads morrem onde estiverem. Tirando o `join` do exemplo acima, três execuções seguidas deram isto (duas vezes):

```text
main: 1
thread: 1
main: 2
thread: 2
thread: 
```

A thread morreu no meio de um `println!`, depois de escrever `thread: ` e antes do número. Não há aviso nem erro: as linhas 3 a 5 simplesmente não aparecem.

> [!NOTE]
> Em C++, destruir um `std::thread` sem ter chamado `join()` ou `detach()` chama `std::terminate` e derruba o programa. Em Rust, descartar o `JoinHandle` é o equivalente a `detach()`: a thread continua solta até o processo acabar. Se você precisa do resultado, guarde o handle e chame `join`.

### A thread pode devolver um valor

`JoinHandle<T>` carrega o tipo de retorno da closure. O `join` devolve `Result<T, ...>`:

```rust
let nomes = [String::from("Ana"), String::from("Bia")];
let h = thread::spawn(move || nomes.iter().map(|n| n.len()).sum::<usize>());
println!("total de letras: {}", h.join().unwrap()); // total de letras: 6
```

Não é preciso uma variável compartilhada para "pegar o resultado": o valor volta pelo handle, com o tipo certo.

## Por que `move`: a thread pode viver mais que a função

O Módulo 07 mostrou este erro de passagem. Agora dá para entender a causa:

```rust
// ❌ NÃO COMPILA
use std::thread;

fn main() {
    let nomes = vec!["Ana", "Bia"];
    let h = thread::spawn(|| {
        println!("{nomes:?}");
    });
    h.join().unwrap();
}
```

```text
error[E0373]: closure may outlive the current function, but it borrows `nomes`, which is owned by the current function
 --> src/main.rs:5:27
  |
5 |     let h = thread::spawn(|| {
  |                           ^^ may outlive borrowed value `nomes`
6 |         println!("{nomes:?}");
  |                    ----- `nomes` is borrowed here
  |
note: function requires argument type to outlive `'static`
...
help: to force the closure to take ownership of `nomes` (and any other referenced variables), use the `move` keyword
  |
5 |     let h = thread::spawn(move || {
  |                           ++++
```

A closure só lê `nomes`, então por padrão ela **empresta** (Módulo 07). Mas o compilador não sabe que o `join` vem logo depois: para ele, a thread pode continuar rodando depois que `main` retornar e `nomes` for liberado. Um empréstimo que pode sobreviver ao dono é exatamente o que o *borrow checker* proíbe.

Com `move`, a closure leva a posse de `nomes` para dentro da thread. Daí em diante, quem libera o vetor é a thread. O preço: `main` não pode mais usá-lo.

```rust
// ❌ NÃO COMPILA
let nomes = vec!["Ana", "Bia"];
let h = thread::spawn(move || {
    println!("na thread: {nomes:?}");
});
println!("no main: {nomes:?}");
```

```text
error[E0382]: borrow of moved value: `nomes`
 --> src/main.rs:8:25
  |
4 |     let nomes = vec!["Ana", "Bia"];
  |         ----- move occurs because `nomes` has type `Vec<&str>`, which does not implement the `Copy` trait
5 |     let h = thread::spawn(move || {
  |                           ------- value moved into closure here
...
8 |     println!("no main: {nomes:?}");
  |                         ^^^^^ value borrowed here after move
  |
help: consider cloning the value before moving it into the closure
```

É o *move* do Módulo 02, sem nada de novo. As saídas: clonar antes do `move` (a receita do Módulo 07), compartilhar com `Arc` (lição 02) ou usar `thread::scope` (mais abaixo), quando a thread não precisa sobreviver à função.

### O que `'static` quer dizer aqui

A assinatura de `spawn` diz por que o `move` é necessário:

```rust
pub fn spawn<F, T>(f: F) -> JoinHandle<T>
where
    F: FnOnce() -> T + Send + 'static,
    T: Send + 'static,
```

`Send` é assunto da próxima lição. `F: 'static` **não** quer dizer que a closure vive o programa todo. Como o Módulo 03 adiantou, quer dizer que ela **não contém empréstimos temporários**. Uma `String` movida para dentro satisfaz o requisito, mesmo sendo liberada no fim da thread. Uma `&str` apontando para uma `String` local, não:

```rust
// ❌ NÃO COMPILA
let nome = String::from("Ana");
let r: &str = &nome;
let h = thread::spawn(move || println!("{r}"));
h.join().unwrap();
```

```text
error[E0597]: `nome` does not live long enough
   --> src/main.rs:5:19
    |
  4 |     let nome = String::from("Ana");
    |         ---- binding `nome` declared here
  5 |     let r: &str = &nome;
    |                   ^^^^^ borrowed value does not live long enough
  6 |     let h = thread::spawn(move || println!("{r}"));
    |             -------------------------------------- argument requires that `nome` is borrowed for `'static`
...
```

O `move` moveu a **referência** `r` para a thread, mas a referência continua apontando para `nome`, que é de `main`. O `move` não transforma empréstimos em posse.

## Quando uma thread entra em pânico

O Módulo 05 disse que um *panic* encerra **a thread** em que aconteceu. Numa thread criada por você, isso não derruba o programa: o `join` devolve `Err`.

```rust
let h = thread::spawn(|| {
    let v: Vec<i32> = Vec::new();
    v[0]
});
match h.join() {
    Ok(x) => println!("deu {x}"),
    Err(_) => println!("a thread entrou em pânico, e o main continua"),
}
```

```text
thread '<unnamed>' (93539) panicked at src/main.rs:46:10:
index out of bounds: the len is 0 but the index is 0
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
a thread entrou em pânico, e o main continua
```

A mensagem de pânico vai para o stderr, como sempre. O `Err` carrega o valor do pânico como `Box<dyn Any + Send>`; para ler a mensagem, tente `e.downcast_ref::<String>()` e depois `e.downcast_ref::<&str>()`. O `<unnamed>` é o nome da thread. Para dar um nome, que aparece nas mensagens de pânico e nos *profilers*, use `thread::Builder`:

```rust
let h = thread::Builder::new()
    .name("contador".into())
    .spawn(|| format!("rodando em '{}'", thread::current().name().unwrap()))
    .unwrap();
println!("{}", h.join().unwrap()); // rodando em 'contador'
```

O `.unwrap()` em `h.join()` repassa o pânico da thread para quem chamou. É o comportamento certo na maioria dos casos: um bug numa thread vira um bug visível no main. Se uma thread estava segurando um *lock* quando entrou em pânico, o lock fica "envenenado"; isso está na lição 04.

## `thread::scope`: threads que emprestam

Muitas vezes a thread **não** precisa viver mais que a função: você quer dividir um trabalho, esperar todo mundo e seguir. Para esse caso, `thread::scope` (desde a 1.63) elimina o requisito de `'static`:

```rust
fn soma_paralela(dados: &[u64], partes: usize) -> u64 {
    let tamanho = dados.len().div_ceil(partes.max(1)).max(1);
    thread::scope(|s| {
        let handles: Vec<_> = dados
            .chunks(tamanho)
            .map(|pedaco| s.spawn(move || pedaco.iter().sum::<u64>()))
            .collect();
        handles.into_iter().map(|h| h.join().unwrap()).sum()
    })
}
```

O `scope` garante que **todas** as threads criadas com `s.spawn` terminam antes de ele retornar (o que não tiver recebido `join` é esperado automaticamente no fim). Com essa garantia, o compilador sabe que os empréstimos de `dados` não sobrevivem ao dono, e as threads podem receber `&[u64]` direto. Nada é copiado, nada precisa de `Arc`.

O `move` dentro do `map` ainda aparece, mas por outro motivo: ele move a **referência** `pedaco` (que é `Copy`) para a closure, em vez de emprestar a variável `pedaco`, que deixa de existir a cada volta do `map`.

As regras de empréstimo continuam valendo dentro do escopo. Duas threads não podem ter `&mut` para o mesmo valor:

```rust
// ❌ NÃO COMPILA
let mut total = 0;
thread::scope(|s| {
    s.spawn(|| total += 1);
    s.spawn(|| total += 2);
});
```

```text
error[E0499]: cannot borrow `total` as mutable more than once at a time
   --> src/main.rs:7:17
    |
  6 |         s.spawn(|| total += 1);
    |         ----------------------
    |         |       |  |
    |         |       |  first borrow occurs due to use of `total` in closure
    |         |       first mutable borrow occurs here
    |         argument requires that `total` is borrowed for `'1`
  7 |         s.spawn(|| total += 2);
    |                 ^^ ----- second borrow occurs due to use of `total` in closure
    |                 |
    |                 second mutable borrow occurs here
```

É o mesmo E0499 do Módulo 03. "Muitos `&` ou um `&mut`" é exatamente a regra que impede duas threads de escreverem no mesmo lugar ao mesmo tempo. Para escrever em paralelo, dê a cada thread um pedaço **diferente**. `chunks_mut` faz isso, e o compilador aceita porque os pedaços não se sobrepõem:

```rust
fn escalar_paralelo(dados: &mut [i32], fator: i32, partes: usize) {
    let tamanho = dados.len().div_ceil(partes.max(1)).max(1);
    thread::scope(|s| {
        for pedaco in dados.chunks_mut(tamanho) {
            s.spawn(move || {
                for x in pedaco {
                    *x *= fator;
                }
            });
        }
    });
}
```

```text
escalado: [10, 20, 30, 40, 50, 60, 70]
```

Quando várias threads precisam alterar o **mesmo** valor, é preciso sincronização: `Mutex` (lição 04) ou atômicos (lição 05). Código completo em [`codigo/threads-e-scope`](codigo/threads-e-scope/src/main.rs).

> [!TIP]
> Quantas threads criar? Para trabalho de CPU, mais threads que núcleos não ajuda. `thread::available_parallelism()` devolve o número que o sistema oferece. Para paralelizar coleções, a lição 07 apresenta o `rayon`, que já cuida disso.

## E o Python?

Em CPython, `threading.Thread` também cria threads do sistema, mas o *GIL* (*Global Interpreter Lock*) deixa só uma delas executar bytecode Python por vez. Threads ajudam com E/S (enquanto uma espera a rede, outra roda), mas não aceleram cálculo; para isso existe o `multiprocessing`, que usa processos separados. Há uma versão do CPython sem GIL (*free-threaded*, PEP 703) desde o 3.13, ainda opcional.

Em Rust não há GIL: as threads rodam em paralelo de verdade, em núcleos diferentes. É por isso que o compilador precisa ser rigoroso sobre o que pode ser compartilhado, o assunto da próxima lição.

## Armadilhas comuns

- **Esquecer o `join`.** O programa termina e as threads morrem no meio, sem aviso. Guarde os handles num `Vec` e faça `join` em todos, ou use `thread::scope`.
- **Achar que `move` copia.** `move` move a posse (ou copia, se o tipo for `Copy`). Se você ainda precisa do valor depois, clone antes.
- **Pôr `move` e ainda assim levar uma referência.** Mover um `&T` para a thread não resolve o E0597: a referência continua presa ao dono original. Mova o dono, ou use `scope`.
- **Contar com a ordem das mensagens.** Nada garante que a thread criada primeiro imprima primeiro. Se a ordem importa, ela precisa vir do seu código (por exemplo, juntando os resultados na ordem dos handles, como em `soma_paralela`).

## Resumo

- `thread::spawn(closure)` cria uma thread do sistema; `join()` espera e devolve `Result<T, _>` com o retorno da closure.
- Quando `main` termina, o processo acaba e as outras threads morrem onde estiverem.
- `spawn` exige `F: Send + 'static`: a closure não pode ter empréstimos de variáveis locais. Use `move` para levar a posse.
- Um *panic* numa thread vira `Err` no `join`; o resto do programa continua.
- `thread::scope` espera todas as threads no fim, e por isso permite emprestar dados locais; `chunks_mut` divide um `&mut` entre threads.

## Exercícios

Veja o exercício 1 em [`exercicios.md`](exercicios.md).

## Leitura complementar

- [The Rust Programming Language, 16.1 Using Threads to Run Code Simultaneously](https://doc.rust-lang.org/stable/book/ch16-01-threads.html)
- [Documentação de `std::thread`](https://doc.rust-lang.org/std/thread/index.html) e de [`thread::scope`](https://doc.rust-lang.org/std/thread/fn.scope.html)
- [Comprehensive Rust: Threads](https://google.github.io/comprehensive-rust/concurrency/threads.html)
- [Rust Atomics and Locks, cap. 1](https://marabos.nl/atomics/basics.html) (Mara Bos; gratuito online)

---

[← Índice do módulo](README.md) · [Próxima lição →](02-send-e-sync.md)

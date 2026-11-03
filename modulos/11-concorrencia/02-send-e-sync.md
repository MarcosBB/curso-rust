# 02 — `Send` e `Sync`: como o compilador impede *data races*

**Objetivos.** Ao final, você vai saber:

- o que é uma *data race* e por que ela é *undefined behavior* em C++;
- o que as traits `Send` e `Sync` dizem sobre um tipo, e quais tipos da std têm cada uma;
- ler os erros E0277 de `Send`/`Sync` e escolher a troca certa (`Rc` → `Arc`, `RefCell` → `Mutex`);
- declarar erros que atravessam threads (`Box<dyn Error + Send + Sync>`).

## O problema: duas threads, um contador

Uma *data race* acontece quando duas threads acessam o mesmo endereço ao mesmo tempo, pelo menos uma escreve, e nada sincroniza os acessos. Em C++, este programa tem uma:

```cpp
#include <iostream>
#include <thread>
#include <vector>

int contador = 0;

int main() {
    std::vector<std::thread> ts;
    for (int i = 0; i < 8; ++i)
        ts.emplace_back([] {
            for (int j = 0; j < 100000; ++j) contador++;
        });
    for (auto& t : ts) t.join();
    std::cout << contador << "\n";
}
```

Oito threads, cem mil incrementos cada: o esperado é 800000. Três execuções (clang, `-O0`):

```text
258547
285687
172723
```

`contador++` é "ler, somar, gravar". Duas threads leem o mesmo valor, as duas somam um, as duas gravam, e um incremento se perde. Mas o problema é maior que números errados: pelo padrão do C++, uma data race é *undefined behavior* (UB). O compilador pode assumir que ela não acontece e otimizar com base nisso, e aí qualquer coisa vale. Com `-fsanitize=thread`, o ThreadSanitizer acusa em tempo de execução:

```text
WARNING: ThreadSanitizer: data race (pid=7368)
  Write of size 4 at 0x000104c10000 by thread T4:
    #0 main::$_0::operator()() const corrida.cpp:11
...
  Previous write of size 4 at 0x000104c10000 by thread T3:
```

O sanitizador ajuda, mas só encontra a corrida se ela acontecer durante o teste. Em Rust, o equivalente **não compila**, e a explicação são duas traits.

## As duas traits

- **`Send`**: um valor do tipo `T` pode ser **movido** para outra thread.
- **`Sync`**: um `&T` pode ser **compartilhado** entre threads. Formalmente, `T: Sync` se e somente se `&T: Send`.

As duas são *marker traits* (traits marcadoras): não têm métodos, só dizem algo sobre o tipo. E são *auto traits*: o compilador as implementa sozinho para uma struct ou enum quando **todos os campos** as implementam. Um `struct Ponto { x: f64, y: f64 }` é `Send` e `Sync` sem você escrever nada. Uma struct com um campo `Rc` deixa de ser `Send`, também sem você escrever nada.

O `thread::spawn` exige `F: Send`, e a closure só é `Send` se tudo o que ela captura for `Send`. É assim que a checagem chega até o seu tipo.

Quase tudo é `Send` e `Sync`: números, `String`, `Vec<T>` e `HashMap` (se o `T` for), `Box<T>`, referências `&T` para tipos `Sync`. As exceções são os tipos que fazem algo sem sincronização por baixo:

| Tipo | `Send` | `Sync` | Por quê | Em várias threads, use |
|---|---|---|---|---|
| `Rc<T>` | não | não | contador de referências não atômico | `Arc<T>` |
| `Cell<T>` | sim | não | altera através de `&` sem trava | atômicos (`AtomicU32`...) |
| `RefCell<T>` | sim | não | contador de empréstimos não atômico | `Mutex<T>` ou `RwLock<T>` |
| `MutexGuard<'_, T>` | não | sim | alguns sistemas exigem que quem trava destrave | soltar o guarda antes |
| ponteiros crus `*const T` | não | não | o compilador não sabe o que eles apontam | encapsular (Módulo 15) |
| `Arc<T>` | se `T: Send + Sync` | se `T: Send + Sync` | compartilha o `T` entre threads | — |
| `Mutex<T>` | se `T: Send` | se `T: Send` | a trava garante acesso exclusivo | — |

`Cell` e `RefCell` são `Send`: mover um `RefCell` inteiro para outra thread é seguro, porque só uma thread fica com ele. O que não pode é **compartilhar**: duas threads com `&RefCell` mexeriam no contador de empréstimos ao mesmo tempo.

## Lendo os erros

### `Rc` não é `Send`

O Módulo 10 adiantou este:

```rust
// ❌ NÃO COMPILA
use std::rc::Rc;
use std::thread;

fn main() {
    let dados = Rc::new(vec![1, 2, 3]);
    let copia = Rc::clone(&dados);
    let h = thread::spawn(move || println!("{copia:?}"));
    h.join().unwrap();
}
```

```text
error[E0277]: `Rc<Vec<i32>>` cannot be sent between threads safely
   --> src/main.rs:7:27
    |
  7 |     let h = thread::spawn(move || println!("{copia:?}"));
    |             ------------- -------^^^^^^^^^^^^^^^^^^^^^^
    |             |             |
    |             |             `Rc<Vec<i32>>` cannot be sent between threads safely
    |             |             within this `{closure@src/main.rs:7:27: 7:34}`
    |             required by a bound introduced by this call
    |
    = help: within `{closure@src/main.rs:7:27: 7:34}`, the trait `Send` is not implemented for `Rc<Vec<i32>>`
note: required because it's used within this closure
...
note: required by a bound in `spawn`
...
128 |     F: Send + 'static,
    |        ^^^^ required by this bound in `spawn`
```

Leia de baixo para cima: `spawn` exige `F: Send`; a closure não é `Send` porque captura `copia`; `copia` é um `Rc<Vec<i32>>`, que não é `Send`. Se o `Rc` pudesse ir, as duas threads fariam `strong += 1` e `strong -= 1` sem sincronização, que é a mesma corrida do contador em C++. Um incremento perdido aqui libera a memória cedo demais (*use after free*) ou nunca.

A troca é `Arc<T>` (*Atomically Reference Counted*): mesma interface, contador atômico.

```rust
use std::sync::Arc;

let dados = Arc::new(vec![1, 2, 3]);
let copia = Arc::clone(&dados);
let h = thread::spawn(move || println!("na thread: {copia:?}"));
h.join().unwrap();
println!("no main: {dados:?}");
```

Cada `Arc::clone` faz um incremento atômico, um pouco mais caro que o do `Rc`. Por isso a std tem os dois, e o compilador escolhe por você quando importa. O `shared_ptr` do C++ é sempre atômico.

### `Arc` compartilha, mas não deixa alterar

`Arc<T>`, como `Rc<T>`, só dá `&T`:

```text
error[E0596]: cannot borrow data in an `Arc` as mutable
 --> src/main.rs:7:35
  |
7 |     let h = thread::spawn(move || copia.push(4));
  |                                   ^^^^^ cannot borrow as mutable
  |
  = help: trait `DerefMut` is required to modify through a dereference, but it is not implemented for `Arc<Vec<i32>>`
```

No Módulo 10, a saída foi `Rc<RefCell<T>>`. A tentação é trocar só o `Rc`:

```rust
// ❌ NÃO COMPILA
let dados = Arc::new(RefCell::new(vec![1, 2, 3]));
let copia = Arc::clone(&dados);
let h = thread::spawn(move || copia.borrow_mut().push(4));
```

```text
error[E0277]: `RefCell<Vec<i32>>` cannot be shared between threads safely
   --> src/main.rs:8:27
    |
  8 |     let h = thread::spawn(move || copia.borrow_mut().push(4));
    |             ------------- ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ `RefCell<Vec<i32>>` cannot be shared between threads safely
    |             |
    |             required by a bound introduced by this call
    |
    = help: the trait `Sync` is not implemented for `RefCell<Vec<i32>>`
    = note: if you want to do aliasing and mutation between multiple threads, use `std::sync::RwLock` instead
    = note: required for `Arc<RefCell<Vec<i32>>>` to implement `Send`
```

Agora o erro é de `Sync`: o `Arc` só é `Send` se o conteúdo puder ser compartilhado (`T: Sync`), e o `RefCell` não pode. O próprio compilador sugere a troca: `RwLock`, ou o mais comum, `Mutex`. A receita é a tabela abaixo, que você vai usar o módulo inteiro:

| Uma thread (Módulo 10) | Várias threads (este módulo) |
|---|---|
| `Rc<T>` | `Arc<T>` |
| `RefCell<T>` | `Mutex<T>`, `RwLock<T>` (lição 04) |
| `Cell<T>` (números, flags) | `AtomicU64`, `AtomicBool`... (lição 05) |
| `OnceCell<T>` | `OnceLock<T>` (lição 06) |
| `Rc<RefCell<T>>` | `Arc<Mutex<T>>` |

### Erros que atravessam threads

O Módulo 05 avisou que, se um erro precisar atravessar threads, o tipo usado é `Box<dyn Error + Send + Sync>`. Este é o motivo:

```rust
// ❌ NÃO COMPILA
use std::error::Error;
use std::thread;

fn ler_numero(s: &str) -> Result<i32, Box<dyn Error>> {
    Ok(s.parse::<i32>()?)
}

fn main() {
    let h = thread::spawn(|| ler_numero("42"));
    println!("{:?}", h.join().unwrap());
}
```

```text
error[E0277]: `dyn std::error::Error` cannot be sent between threads safely
   --> src/main.rs:9:13
    |
  9 |     let h = thread::spawn(|| ler_numero("42"));
    |             ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ `dyn std::error::Error` cannot be sent between threads safely
    |
    = help: the trait `Send` is not implemented for `dyn std::error::Error`
...
note: required by a bound in `spawn`
...
129 |     T: Send + 'static,
    |        ^^^^ required by this bound in `spawn`
```

Agora quem falha é o `T: Send` (o valor devolvido pela thread). Um `dyn Error` pode esconder qualquer tipo, inclusive um com `Rc` dentro, então o compilador não pode supor que ele é `Send`. A correção é prometer na assinatura: `Result<i32, Box<dyn Error + Send + Sync>>`. Os tipos de erro da std (`ParseIntError`, `io::Error`...) satisfazem as duas, e o `?` continua funcionando. Crates como `anyhow` já usam `Send + Sync` por padrão pelo mesmo motivo.

### O guarda de um lock fica na thread que travou

```rust
// ❌ NÃO COMPILA
let m = Mutex::new(0);
let guarda = m.lock().unwrap();
thread::scope(|s| {
    s.spawn(move || println!("{}", *guarda));
});
```

```text
error[E0277]: `std::sync::MutexGuard<'_, i32>` cannot be sent between threads safely
...
    = help: within `{closure@src/main.rs:8:17: 8:24}`, the trait `Send` is not implemented for `std::sync::MutexGuard<'_, i32>`
```

Em alguns sistemas, só a thread que travou um mutex pode destravá-lo, e o guarda destrava no `Drop`. Por isso ele não pode mudar de thread. Na prática, isso raramente aparece; quando aparecer, a solução é copiar o valor e soltar o guarda antes.

## Data race x race condition

`Send` e `Sync` eliminam **data races**: dois acessos sem sincronização ao mesmo dado. Eles não eliminam toda *race condition* (condição de corrida), que é qualquer resultado que depende da ordem das threads. Duas threads que fazem "travar, ler o saldo, soltar; travar de novo, gravar saldo − 10, soltar" não têm data race (cada acesso está protegido), mas as duas podem ler o mesmo saldo e uma das retiradas se perde. *Deadlocks* (lição 04) também continuam possíveis. O que o compilador garante é que o seu programa nunca chega ao UB; a lógica correta ainda é sua.

## E o Python?

O GIL do CPython evita que duas threads corrompam o próprio interpretador (os contadores de referência dos objetos, por exemplo). Ele não torna o seu código seguro: `contador += 1` são várias instruções de bytecode, e a troca de thread pode cair no meio. O resultado é o mesmo incremento perdido do C++, só que sem UB. Para dados compartilhados, você usa `threading.Lock` por convenção, e nada avisa se esquecer.

Rust faz o caminho oposto: não há trava global, as threads rodam em paralelo, e a falta de sincronização é um erro de **tipo**, encontrado na compilação.

## Armadilhas comuns

- **Implementar `Send`/`Sync` à mão para calar o erro.** `unsafe impl Send for MeuTipo {}` compila, e a partir daí a garantia é sua. Só faz sentido para tipos que encapsulam ponteiros crus com sincronização correta (Módulo 15). No código de aplicação, a resposta é trocar o tipo.
- **`Arc<RefCell<T>>`.** É o `Rc<RefCell<T>>` traduzido pela metade. O par certo é `Arc<Mutex<T>>`.
- **Ler só a primeira linha do E0277.** A cadeia de `note: required because...` diz qual campo, de qual struct, quebrou o `Send`. Comece pela última nota e suba.

## Resumo

- Data race: acesso simultâneo, sem sincronização, com pelo menos uma escrita. Em C++, é UB; em Rust seguro, não compila.
- `Send`: pode ser movido para outra thread. `Sync`: `&T` pode ser compartilhado. São auto traits, deduzidas dos campos.
- `Rc` (não `Send`) vira `Arc`; `RefCell`/`Cell` (não `Sync`) viram `Mutex`/`RwLock`/atômicos.
- Erros devolvidos por threads precisam de `Box<dyn Error + Send + Sync>`.
- O compilador impede data races, não toda race condition nem deadlocks.

## Exercícios

Veja o exercício 3 em [`exercicios.md`](exercicios.md).

## Leitura complementar

- [The Rust Programming Language, 16.4 Extensible Concurrency with `Send` and `Sync`](https://doc.rust-lang.org/stable/book/ch16-04-extensible-concurrency-sync-and-send.html)
- [Rustonomicon: Send and Sync](https://doc.rust-lang.org/nomicon/send-and-sync.html) e [Data Races and Race Conditions](https://doc.rust-lang.org/nomicon/races.html)
- [Documentação de `Send`](https://doc.rust-lang.org/std/marker/trait.Send.html) e de [`Sync`](https://doc.rust-lang.org/std/marker/trait.Sync.html)
- [Comprehensive Rust: Send and Sync](https://google.github.io/comprehensive-rust/concurrency/send-sync.html)

---

[← Lição anterior](01-threads-e-scope.md) · [Índice do módulo](README.md) · [Próxima lição →](03-canais.md)

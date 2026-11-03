# 04 — Comportamento indefinido e Miri

**Objetivos.** Ao final, você vai saber:

- o que é comportamento indefinido (UB) e por que "rodou certo" não prova nada;
- reconhecer os casos mais comuns: ponteiro pendurado, fora dos limites, dois `&mut`, valor inválido, double free, memória não inicializada, data race;
- instalar e rodar o Miri, ler o relatório dele e conhecer seus limites;
- comparar o Miri com os *sanitizers* do C++.

## O que é UB

*Comportamento indefinido* (*undefined behavior*, UB) é uma operação para a qual a linguagem não define resultado nenhum. Não é "dá um valor errado" nem "dá crash": é **qualquer coisa**. O compilador otimiza supondo que UB nunca acontece. Se acontece, as suposições ficam falsas, e o programa pode imprimir lixo, travar, funcionar por anos e quebrar com outra versão do compilador, ou fazer algo que nem está no código-fonte.

A Rust Reference mantém a lista do que é UB. Os itens principais:

| UB | Exemplo |
|---|---|
| data race | duas threads, uma escrevendo, sem sincronização |
| acessar um lugar pendurado ou desalinhado | ponteiro para variável que saiu de escopo, `*(p as *const u32)` em endereço ímpar |
| aritmética de ponteiro fora da alocação | `p.add(10)` num array de 3 |
| quebrar as regras de aliasing | dois `&mut` vivos para o mesmo dado; mudar algo através de `&T` sem `UnsafeCell` |
| produzir um valor inválido | `bool` diferente de 0/1, `char` fora do Unicode, referência nula, enum com tag inexistente, inteiro não inicializado |
| *unwind* por uma função que não permite | panic escapando de `extern "C"` (lição 06) |
| chamar uma função com a ABI errada | declarar em Rust uma assinatura diferente da de C (lição 05) |

E o que **não** é UB, mesmo sendo bug: vazamento de memória, deadlock, overflow de inteiro (dá panic em debug e dá a volta em release, de forma definida), panic, e erros de lógica. Por isso `mem::forget` e `Rc` em ciclo são seguros (Módulo 02).

## "Funcionou" não prova nada

Os programas abaixo compilam (o UB está dentro de `unsafe`) e, rodando normalmente, quase todos "funcionam". Para cada um, a tabela mostra o que aconteceu em `cargo run` e em `cargo run --release`, e a seção seguinte mostra o que o Miri diz.

```rust
// 1. ponteiro pendurado
fn main() {
    let p: *const i32;
    {
        let x = 42;
        p = &raw const x;
    }
    println!("{}", unsafe { *p });
}
```

```rust
// 2. leitura fora dos limites
fn main() {
    let v = vec![1, 2, 3];
    let p = v.as_ptr();
    let quarto = unsafe { *p.add(3) };
    println!("{quarto}");
}
```

```rust
// 3. dois &mut para o mesmo dado
fn main() {
    let mut x = 0;
    let p = &raw mut x;
    let a = unsafe { &mut *p };
    let b = unsafe { &mut *p };
    *a += 1;
    *b += 1;
    println!("{x}");
}
```

```rust
// 4. valor inválido
fn main() {
    let byte: u8 = 2;
    let b: bool = unsafe { std::mem::transmute(byte) };
    println!("{b}");
}
```

```rust
// 5. double free
fn main() {
    let caixa = Box::new(10);
    let p = Box::into_raw(caixa);
    unsafe {
        drop(Box::from_raw(p));
        drop(Box::from_raw(p));
    }
}
```

```rust
// 6. memória não inicializada
fn main() {
    let mut v: Vec<String> = Vec::with_capacity(3);
    unsafe { v.set_len(3) };
    println!("{}", v[0].len());
}
```

```rust
// 7. data race
use std::thread;

static mut TOTAL: u64 = 0;

fn main() {
    let t = thread::spawn(|| {
        for _ in 0..1000 {
            unsafe { *(&raw mut TOTAL) += 1 };
        }
    });
    for _ in 0..1000 {
        unsafe { *(&raw mut TOTAL) += 1 };
    }
    t.join().unwrap();
    println!("{}", unsafe { *(&raw const TOTAL) });
}
```

| # | `cargo run` | `cargo run --release` |
|---|---|---|
| 1 | `42` | `42` |
| 2 | `0` | `1` |
| 3 | `2` | `2` |
| 4 | `false` | `false` |
| 5 | morreu com SIGTRAP (código de saída 133) | saiu normalmente (código 0) |
| 6 | `0` | morreu com SIGTRAP (código 133) |
| 7 | `2000` | `2000` |

Quatro dos sete imprimiram exatamente o que um leitor ingênuo esperaria. O 2 mudou de resposta entre debug e release. O 5 e o 6 trocaram de comportamento: o double free quebrou em debug e passou em release; a `String` não inicializada passou em debug e quebrou em release. E o 7 deu o total "certo" porque mil incrementos terminam antes de a outra thread começar; com mais trabalho, ou em outra máquina, ele perderia incrementos (Módulo 11).

É isso que torna UB perigoso: o teste passa, o código vai para produção, e ele quebra quando o compilador, o alocador ou a carga mudam.

## Miri

O [Miri](https://github.com/rust-lang/miri) é um interpretador oficial do Rust. Em vez de compilar para código de máquina, ele executa o programa passo a passo sobre uma memória simulada, que guarda mais informação do que a real: de qual alocação veio cada ponteiro, quais bytes estão inicializados, quais referências ainda são válidas, que thread escreveu por último em cada posição. Com isso, ele detecta UB **no momento em que acontece**.

Ele só existe no *nightly*. Instale uma vez:

```bash
rustup toolchain install nightly --component miri
# ou, se você já tem o nightly:
rustup +nightly component add miri
```

E rode no lugar do `cargo run` ou `cargo test`:

```bash
cargo +nightly miri run
cargo +nightly miri test
```

O seu projeto continua na versão estável; o `+nightly` vale só para aquele comando. Os sete programas acima, no Miri:

**1. Pendurado:**

```text
error: Undefined Behavior: memory access failed: alloc205 has been freed, so this pointer is dangling
 --> src/main.rs:7:29
  |
7 |     println!("{}", unsafe { *p });
  |                             ^^ Undefined Behavior occurred here
  |
  = help: this indicates a bug in the program: it performed an invalid operation, and caused Undefined Behavior
  = help: see https://doc.rust-lang.org/nightly/reference/behavior-considered-undefined.html for further information
help: alloc205 was allocated here:
 --> src/main.rs:4:13
  |
4 |         let x = 42;
  |             ^
help: alloc205 was deallocated here:
 --> src/main.rs:6:5
  |
6 |     }
  |     ^
```

O relatório diz onde a memória nasceu, onde morreu e onde foi usada depois disso. É o mesmo tipo de informação de um *use-after-free* no AddressSanitizer.

**2. Fora dos limites:**

```text
error: Undefined Behavior: memory access failed: attempting to access 4 bytes, but got alloc290+0xc which is at or beyond the end of the allocation of size 12 bytes
 --> src/main.rs:4:27
  |
4 |     let quarto = unsafe { *p.add(3) };
  |                           ^^^^^^^^^ Undefined Behavior occurred here
```

**3. Dois `&mut`:**

```text
error: Undefined Behavior: attempting a read access using <552> at alloc205[0x0], but that tag does not exist in the borrow stack for this location
 --> src/main.rs:6:5
  |
6 |     *a += 1;
  |     ^^^^^^^ this error occurs as part of an access at alloc205[0x0..0x4]
  |
  = help: this indicates a potential bug in the program: it performed an invalid operation, but the Stacked Borrows rules it violated are still experimental
  = help: see https://github.com/rust-lang/unsafe-code-guidelines/blob/master/wip/stacked-borrows.md for further information
help: <552> was created by a Unique retag at offsets [0x0..0x4]
 --> src/main.rs:4:22
  |
4 |     let a = unsafe { &mut *p };
  |                      ^^^^^^^
help: <552> was later invalidated at offsets [0x0..0x4] by a Unique retag
 --> src/main.rs:5:22
  |
5 |     let b = unsafe { &mut *p };
  |                      ^^^^^^^
```

Este é o caso mais "Rust" da lista. Em C, dois ponteiros para o mesmo `int` são normais. Em Rust, um `&mut` promete acesso **exclusivo**, e o compilador usa essa promessa para otimizar (por exemplo, guardar `*a` num registrador sem reler a memória). Criar `b` invalida `a`; usar `a` depois disso é UB, mesmo que o resultado tenha dado `2`.

**4. Valor inválido:**

```text
error: Undefined Behavior: constructing invalid value of type bool: encountered 0x02, but expected a boolean
 --> src/main.rs:3:28
  |
3 |     let b: bool = unsafe { std::mem::transmute(byte) };
  |                            ^^^^^^^^^^^^^^^^^^^^^^^^^ Undefined Behavior occurred here
```

**5. Double free:**

```text
error: Undefined Behavior: constructing invalid value of type std::boxed::Box<i32>: encountered a dangling box (use-after-free)
...
     = note: stack backtrace:
             0: std::boxed::Box::<i32>::from_raw_in
...
             2: main
                 at src/main.rs:6:14: 6:30
```

O Miri pega o problema antes mesmo da segunda liberação: um `Box` apontando para memória já liberada é, por si só, um valor inválido.

**6. Não inicializada:**

```text
error: Undefined Behavior: reading memory at alloc306[0x10..0x18], but memory is uninitialized at [0x10..0x18], and this operation requires initialized memory
...
             2: main
                 at src/main.rs:4:20: 4:30
```

**7. Data race:**

```text
error: Undefined Behavior: Data race detected between (1) non-atomic write on thread `main` and (2) non-atomic read on thread `unnamed-1` at alloc1
  --> src/main.rs:8:22
   |
 8 |             unsafe { *(&raw mut TOTAL) += 1 };
   |                      ^^^^^^^^^^^^^^^^^^^^^^ (2) just happened here
   |
help: and (1) occurred earlier here
  --> src/main.rs:12:18
   |
12 |         unsafe { *(&raw mut TOTAL) += 1 };
   |                  ^^^^^^^^^^^^^^^^^^^^^^
```

Sete de sete, cada um com a linha exata. Nas execuções normais, quatro tinham passado sem sinal nenhum.

## O modelo de aliasing ainda é experimental

Repare no caso 3: *"the Stacked Borrows rules it violated are still experimental"*. As regras exatas de quando um ponteiro cru ou uma referência deixam de valer ainda não foram fechadas pela linguagem. O Miri implementa dois modelos candidatos: **Stacked Borrows** (o padrão) e **Tree Borrows** (`MIRIFLAGS=-Zmiri-tree-borrows`), que é mais permissivo em alguns casos.

Um exemplo real, encontrado ao escrever a lição 02. A primeira versão do exemplo criava os dois ponteiros direto da variável:

```rust
let mut x = 10;
let p_leitura = &raw const x;
let p_escrita = &raw mut x;
unsafe {
    *p_escrita += 1;
    println!("valor: {}", *p_leitura);
}
```

Rodando normalmente, imprimiu `valor: 11`. No Miri padrão (as linhas citadas são as do programa completo):

```text
error: Undefined Behavior: trying to retag from <550> for SharedReadOnly permission at alloc224[0x0], but that tag does not exist in the borrow stack for this location
  --> src/main.rs:18:31
   |
18 |         println!("valor: {}", *p_leitura);
   |                               ^^^^^^^^^^ this error occurs as part of retag at alloc224[0x0..0x4]
   |
   = help: this indicates a potential bug in the program: it performed an invalid operation, but the Stacked Borrows rules it violated are still experimental
...
help: <550> was created by a SharedReadOnly retag at offsets [0x0..0x4]
  --> src/main.rs:10:21
   |
10 |     let p_leitura = &raw const x;
   |                     ^^^^^^^^^^^^
help: <550> was later invalidated at offsets [0x0..0x4] by a write access
  --> src/main.rs:17:9
   |
17 |         *p_escrita += 1;
   |         ^^^^^^^^^^^^^^^
```

Com `MIRIFLAGS=-Zmiri-tree-borrows`, o mesmo programa passou. Ou seja, é um caso em que os dois modelos discordam. A saída prática é não depender da discordância: **derivar todos os ponteiros de uma mesma origem** (`let p_leitura = p_escrita as *const i32;`) satisfaz os dois modelos, e é como o exemplo ficou. Se o seu código passa no Stacked Borrows, ele passa nos dois.

## O que o Miri não faz

- **Só verifica o que executa.** Se o caminho com o bug não roda nos testes, o Miri não vê. Ele é tão bom quanto a sua suíte de testes. Nas palavras do Rust Book: se o Miri acha um problema, há um bug; se não acha, isso não prova que não há.
- **Não executa código de fora do Rust.** Uma chamada a uma função de C termina com `unsupported operation: can't call foreign function`. Nos projetos com FFI (lições 05 e 06), o Miri cobre a parte Rust; o lado C precisa de sanitizers.
- **É lento.** Dezenas a centenas de vezes mais lento que o código nativo. Use entradas pequenas nos testes rodados no Miri, ou marque os pesados com `#[cfg_attr(miri, ignore)]`.
- **Não substitui o raciocínio.** Os comentários `// SAFETY:` continuam sendo a primeira linha de defesa; o Miri confere se eles estão certos nos casos testados.

Uma prática comum em crates com `unsafe` é rodar `cargo +nightly miri test` na CI.

## Comparando com C++ e os sanitizers

O mesmo bug do *iterador invalidado*, em C++:

```cpp
#include <iostream>
#include <vector>

int main() {
    std::vector<int> v = {1, 2, 3};
    const int &primeiro = v[0];
    v.push_back(4);              // pode realocar: `primeiro` fica pendurada
    std::cout << primeiro << "\n";
}
```

Com `c++ -std=c++20 -Wall -Wextra`, compila sem nenhum aviso e imprime `7`, um valor que nunca esteve no vetor. Com `-fsanitize=address`:

```text
==78370==ERROR: AddressSanitizer: heap-use-after-free on address 0x6020000000d0 at pc 0x0001026f8b20 bp 0x00016d705830 sp 0x00016d705828
READ of size 4 at 0x6020000000d0 thread T0
    #0 0x0001026f8b1c in main invalida.cpp:8
...
```

Em Rust seguro, o mesmo código não compila:

```text
error[E0502]: cannot borrow `v` as mutable because it is also borrowed as immutable
 --> src/main.rs:4:5
  |
3 |     let primeiro = &v[0];
  |                     - immutable borrow occurs here
4 |     v.push(4);
  |     ^^^^^^^^^ mutable borrow occurs here
5 |     println!("{primeiro}");
  |                -------- immutable borrow later used here
```

| | C++ | Rust |
|---|---|---|
| onde pode haver UB | em qualquer linha | só em código `unsafe` |
| detecção na compilação | avisos e analisadores estáticos, parciais | o borrow checker, para todo código seguro |
| detecção em execução | ASan, UBSan, TSan, MSan (rápidos, no binário real) | Miri (lento, interpretado, mais preciso para aliasing e valores inválidos) |
| o que revisar | o programa todo | os blocos `unsafe` e os módulos onde estão |

Os sanitizers também existem para Rust no nightly (`-Zsanitizer=address`) e são úteis justamente onde o Miri não chega: no código C de um projeto com FFI.

## Armadilhas comuns

- **"Testei e funcionou."** Todos os sete exemplos pareceram funcionar em pelo menos um dos modos. Rode o Miri em todo código com `unsafe`.
- **Confiar nas verificações de debug.** `get_unchecked` fora dos limites e desreferências desalinhadas abortam em debug (lição 03), mas a própria mensagem diz que a verificação é opcional e não existe em release.
- **Achar que o Miri cobre o C.** Ele para na primeira chamada estrangeira. Para o lado C, use sanitizers.
- **Ignorar o aviso "still experimental".** Um erro de Stacked Borrows indica um padrão que pode virar UB oficial. Prefira mudar o código a desligar o modelo.

## Resumo

- UB é qualquer coisa, e o compilador otimiza supondo que ele não acontece. Vazamento, deadlock e overflow de inteiro não são UB.
- Casos comuns: pendurado, fora dos limites, dois `&mut`, valor inválido, double free, não inicializado, data race.
- `cargo +nightly miri run`/`test` detecta UB no momento em que ocorre, com a linha e a origem da memória.
- O modelo de aliasing ainda é experimental (Stacked x Tree Borrows); derive ponteiros de uma origem só.
- O Miri só vê o que executa e não roda C. Em C++, o papel dele é dos sanitizers, para o programa inteiro.

## Exercícios

Veja o exercício 3 em [`exercicios.md`](exercicios.md).

## Leitura complementar

- [The Rust Reference: Behavior considered undefined](https://doc.rust-lang.org/reference/behavior-considered-undefined.html) e [Behavior not considered unsafe](https://doc.rust-lang.org/reference/behavior-not-considered-unsafe.html)
- [Miri (repositório e README)](https://github.com/rust-lang/miri)
- [The Rust Programming Language, 20.1: Using Miri to Check Unsafe Code](https://doc.rust-lang.org/stable/book/ch20-01-unsafe-rust.html#using-miri-to-check-unsafe-code)
- [The Rustonomicon: What Unsafe Rust Can Do](https://doc.rust-lang.org/nomicon/what-unsafe-does.html)

---

[← Lição anterior](03-abstracoes-seguras.md) · [Índice do módulo](README.md) · [Próxima lição →](05-chamando-c.md)

# 04 — `Cell` e `RefCell`: mutabilidade interior

**Objetivos.** Ao final, você vai saber:

- o que é mutabilidade interior, e por que ela não quebra as regras do Módulo 03;
- usar `Cell` para valores pequenos, `RefCell` para o resto e `OnceCell` para inicializar uma vez;
- combinar `Rc<RefCell<T>>` para estado compartilhado e mutável;
- evitar o *panic* do `RefCell`, inclusive os casos que mudaram na edition 2024.

## O problema: alterar através de `&self`

No exercício 3 do Módulo 06, o `MemoriaLogger` guardava as mensagens num `Vec`, e por isso o método era `registrar(&mut self, ...)`. A pergunta que ficou: e se a trait pedir `&self`? Faz sentido pedir: um logger é usado por vários objetos ao mesmo tempo, cada um com uma referência compartilhada para ele.

```rust
trait Logger {
    fn registrar(&self, msg: &str);
}

struct Pedido<'a> {
    id: u32,
    log: &'a dyn Logger,
}
```

Com `&self`, `self.linhas.push(...)` não compila: é alterar através de um empréstimo compartilhado. Mas, do ponto de vista de quem usa, registrar uma mensagem não muda nada que o `Pedido` observe. A mudança é um detalhe interno.

*Interior mutability* (mutabilidade interior) é isso: um tipo que permite alterar o seu conteúdo através de `&self`, e que **verifica as regras de empréstimo de outro jeito**, em vez de abandoná-las. A std tem três ferramentas, em `std::cell`:

| Tipo | Como garante a regra | Custo | Para quê |
|---|---|---|---|
| `Cell<T>` | nunca empresta o interior: só copia ou troca valores inteiros | zero | contadores, flags, valores `Copy` pequenos |
| `RefCell<T>` | conta os empréstimos **em tempo de execução**; violação dá *panic* | um contador e uma verificação por empréstimo | qualquer `T` |
| `OnceCell<T>` | só pode ser escrito uma vez; depois, só leitura | zero na leitura | inicialização preguiçosa |

## `Cell<T>`: trocar o valor inteiro

```rust
use std::cell::Cell;

let c = Cell::new(1);
c.update(|x| x + 1);  // lê, aplica, grava
c.set(c.get() * 10);
println!("{}", c.get()); // 20
```

`Cell` nunca dá uma referência para o que está dentro: `get` devolve uma **cópia**, `set` grava um valor novo, `replace` troca e devolve o antigo, `take` troca pelo `Default` e devolve o antigo. Como ninguém tem um `&T` apontando lá para dentro, alterar não invalida nada, e não há o que verificar. Por isso o custo é zero.

A consequência é que `get` só existe para tipos `Copy`:

```rust
// ❌ NÃO COMPILA
use std::cell::Cell;

fn main() {
    let nome = Cell::new(String::from("Ana"));
    let n = nome.get();
    println!("{n}");
}
```

```text
error[E0599]: the method `get` exists for struct `Cell<String>`, but its trait bounds were not satisfied
   --> src/main.rs:5:18
    |
  5 |     let n = nome.get();
    |                  ^^^
...
    = note: the following trait bounds were not satisfied:
            `String: Copy`
```

Com um `String`, use `replace` ou `take` (`let velho = nome.replace(String::from("Bia"))`), ou passe para o `RefCell`.

## `RefCell<T>`: o borrow checker em tempo de execução

`RefCell` empresta o interior, e para isso guarda um contador de empréstimos ativos:

- `borrow()` devolve um `Ref<T>`, que se comporta como `&T`;
- `borrow_mut()` devolve um `RefMut<T>`, que se comporta como `&mut T`;
- `Ref` e `RefMut` são **guardas** (lição 02): o `Drop` deles devolve o empréstimo.

As regras são as do Módulo 03, só que verificadas na hora: vários `Ref` ao mesmo tempo, ou um `RefMut` sozinho. O logger fica assim:

```rust
use std::cell::{Cell, RefCell};

#[derive(Default)]
struct MemoriaLogger {
    linhas: RefCell<Vec<String>>,
    chamadas: Cell<u32>,
}

impl Logger for MemoriaLogger {
    fn registrar(&self, msg: &str) {
        self.chamadas.set(self.chamadas.get() + 1);
        self.linhas.borrow_mut().push(msg.to_string());
    }
}

let log = MemoriaLogger::default();
let p1 = Pedido { id: 1, log: &log };
let p2 = Pedido { id: 2, log: &log };
p1.pagar();
p2.pagar();
println!("{} chamadas: {:?}", log.chamadas.get(), log.linhas.borrow());
```

```text
2 chamadas: ["pedido 1 pago", "pedido 2 pago"]
```

O `RefMut` criado por `borrow_mut()` vive só até o fim da linha (é um temporário), então o empréstimo é curtíssimo. Esse é o jeito saudável de usar `RefCell`. Código completo em [`codigo/rc-e-refcell`](codigo/rc-e-refcell/src/main.rs).

### Quando a regra é violada

```rust
// 💥 PANIC EM RUNTIME
use std::cell::RefCell;

fn main() {
    let lista = RefCell::new(vec![1, 2, 3]);
    let leitura = lista.borrow();
    println!("primeiro: {}", leitura[0]);
    lista.borrow_mut().push(4);
    println!("{leitura:?}");
}
```

```text
primeiro: 1

thread 'main' (245926) panicked at src/main.rs:7:11:
RefCell already borrowed
```

Com `&` e `&mut` comuns, este código seria um E0502 na compilação. Com `RefCell`, compila e dá *panic* na linha 7. O `RefCell` não deixa você ter dois empréstimos incompatíveis: ele **troca o erro de compilação por um panic**. O *undefined behavior* continua impossível; o que você perde é o aviso antecipado.

> [!NOTE]
> Material antigo mostra a mensagem `already borrowed: BorrowMutError`. Nas versões atuais ela é só `RefCell already borrowed`. É o mesmo pânico.

Para tratar o conflito em vez de quebrar, use `try_borrow_mut` (e `try_borrow`), que devolvem `Result`:

```rust
let leitura = lista.borrow();
match lista.try_borrow_mut() {
    Ok(_) => println!("consegui escrever"),
    Err(e) => println!("não deu: {e}"),
}
drop(leitura);
```

```text
não deu: RefCell already borrowed
```

## `Rc<RefCell<T>>`: vários donos que alteram

`Rc` dá vários donos, mas só leitura. `RefCell` dá alteração através de `&`. Juntos, dão o que um objeto de Python é por padrão: algo compartilhado que qualquer um pode alterar.

```rust
struct Carrinho {
    itens: Rc<RefCell<Vec<String>>>,
}

impl Carrinho {
    fn adicionar(&self, item: &str) {
        self.itens.borrow_mut().push(item.to_string());
    }
}

let lista = Rc::new(RefCell::new(Vec::new()));
let celular = Carrinho { itens: Rc::clone(&lista) };
let site = Carrinho { itens: Rc::clone(&lista) };
celular.adicionar("café");
site.adicionar("pão");
println!("carrinho: {:?}", lista.borrow());
```

```text
carrinho: ["café", "pão"]
```

A ordem dos tipos importa: `Rc<RefCell<T>>` é "vários donos de **uma** célula mutável", e todos veem as mudanças. `RefCell<Rc<T>>` seria "uma célula cujo conteúdo é um ponteiro compartilhado"; você poderia trocar o ponteiro, mas não alterar o `T`.

## `OnceCell<T>`: calcular uma vez

Um cache num método `&self` é o terceiro uso típico. `OnceCell::get_or_init` roda a closure na primeira chamada e devolve `&T` daí em diante:

```rust
use std::cell::OnceCell;

struct Documento {
    texto: String,
    palavras: OnceCell<usize>,
}

impl Documento {
    fn palavras(&self) -> usize {
        *self.palavras.get_or_init(|| {
            println!("  (contando...)");
            self.texto.split_whitespace().count()
        })
    }
}
```

```text
  (contando...)
palavras: 4
palavras: 4
```

Como o valor nunca muda depois de escrito, `OnceCell` pode devolver `&T` sem contador nenhum. `LazyCell` é a variante que recebe a closure já na criação. As versões para várias threads, `OnceLock` e `LazyLock`, são o jeito moderno de fazer estado global, no [Módulo 11](../11-concorrencia/README.md).

## Comparando com C++ e Python

O C++ tem a mesma ideia com a palavra `mutable`: um membro `mutable` pode ser alterado dentro de um método `const`.

```cpp
class Logger {
    mutable std::vector<std::string> linhas;
public:
    void registrar(const std::string& msg) const { linhas.push_back(msg); }
};
```

A diferença: o `mutable` desliga a verificação e não põe nada no lugar. Se alguém estiver percorrendo `linhas` enquanto outro método `const` faz `push_back`, o iterador fica pendente e nada avisa. O `RefCell` detecta esse caso e para o programa com uma mensagem. O `const_cast` é ainda mais forte: alterar através dele um objeto que foi **declarado** `const` é UB. Em Rust, `Cell` e `RefCell` são a **única** forma segura de alterar através de `&`; tudo o mais é `unsafe` (o `UnsafeCell`, a peça que está por baixo dos dois, é assunto do Módulo 15).

Em Python, não há o que comparar no nível do tipo: todo objeto é mutável através de qualquer referência, e um erro como alterar uma lista durante a iteração aparece (às vezes) como `RuntimeError: ... changed size during iteration`, só para alguns tipos. O `RefCell` aplica essa verificação a qualquer tipo, e sempre.

## Armadilhas comuns

### Um `Ref` vivo mais tempo do que parece

O pânico raramente aparece tão óbvio quanto no exemplo acima. O caso típico é um laço que segura um `borrow()` e chama algo que faz `borrow_mut()` no mesmo `RefCell`, às vezes três funções abaixo. A regra prática: **não guarde `Ref`/`RefMut` em variáveis**; use o empréstimo e solte-o na mesma expressão, ou copie o que precisa antes de alterar.

### O escopo dos temporários mudou na edition 2024

Um `borrow()` usado como condição de `if let` cria um temporário (o `Ref`). Até a edition 2021, esse temporário vivia até o fim do `else`. Este código, então, dava pânico:

```rust
use std::cell::RefCell;

fn main() {
    let fila = RefCell::new(Vec::<i32>::new());
    if let Some(x) = fila.borrow().first() {
        println!("primeiro: {x}");
    } else {
        fila.borrow_mut().push(1);
    }
    println!("{:?}", fila.borrow());
}
```

```text
# edition 2021
thread 'main' (246718) panicked at src/main.rs:8:14:
RefCell already borrowed

# edition 2024
[1]
```

Na edition 2024, o temporário do `if let` é descartado antes do `else`, e o código funciona. Mas **o `match` não mudou**: o temporário da expressão do `match` vive até o fim do `match` inteiro.

```rust
// 💥 PANIC EM RUNTIME
use std::cell::RefCell;

fn main() {
    let fila = RefCell::new(Vec::<i32>::new());
    match fila.borrow().first() {
        Some(x) => println!("primeiro: {x}"),
        None => fila.borrow_mut().push(1),
    }
}
```

```text
thread 'main' (247063) panicked at src/main.rs:7:22:
RefCell already borrowed
```

A correção é encerrar o empréstimo antes de decidir: `let primeiro = fila.borrow().first().copied();` e depois o `match primeiro`. Desconfie de exemplos de `RefCell` em material antigo: alguns que davam pânico não dão mais, e o contrário não acontece.

A outra mudança da 2024 é na expressão final de um bloco: `let n = { let c = RefCell::new(vec![1, 2]); c.borrow().len() };` dava E0597 na 2021 (o `Ref` vivia mais que `c`) e compila na 2024.

### Devolver uma referência de dentro do `RefCell`

```rust
// ❌ NÃO COMPILA
use std::cell::RefCell;

struct Agenda {
    nomes: RefCell<Vec<String>>,
}

impl Agenda {
    fn primeiro(&self) -> &String {
        &self.nomes.borrow()[0]
    }
}
```

```text
error[E0515]: cannot return value referencing temporary value
 --> src/main.rs:9:9
  |
9 |         &self.nomes.borrow()[0]
  |         ^-------------------^^^
  |         ||
  |         |temporary value created here
  |         returns a value referencing data owned by the current function
```

O `&String` só é válido enquanto o `Ref` existir, e o `Ref` morre no fim da função. Se o `&String` saísse, o `RefCell` acharia que não há mais leitores e permitiria um `borrow_mut`. As saídas: devolver uma cópia (`self.nomes.borrow()[0].clone()`), devolver o próprio guarda (`Ref<'_, String>`, com `Ref::map(self.nomes.borrow(), |v| &v[0])`), ou receber uma closure que usa o valor lá dentro.

### Usar `RefCell` para calar o borrow checker

`RefCell` não é um atalho para quando o `&mut` "não deixa". Ele transforma erros de compilação em pânicos, que aparecem só no caminho do código que os dispara. Use-o quando a mutação através de `&` é parte do projeto (cache, logger, grafo com `Rc`), e não para contornar um erro que uma reorganização resolveria.

## Resumo

- Mutabilidade interior: alterar através de `&self`, com as regras de empréstimo garantidas de outro jeito.
- `Cell<T>`: troca valores inteiros (`get`, `set`, `replace`, `take`, `update`), nunca empresta o interior, custo zero. `get` exige `Copy`.
- `RefCell<T>`: `borrow`/`borrow_mut` com contagem em tempo de execução; violar a regra dá *panic* (`RefCell already borrowed`). `try_borrow_mut` devolve `Result`.
- `Rc<RefCell<T>>`: vários donos que alteram e veem as mudanças. `OnceCell`: inicializar uma vez em `&self`.
- Edition 2024: o temporário do `if let` some antes do `else`; o do `match` continua vivo até o fim. Teste exemplos antigos.
- Nada disso atravessa threads: os equivalentes são `Mutex`, `RwLock`, atômicos e `OnceLock` (Módulo 11).

## Exercícios

Veja os exercícios 2 e 3 em [`exercicios.md`](exercicios.md).

## Leitura complementar

- [The Rust Programming Language, 15.5 `RefCell<T>` and the Interior Mutability Pattern](https://doc.rust-lang.org/stable/book/ch15-05-interior-mutability.html)
- [Documentação de `std::cell`](https://doc.rust-lang.org/std/cell/index.html) (a seção "When to choose interior mutability")
- [Edition Guide: `if let` temporary scope](https://doc.rust-lang.org/edition-guide/rust-2024/temporary-if-let-scope.html) e [Tail expression temporary scope](https://doc.rust-lang.org/edition-guide/rust-2024/temporary-tail-expr-scope.html)
- [Comprehensive Rust: Interior Mutability](https://google.github.io/comprehensive-rust/borrowing/interior-mutability.html)

---

[← Lição anterior](03-rc-posse-compartilhada.md) · [Índice do módulo](README.md) · [Próxima lição →](05-weak-e-ciclos.md)

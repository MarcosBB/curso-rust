# 04 — Closures e captura

**Objetivos.** Ao final, você vai saber:

- como o compilador decide se uma closure captura por `&`, por `&mut` ou por valor;
- o que é uma closure por dentro (uma struct anônima) e por que cada uma tem um tipo próprio;
- quando e por que usar `move`;
- o que fazer quando uma closure recebe e devolve referências.

O [kit do Módulo 01](../01-tour-da-sintaxe/08-kit-closures.md) mostrou a sintaxe (`|x| x + 1`), a inferência que trava no primeiro uso e os usos com `map`, `filter`, `sort_by_key` e `retain`. Ele também disse que a closure "enxerga as variáveis do escopo onde foi criada" e deixou o *como* para depois. É o assunto desta lição.

## Três modos de captura, escolhidos pelo uso

Em C++, você declara como cada variável entra na lambda: `[x]` por cópia, `[&x]` por referência, `[=]` ou `[&]` para todas. Em Rust, você não declara nada. O compilador olha **o que o corpo faz com cada variável** e escolhe o modo menos exigente que funciona:

```rust
let lista = vec![1, 2, 3];
let mut contador = 0;
let nome = String::from("Ana");

let mostra = || println!("lista: {lista:?}"); // só lê: captura &lista
let mut conta = || contador += 1; // altera: captura &mut contador
let entrega = || nome; // devolve `nome`: captura por valor

mostra();
conta();
conta();
let dono = entrega();
println!("{lista:?} {contador} {dono}");
```

```text
lista: [1, 2, 3]
[1, 2, 3] 2 Ana
```

| O corpo... | A closure captura | Equivale a |
|---|---|---|
| só lê a variável | `&T` (empréstimo compartilhado) | `[&x]` em C++ |
| altera a variável | `&mut T` (empréstimo mutável) | `[&x]` com escrita |
| move a variável para fora (devolve, passa por valor, faz `drop`) | `T` (por valor) | `[x = std::move(x)]` |

A decisão é **por variável**: a mesma closure pode ler uma variável e alterar outra. E é a mais fraca possível: se só ler basta, ela não pega `&mut` nem toma posse.

## Uma closure é uma struct

O jeito mais útil de pensar numa closure é este: o compilador cria uma struct anônima, com um campo para cada variável capturada, e implementa nela o método de chamada. As duas closures abaixo viram, conceitualmente, as structs ao lado:

```text
let n = 5;

let soma = |x| x + n;          struct Soma<'a> { n: &'a i32 }      // guarda uma referência
                               chamar(&self, x) = x + *self.n

let soma = move |x| x + n;     struct SomaMove { n: i32 }          // guarda uma cópia
                               chamar(&self, x) = x + self.n
```

Dá para conferir o tamanho dessas structs:

```rust
use std::mem::size_of_val;

let n = 5_i32;
let s = String::from("oi");

let sem_captura = |x: i32| x + 1;
let por_ref = |x: i32| x + n;
let por_valor = move |x: i32| x + n;
let com_string = move || s.len();

println!(
    "{} {} {} {}",
    size_of_val(&sem_captura),
    size_of_val(&por_ref),
    size_of_val(&por_valor),
    size_of_val(&com_string)
);
```

```text
0 8 4 24
```

Sem captura, a closure não ocupa nada. Com `&i32`, ocupa um ponteiro (8 bytes). Com o `i32` copiado, 4 bytes. Com a `String` movida, os 24 bytes da `String` (ponteiro, tamanho e capacidade, como no [Módulo 02](../02-ownership/05-string-e-str.md)). Não há alocação no heap nem ponteiro escondido: a closure é um valor comum, que mora onde você o guardar.

Duas consequências desse modelo:

- **Cada closure tem um tipo próprio, sem nome.** Mesmo duas closures com o mesmo texto são structs diferentes. É por isso que você não consegue escrever o tipo de uma closure, e por isso que a lição 05 precisa de generics, `impl Fn` e `Box<dyn Fn>` para guardá-las e passá-las adiante.
- **As regras de empréstimo valem para a closure inteira**, enquanto ela existir. Uma closure que capturou `&mut v` é, para o borrow checker, um `&mut v` vivo.

## A closure segura o empréstimo

Como a closure guarda a referência, o empréstimo começa quando ela é **criada**, e não quando é chamada, e dura até o último uso dela:

```rust
// ❌ NÃO COMPILA
fn main() {
    let mut v = vec![1, 2];
    let mut add = || v.push(3);
    println!("{v:?}");
    add();
}
```

```text
error[E0502]: cannot borrow `v` as immutable because it is also borrowed as mutable
 --> src/main.rs:4:16
  |
3 |     let mut add = || v.push(3);
  |                   -- - first borrow occurs due to use of `v` in closure
  |                   |
  |                   mutable borrow occurs here
4 |     println!("{v:?}");
  |                ^ immutable borrow occurs here
5 |     add();
  |     --- mutable borrow later used here
```

É o E0502 do [Módulo 03](../03-borrowing-e-lifetimes/02-borrow-checker-e-nll.md), com a closure no papel do `&mut`. A correção é a mesma de sempre: mova o `println!` para depois do último `add()`. Duas closures que alteram a mesma variável dão o outro erro clássico:

```rust
// ❌ NÃO COMPILA
fn main() {
    let mut n = 0;
    let mut a = || n += 1;
    let mut b = || n += 2;
    a();
    b();
}
```

```text
error[E0499]: cannot borrow `n` as mutable more than once at a time
 --> src/main.rs:4:17
  |
3 |     let mut a = || n += 1;
  |                 -- - first borrow occurs due to use of `n` in closure
  |                 |
  |                 first mutable borrow occurs here
4 |     let mut b = || n += 2;
  |                 ^^ - second borrow occurs due to use of `n` in closure
  |                 |
  |                 second mutable borrow occurs here
5 |     a();
  |     - first borrow later used here
```

E chamar uma closure que altera algo exige `let mut` **na closure**:

```rust
// ❌ NÃO COMPILA
fn main() {
    let mut contador = 0;
    let inc = || contador += 1;
    inc();
    println!("{contador}");
}
```

```text
error[E0596]: cannot borrow `inc` as mutable, as it is not declared as mutable
 --> src/main.rs:4:5
  |
3 |     let inc = || contador += 1;
  |                  -------- calling `inc` requires mutable binding due to mutable borrow of `contador`
4 |     inc();
  |     ^^^ cannot borrow as mutable
  |
help: consider changing this to be mutable
  |
3 |     let mut inc = || contador += 1;
  |         +++
```

Pelo modelo da struct, faz sentido: chamar `inc` altera um campo dela (o `&mut contador` é usado para escrever), então a própria `inc` precisa ser mutável. É o `mutable` das lambdas do C++, só que exigido pelo compilador.

## Captura disjunta: só o campo usado

Desde a edition 2021, a closure captura o **caminho** que usa, e não a variável inteira:

```rust
struct Config {
    nome: String,
    nivel: u8,
}

fn main() {
    let mut cfg = Config { nome: String::from("app"), nivel: 1 };
    let mut sobe = || cfg.nivel += 1;
    println!("nome: {}", cfg.nome); // ok: a closure capturou só cfg.nivel
    sobe();
    println!("nivel: {}", cfg.nivel);
}
```

```text
nome: app
nivel: 2
```

A closure pegou `&mut cfg.nivel`, e `cfg.nome` continua livre. Em C++, `[&]` ou `[this]` capturam o objeto inteiro (na prática, o ponteiro `this`), e nada impede você de usar os outros campos ao mesmo tempo, com ou sem segurança. Em Rust, a captura é fina e o compilador confere.

## `move`: tomar posse de tudo

`move` antes das barras força a closure a capturar **por valor** tudo o que usa, mesmo o que ela só lê:

```rust
let k = 10;
let vezes_k = move |x: i32| x * k; // i32 é Copy: a closure leva uma cópia
println!("{} {k}", vezes_k(2));

let nome = String::from("Ana");
let saudacao = move || format!("oi, {nome}"); // String: a closure leva o original
println!("{}", saudacao());
// println!("{nome}"); // E0382: `nome` foi movida para a closure
```

```text
20 10
oi, Ana
```

Com tipos `Copy`, "por valor" é uma cópia, e o original continua usável. Com `String`, `Vec` e afins, é um *move*: o [Módulo 02](../02-ownership/02-regras-de-ownership-e-move.md) mostrou o E0382 que aparece se você usar `nome` depois.

Se a closure só lê, por que você ia querer `move`? Porque uma closure que guarda referências **não pode viver mais que as variáveis referenciadas**. Os dois casos típicos são devolver uma closure e mandá-la para outra thread.

### Devolver uma closure

```rust
// ❌ NÃO COMPILA
fn somador(n: i32) -> impl Fn(i32) -> i32 {
    |x| x + n
}

fn main() {
    let mais5 = somador(5);
    println!("{}", mais5(1));
}
```

```text
error[E0373]: closure may outlive the current function, but it borrows `n`, which is owned by the current function
 --> src/main.rs:2:5
  |
2 |     |x| x + n
  |     ^^^     - `n` is borrowed here
  |     |
  |     may outlive borrowed value `n`
  |
note: closure is returned here
 --> src/main.rs:2:5
  |
2 |     |x| x + n
  |     ^^^^^^^^^
help: to force the closure to take ownership of `n` (and any other referenced variables), use the `move` keyword
  |
2 |     move |x| x + n
  |     ++++
```

Sem `move`, a closure guardaria `&n`, e `n` morre quando `somador` termina. O `help:` está certo: com `move |x| x + n`, a closure leva uma cópia de `n` e pode sair da função. (O `impl Fn(i32) -> i32` no retorno significa "algum tipo que implementa `Fn(i32) -> i32`", e a lição 05 explica.)

O mesmo código em C++ compila sem aviso, mesmo com `-Wall -Wextra`:

```cpp
std::function<int(int)> somador(int n) {
    return [&](int x) { return x + n; };  // captura n por referência
}

int main() {
    auto mais5 = somador(5);
    std::cout << mais5(1) << "\n";
}
```

Aqui imprimiu `2` em vez de `6`, e com o AddressSanitizer o problema aparece:

```text
==4827==ERROR: AddressSanitizer: stack-buffer-underflow on address 0x00016d339460 ...
READ of size 4 at 0x00016d339460 thread T0
    #0 0x000102ac5920 in somador(int)::$_0::operator()(int) const lambda.cpp:5
```

A referência pendurada numa lambda é um bug clássico de C++. Em Rust, é o E0373.

### Mandar para uma thread

```rust
// ❌ NÃO COMPILA
use std::thread;

fn main() {
    let v = vec![1, 2, 3];
    let h = thread::spawn(|| println!("{v:?}"));
    h.join().unwrap();
}
```

```text
error[E0373]: closure may outlive the current function, but it borrows `v`, which is owned by the current function
 --> src/main.rs:5:27
  |
5 |     let h = thread::spawn(|| println!("{v:?}"));
  |                           ^^            - `v` is borrowed here
  |                           |
  |                           may outlive borrowed value `v`
  |
note: function requires argument type to outlive `'static`
 --> src/main.rs:5:13
  |
5 |     let h = thread::spawn(|| println!("{v:?}"));
  |             ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
help: to force the closure to take ownership of `v` (and any other referenced variables), use the `move` keyword
...
```

A `note:` muda: `thread::spawn` exige que a closure não tenha empréstimos de nada local (*outlive `'static`*), porque a thread pode continuar rodando depois que `main` sair do escopo. O Módulo 11 trata de threads. Por ora, a receita: se você ainda precisa do valor fora da closure, **clone antes do `move`**, e a closure leva a cópia:

```rust
let v = vec![1, 2, 3];
let copia = v.clone();
let h = thread::spawn(move || println!("na thread: {copia:?}"));
h.join().unwrap();
println!("no main: {v:?}");
```

```text
na thread: [1, 2, 3]
no main: [1, 2, 3]
```

> [!WARNING]
> `move` decide **como** a closure captura, e não **o que** ela faz com o capturado. Uma closure `move` que só lê continua podendo ser chamada quantas vezes você quiser. Quem decide se ela pode ser chamada uma vez ou várias é o corpo, e esse é o assunto da lição 05.

## Sem *late binding*

Em Python, as closures capturam a **variável**, e não o valor dela no momento da criação. O resultado clássico:

```python
fs = [lambda: i for i in range(3)]
print([f() for f in fs])
```

```text
[2, 2, 2]
```

As três lambdas olham para o mesmo `i`, que no fim vale 2. Em Rust, isso não acontece em silêncio. Sem `move`, cada closure tentaria guardar `&i`, e o `i` de cada volta morre no fim da volta:

```rust
// ❌ NÃO COMPILA
fn main() {
    let mut fs = Vec::new();
    for i in 0..3 {
        fs.push(|| i);
    }
    let r: Vec<i32> = fs.iter().map(|f| f()).collect();
    println!("{r:?}");
}
```

```text
error[E0597]: `i` does not live long enough
 --> src/main.rs:4:20
  |
3 |     for i in 0..3 {
  |         - binding `i` declared here
4 |         fs.push(|| i);
  |         --      -- ^ borrowed value does not live long enough
  |         |       |
  |         |       value captured here
  |         borrow later used here
5 |     }
  |     - `i` dropped here while still borrowed
```

Com `fs.push(move || i)`, cada closure leva a sua cópia, e a saída é `[0, 1, 2]`. (O `Vec` aceita as três porque é **a mesma** expressão de closure executada três vezes: um tipo só, com valores diferentes no campo.)

## Closures que recebem e devolvem referências

O [Módulo 03](../03-borrowing-e-lifetimes/05-elisao-e-lifetime-anonimo.md) avisou que as regras de elisão valem para `fn`, e não para closures. Eis o caso:

```rust
// ❌ NÃO COMPILA
fn main() {
    let primeira = |s: &str| -> &str { s.split(' ').next().unwrap_or("") };
    println!("{}", primeira("Ana Souza"));
}
```

```text
error: lifetime may not live long enough
 --> src/main.rs:2:40
  |
2 |     let primeira = |s: &str| -> &str { s.split(' ').next().unwrap_or("") };
  |                        -        -      ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ returning this value requires that `'1` must outlive `'2`
  |                        |        |
  |                        |        let's call the lifetime of this reference `'2`
  |                        let's call the lifetime of this reference `'1`
```

Numa `fn primeira(s: &str) -> &str`, a elisão liga a saída à entrada. Numa closure, o compilador dá a cada `&` anotado um lifetime **independente** (`'1` e `'2`), e então devolver um pedaço de `s` exige que `'1` viva tanto quanto `'2`, o que ele não pode provar. Tirar a anotação do retorno (`|s: &str| s.split(...)...`) dá o mesmo erro.

As saídas, da mais simples para a menos:

```rust
// ✅ 1. Escreva uma fn: a elisão funciona
fn primeira(s: &str) -> &str {
    s.split(' ').next().unwrap_or("")
}

// ✅ 2. Escreva a closure direto na chamada, sem anotar tipos:
//       o tipo vem da assinatura do método que a recebe
let nomes = ["Ana Souza", "Bia Lima"];
let b: Vec<&str> = nomes.iter().map(|s| s.split(' ').next().unwrap_or("")).collect();
```

```text
["Ana", "Bia"]
```

Na prática, o problema só aparece quando você guarda numa variável uma closure que devolve uma referência para dentro do argumento. Se isso acontecer, escreva uma `fn`.

## Armadilhas comuns

- **Usar a variável enquanto a closure que a capturou ainda vai ser chamada** (E0502, E0499): a closure é um empréstimo vivo. Termine de usar a closure antes, ou reorganize a ordem.
- **Chamar uma closure que altera algo sem `let mut`** (E0596): siga o `help:`.
- **Devolver ou mandar para uma thread uma closure sem `move`** (E0373): siga o `help:`. Se ainda precisar do valor, clone antes.
- **Achar que `move` torna a closure "de uso único"**: não torna. `move` é sobre captura; o corpo decide a trait (lição 05).
- **Anotar `|s: &str| -> &str`** numa closure guardada em variável: troque por uma `fn`.

## Resumo

- A closure captura cada variável do jeito mais fraco que o corpo permite: `&`, depois `&mut`, depois por valor. Desde a edition 2021, captura só o campo usado.
- Uma closure é uma struct anônima com os capturados como campos. Cada closure tem um tipo próprio, e o seu tamanho é o dos capturados.
- Enquanto a closure existe, os empréstimos dela estão vivos: valem E0502, E0499 e E0596.
- `move` força a captura por valor. É necessário quando a closure sai do escopo (retorno, thread). Clone antes se precisar do original.
- Closures que devolvem referências para dentro do argumento não seguem a elisão das `fn`: prefira uma `fn`.

## Exercícios

Veja os exercícios 1 e 5 em [`exercicios.md`](exercicios.md).

## Leitura complementar

- [The Rust Programming Language, 13.1 Closures](https://doc.rust-lang.org/stable/book/ch13-01-closures.html)
- [The Rust Reference: Closure expressions](https://doc.rust-lang.org/reference/expressions/closure-expr.html) e [Closure types](https://doc.rust-lang.org/reference/types/closure.html) (o modelo da struct e as regras de captura)
- [Edition Guide: Disjoint capture in closures](https://doc.rust-lang.org/edition-guide/rust-2021/disjoint-capture-in-closures.html)
- [Comprehensive Rust: Closures](https://google.github.io/comprehensive-rust/closures.html)

---

[← Lição anterior](03-outras-colecoes.md) · [Índice do módulo](README.md) · [Próxima lição →](05-fn-fnmut-fnonce.md)

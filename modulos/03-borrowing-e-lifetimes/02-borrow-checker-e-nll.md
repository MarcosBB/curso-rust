# 02 — O borrow checker e NLL

**Objetivos.** Ao final, você vai saber:

- até onde vai um empréstimo (NLL: da criação até o último uso);
- ler os erros E0502, E0499, E0506 e E0596;
- por que passar uma `&mut` para uma função não a "gasta" (*reborrow*) e por que `v.push(v.len())` compila;
- reconhecer *iterator invalidation* e as duas saídas idiomáticas.

## Um empréstimo vive até o último uso

O *borrow checker* é a parte do compilador que aplica a regra da lição anterior. A pergunta que ele faz em cada linha é: "quais referências estão **vivas** aqui?". E uma referência está viva da criação até o **último uso**, e não até o fim do bloco `{}`.

Isso se chama NLL (*non-lexical lifetimes*, tempos de vida não léxicos) e existe desde 2018. Antes, o empréstimo durava até o fim do bloco, e muito código correto era recusado. Compare:

```rust
fn main() {
    let mut v = vec![1, 2, 3];
    let first = &v[0];
    println!("{first}");
    v.push(4);
    println!("{v:?}");
}
```

```text
1
[1, 2, 3, 4]
```

Isso compila, porque `first` é usada pela última vez no primeiro `println!`. Quando o `push` acontece, ela já está morta, mesmo continuando "em escopo" até o fim de `main`. Basta trocar a ordem de duas linhas para cair no E0502 da lição 01:

![Dois painéis com o código e barras ao lado. À esquerda (compila): a barra verde de v cobre as linhas 1 a 5, a barra roxa de &v cobre as linhas 2 e 3, e a barra vermelha de &mut v fica só na linha 4, sem sobrepor a roxa. À direita (E0502): a barra roxa vai da linha 2 à 4 e a vermelha está na linha 3, dentro da roxa; a linha 3 está destacada como conflito.](assets/regioes-nll.svg)

O tempo de vida de uma referência é, então, uma **região do código**: o conjunto de linhas entre a criação e o último uso. O erro aparece quando uma região de `&mut` cruza qualquer outra região de empréstimo do mesmo valor.

A região pode até acabar em pontos diferentes conforme o caminho do código:

```rust
fn main() {
    let mut v = vec![1, 2, 3];
    let first = &v[0];
    if v.len() > 2 {
        println!("primeiro: {first}");
        v.push(4);
    } else {
        v.push(0);
    }
    println!("{v:?}");
}
```

```text
primeiro: 1
[1, 2, 3, 4]
```

No `if`, `first` morre depois do `println!`. No `else`, ela nem é usada, então morreu logo após a criação. Os dois `push` são válidos.

## Os erros mais comuns

Todos são a mesma regra vista de ângulos diferentes. Vale reconhecer cada código.

### E0499: duas `&mut` ao mesmo tempo

```rust
// ❌ NÃO COMPILA
fn main() {
    let mut s = String::from("oi");
    let r1 = &mut s;
    let r2 = &mut s;
    r1.push('!');
    r2.push('?');
}
```

```text
error[E0499]: cannot borrow `s` as mutable more than once at a time
 --> src/main.rs:4:14
  |
3 |     let r1 = &mut s;
  |              ------ first mutable borrow occurs here
4 |     let r2 = &mut s;
  |              ^^^^^^ second mutable borrow occurs here
5 |     r1.push('!');
  |     -- first borrow later used here
```

Duas referências exclusivas para o mesmo valor seriam uma contradição. O rótulo da linha 5 mostra por que `r1` ainda estava viva: ela é usada depois.

### E0502 sem nenhuma escrita

A exclusividade de `&mut` vale mesmo que você nunca escreva por ela:

```rust
// ❌ NÃO COMPILA
fn main() {
    let mut s = String::from("oi");
    let r1 = &mut s;
    let r2 = &s;
    println!("{r1} {r2}");
}
```

```text
error[E0502]: cannot borrow `s` as immutable because it is also borrowed as mutable
 --> src/main.rs:4:14
  |
3 |     let r1 = &mut s;
  |              ------ mutable borrow occurs here
4 |     let r2 = &s;
  |              ^^ immutable borrow occurs here
5 |     println!("{r1} {r2}");
  |                -- mutable borrow later used here
```

O compilador não analisa se `r1` "de fato" escreve. Ele olha os tipos: `&mut` é exclusiva, então nenhum outro empréstimo pode coexistir com ela.

### E0506: atribuir a algo emprestado

```rust
// ❌ NÃO COMPILA
fn main() {
    let mut x = 10;
    let r = &x;
    x = 20;
    println!("{r}");
}
```

```text
error[E0506]: cannot assign to `x` because it is borrowed
 --> src/main.rs:4:5
  |
3 |     let r = &x;
  |             -- `x` is borrowed here
4 |     x = 20;
  |     ^^^^^^ `x` is assigned to here but it was already borrowed
5 |     println!("{r}");
  |                - borrow later used here
```

É o mesmo caso do `push`, sem chamada de método: a atribuição precisa de W em `x`, e `r` tirou W.

### E0596: alterar através de `&`

```rust
// ❌ NÃO COMPILA
fn add_suffix(s: &String) {
    s.push_str(", Esq.");
}

fn main() {
    let mut name = String::from("Fulano");
    add_suffix(&name);
    println!("{name}");
}
```

```text
error[E0596]: cannot borrow `*s` as mutable, as it is behind a `&` reference
 --> src/main.rs:2:5
  |
2 |     s.push_str(", Esq.");
  |     ^ `s` is a `&` reference, so it cannot be borrowed as mutable
  |
help: consider changing this to be a mutable reference
  |
1 | fn add_suffix(s: &mut String) {
  |                   +++
```

Em permissões: `*s` só tem R, e `push_str` precisa de W. Aqui o `help:` está certo, mas lembre de trocar também a chamada para `add_suffix(&mut name)`.

## Reborrow: passar `&mut` adiante não a consome

Uma `&mut T` não é `Copy` (se fosse, existiriam duas cópias exclusivas). Então era de se esperar que passá-la para uma função a movesse. Mas isto compila:

```rust
fn add_one(v: &mut Vec<i32>) {
    v.push(1);
}

fn main() {
    let mut v = Vec::new();
    let r = &mut v;
    add_one(r);
    add_one(r);
    r.push(2);
    println!("{v:?}");
}
```

```text
[1, 1, 2]
```

Em cada chamada, o compilador cria um **reempréstimo** (*reborrow*) implícito, `&mut *r`: uma nova `&mut` derivada de `r`, que vive só durante a chamada. Enquanto ela vive, `r` fica "suspensa". Quando a chamada termina, `r` volta a valer. É por isso que você pode passar a mesma `&mut` para várias funções em sequência.

## Two-phase borrows: por que `v.push(v.len())` compila

Pela regra, isto parece proibido: `push` pede `&mut v`, e o argumento `v.len()` pede `&v`.

```rust
fn main() {
    let mut v = vec![10, 20];
    v.push(v.len());
    println!("{v:?}");
}
```

```text
[10, 20, 2]
```

Compila porque o `&mut v` de uma chamada de método é feito em **duas fases** (*two-phase borrow*): primeiro ele é reservado, depois os argumentos são avaliados (aqui, `v.len()` lê `v` e termina), e só então o empréstimo exclusivo é ativado para executar o `push`. Não há conflito real, porque a leitura termina antes de a escrita começar. Não é uma exceção arbitrária: é o compilador seguindo a ordem real de avaliação.

## Iterator invalidation

Este é o bug em que o borrow checker mais brilha. Em C++, alterar um `vector` enquanto o percorre invalida o iterador do `for`:

```cpp
std::vector<int> v{1, 2, 3};
for (int x : v) {
    if (x % 2 == 1) {
        v.push_back(x * 10);  // invalida o iterador do for
    }
}
std::cout << v.size() << "\n";
```

Compila sem aviso. Sem sanitizer, imprimiu `4` (deveria ser 5, se fosse "acrescentar o dobro de cada ímpar"). Com o AddressSanitizer:

```text
==44466==ERROR: AddressSanitizer: heap-use-after-free on address 0x6020000000d4 ...
READ of size 4 at 0x6020000000d4 thread T0
    #0 0x000102c78b6c in main iter.cpp:6
```

Em Python não há memória liberada, mas o problema lógico continua. Remover itens de uma lista durante a iteração pula elementos em silêncio:

```python
nums = [1, 2, 2, 3]
for x in nums:
    if x == 2:
        nums.remove(x)
print(nums)
```

```text
[1, 2, 3]
```

Sobrou um `2`. Para `dict` e `set`, o Python detecta em runtime:

```text
RuntimeError: dictionary changed size during iteration
```

Em Rust, o problema não chega a rodar:

```rust
// ❌ NÃO COMPILA
fn main() {
    let mut v = vec![1, 2, 3];
    for x in &v {
        if *x % 2 == 1 {
            v.push(*x * 10);
        }
    }
    println!("{v:?}");
}
```

```text
error[E0502]: cannot borrow `v` as mutable because it is also borrowed as immutable
 --> src/main.rs:5:13
  |
3 |       for x in &v {
  |       -        --
  |       |        |
  |       |        immutable borrow occurs here
  |  _____|        immutable borrow later used here
  | |
4 | |         if *x % 2 == 1 {
5 | |             v.push(*x * 10);
  | |             ^^^^^^^^^^^^^^^ mutable borrow occurs here
6 | |         }
7 | |     }
  | |_____- this for loop borrows `v` immutably, preventing mutation within its body
  |
  = help: consider using an index-based loop instead, or collecting modifications into a separate collection
```

`for x in &v` cria um iterador que guarda um `&v` durante **o laço inteiro**. O rótulo que abraça as linhas 3 a 7 mostra essa região. O `help:` traz as duas saídas idiomáticas:

```rust
fn main() {
    // 1. Juntar as mudanças numa coleção separada e aplicar depois.
    let mut v = vec![1, 2, 3];
    let novos: Vec<i32> = v.iter().filter(|x| *x % 2 == 1).map(|x| x * 10).collect();
    v.extend(novos);
    println!("{v:?}");

    // 2. Laço por índice: nenhum empréstimo atravessa as iterações.
    let mut w = vec![1, 2, 3];
    for i in 0..w.len() {
        if w[i] % 2 == 1 {
            w.push(w[i] * 10);
        }
    }
    println!("{w:?}");
}
```

```text
[1, 2, 3, 10, 30]
[1, 2, 3, 10, 30]
```

No laço por índice, `0..w.len()` é calculado **uma vez**, antes do laço. Os elementos novos não são visitados, e por isso o laço termina. Se você quisesse visitá-los, a decisão teria de ficar explícita (um `while i < w.len()`).

> [!TIP]
> A primeira forma (juntar e depois aplicar) costuma ser a mais clara. A cadeia `iter()`/`filter`/`map`/`collect` é a mesma do kit mínimo do M01. O M07 aprofunda iteradores.

## Por que a regra existe (além da memória)

- **Sem *data races*.** Uma *data race* precisa de dois acessos ao mesmo dado, ao mesmo tempo, com pelo menos uma escrita. "Muitos `&` ou um `&mut`" impede isso por construção. No Módulo 11, essa mesma regra é o que deixa o compilador recusar código concorrente inseguro.
- **Otimização.** Como nada muda atrás de uma `&T` viva, o compilador pode guardar o valor num registrador sem reler a memória. Em C++, o otimizador muitas vezes não pode, porque qualquer ponteiro poderia ser um alias.
- **Raciocínio local.** Quando você lê uma função que recebe `&T`, sabe que o valor não muda durante a chamada, sem precisar olhar mais nada.

## Armadilhas comuns

**Achar que o empréstimo vai até o `}`.** Com NLL, ele vai até o último uso. Muitas vezes a correção é só mover uma linha: usar a referência antes de mutar.

**Esconder o último uso.** Uma referência guardada numa estrutura (um `Vec<&T>`, uma struct) vive enquanto a estrutura for usada. Se o erro aponta "later used here" numa linha inesperada, procure onde a referência foi parar.

**Achar que `for x in &v` "copia" o vetor.** Ele empresta `v` pelo laço todo. Dentro do corpo, `v` só pode ser lido.

| Código | Mensagem | Em permissões |
|---|---|---|
| E0502 | `cannot borrow ... as mutable because it is also borrowed as immutable` (ou o inverso) | `&` viva tirou W; ou `&mut` viva tirou R |
| E0499 | `cannot borrow ... as mutable more than once at a time` | `&mut` viva tirou tudo |
| E0506 | `cannot assign to ... because it is borrowed` | atribuição precisa de W |
| E0505 | `cannot move out of ... because it is borrowed` | move precisa de O |
| E0596 | `cannot borrow ... as mutable` | não há W: falta `mut` ou é `&` |

## Resumo

- Um empréstimo vive da criação até o **último uso** (NLL), e não até o fim do bloco.
- Há conflito quando a região de uma `&mut` cruza a de qualquer outro empréstimo do mesmo valor. Escrever ou não pela `&mut` não importa.
- Passar uma `&mut` para uma função faz um *reborrow* temporário, e a referência continua utilizável depois.
- *Two-phase borrows* deixam `v.push(v.len())` compilar: os argumentos são avaliados antes de o `&mut` ser ativado.
- *Iterator invalidation* vira E0502. Saídas: juntar as mudanças e aplicar depois, ou iterar por índice.

## Exercícios

Veja os exercícios 1 e 6 em [`exercicios.md`](exercicios.md).

## Leitura complementar

- [The Rust Programming Language, 4.2 References and Borrowing](https://doc.rust-lang.org/stable/book/ch04-02-references-and-borrowing.html)
- [Comprehensive Rust: Borrow Checking](https://google.github.io/comprehensive-rust/borrowing/borrowck.html) e [Borrow Errors](https://google.github.io/comprehensive-rust/borrowing/examples.html)
- [RFC 2094: non-lexical lifetimes](https://rust-lang.github.io/rfcs/2094-nll.html) (a proposta original do NLL, para quem quiser o detalhe)
- [rustc dev guide: two-phase borrows](https://rustc-dev-guide.rust-lang.org/borrow_check/two_phase_borrows.html)
- Código desta lição: [`codigo/emprestimos`](codigo/emprestimos/)

---

[← Lição anterior](01-referencias-e-emprestimos.md) · [Índice do módulo](README.md) · [Próxima lição →](03-slices-em-profundidade.md)

# 02 — Ponteiros crus

**Objetivos.** Ao final, você vai saber:

- criar ponteiros crus com `&raw const` e `&raw mut`, e o que é preciso para desreferenciá-los;
- usar as operações básicas: nulo, `as_ref`, `add`, `read`/`write`, `swap`, `Box::into_raw`/`from_raw` e `NonNull`;
- acessar um `static mut` e campos de struct `packed` sem criar referências inválidas;
- por que ponteiros crus não são `Send` nem `Sync`.

## Referências x ponteiros crus

Uma referência (`&T`, `&mut T`) carrega garantias: não é nula, está alinhada, aponta para um valor válido, e respeita as regras de empréstimo. Um ponteiro cru (`*const T`, `*mut T`) é só um endereço, como um ponteiro de C. Ele pode ser nulo, pendurado, desalinhado, e vários `*mut T` podem apontar para o mesmo lugar.

| | `&T` / `&mut T` | `*const T` / `*mut T` |
|---|---|---|
| nulo | nunca | pode ser |
| aponta para valor válido | sempre | não garantido |
| regras de empréstimo | verificadas | não verificadas |
| *lifetime* | sim | não tem |
| criar | seguro | seguro |
| desreferenciar | seguro | `unsafe` |

O `*const`/`*mut` não muda o que o ponteiro pode fazer em tempo de execução. É uma intenção, e você pode converter de um para o outro com `as` ou `.cast_mut()`/`.cast_const()`. A diferença entre os dois é parecida com `const int*` e `int*` em C.

## Criando: `&raw const` e `&raw mut`

O jeito atual (Rust 1.82) de obter o endereço de um lugar é o operador `&raw`:

```rust
let mut x = 10;
let p_escrita = &raw mut x;
let p_leitura = p_escrita as *const i32;
println!("endereço: {p_leitura:p}");

// SAFETY: os dois ponteiros vêm de `x`, que está viva até o fim da
// função, e nenhuma referência para `x` existe enquanto eles são usados.
unsafe {
    *p_escrita += 1;
    println!("valor: {}", *p_leitura);
}
```

```text
endereço: 0x16af9cfd4
valor: 11
```

Também dá para converter uma referência (`&x as *const i32`, ou simplesmente passar `&x` onde um `*const i32` é esperado). A diferença é que `&raw` **não cria uma referência intermediária**. Isso importa quando uma referência seria inválida: um campo desalinhado ou um `static mut`, logo abaixo. Material antigo usa as macros `addr_of!` e `addr_of_mut!`, que fazem o mesmo; prefira `&raw`.

> [!NOTE]
> Repare que `p_leitura` foi **derivado** de `p_escrita`, e não criado com outro `&raw const x`. Criar os dois direto de `x` e depois escrever por um e ler pelo outro parece inofensivo, mas é rejeitado pelo Miri no modelo padrão de aliasing. A lição 04 mostra a saída e explica por quê. Regra prática: **derive todos os ponteiros de uma mesma origem**.

## Nulo e `as_ref`

```rust
use std::ptr;

let nulo: *const i32 = ptr::null();
println!("é nulo? {}", nulo.is_null());

let x = 7;
let p = &raw const x;
// SAFETY: se não for nulo, `p` aponta para `x`, viva e inicializada.
let r = unsafe { p.as_ref() };
let n = unsafe { nulo.as_ref() };
println!("as_ref: {r:?} e {n:?}");
```

```text
é nulo? true
as_ref: Some(7) e None
```

`as_ref` (e `as_mut`) converte o ponteiro em `Option<&T>`: `None` se for nulo, `Some(&T)` caso contrário. É o jeito idiomático de tratar um ponteiro que pode ser nulo vindo de C (lição 06). O `unsafe` continua necessário, porque um ponteiro não nulo ainda pode estar pendurado.

## Aritmética: `add`

```rust
let numeros = [10, 20, 30, 40];
let inicio = numeros.as_ptr();
let mut soma = 0;
for i in 0..numeros.len() {
    // SAFETY: `i < numeros.len()`, então `inicio.add(i)` fica dentro do
    // array, que está vivo e não é alterado durante o laço.
    soma += unsafe { *inicio.add(i) };
}
println!("soma: {soma}");
```

```text
soma: 100
```

`p.add(n)` avança `n` **elementos** (não bytes), como `p + n` em C. Ele é `unsafe` por si só: o resultado precisa ficar dentro da mesma alocação (ou uma posição depois do fim). Para aritmética sem essa exigência, há `wrapping_add`, mas desreferenciar o resultado continua exigindo que ele aponte para algo válido.

Na prática, você raramente escreve esse laço: `numeros.iter().sum()` faz o mesmo, sem `unsafe`. Ponteiros e `add` aparecem quando você está implementando a estrutura, e não usando.

## `read`, `write` e `swap`

As funções de `std::ptr` operam sobre ponteiros sem criar referências:

```rust
let mut a = String::from("um");
let mut b = String::from("dois");
// SAFETY: `&mut a` e `&mut b` são válidos, alinhados e não se sobrepõem.
unsafe { ptr::swap(&mut a, &mut b) };
println!("trocados: {a} {b}");
```

```text
trocados: dois um
```

- `ptr::read(p)` copia o valor para fora **sem mover a posse**: se `T` tem `Drop`, agora há duas cópias, e você precisa garantir que só uma seja descartada.
- `ptr::write(p, v)` grava sem descartar o valor antigo (útil para memória não inicializada).
- `read_unaligned`/`write_unaligned` não exigem alinhamento.

Aqui, `ptr::swap` é só ilustração: `std::mem::swap(&mut a, &mut b)` é seguro e faz o mesmo.

## `Box::into_raw` e `Box::from_raw`

`Box::into_raw` entrega a posse de um valor no heap para um ponteiro cru. A partir daí, ninguém vai liberar a memória até alguém chamar `Box::from_raw`, que devolve a posse:

```rust
let caixa = Box::new(String::from("no heap"));
let p: *mut String = Box::into_raw(caixa);
// SAFETY: `p` veio de `Box::into_raw` e ainda não foi liberado.
unsafe { (*p).push_str(", via ponteiro") };
// SAFETY: `p` veio de `Box::into_raw` e é devolvido uma única vez.
let de_volta = unsafe { Box::from_raw(p) };
println!("{de_volta}");
```

```text
no heap, via ponteiro
```

É assim que um objeto Rust atravessa a fronteira com C: o C recebe o ponteiro, guarda, e devolve numa função de liberação (lição 06). Chamar `from_raw` duas vezes com o mesmo ponteiro é um *double free*, e esquecer de chamar é um vazamento.

`std::ptr::NonNull<T>` é um `*mut T` que nunca é nulo. Ele é a base de `Box`, `Vec` e `Rc` por dentro, e o `Option<NonNull<T>>` ocupa o mesmo espaço que um ponteiro (Módulo 04). A lição 03 usa `NonNull` para escrever um `Box` mínimo.

## `static mut`: por valor sim, por referência não

No Módulo 11, `println!("{}", CONTADOR)` com um `static mut` deu erro, porque o `println!` cria uma referência. Acessar **por valor** não cria referência e compila:

```rust
static mut CHAMADAS: u32 = 0;

fn contar_chamada() {
    // SAFETY: este programa só tem uma thread, então não há acesso simultâneo.
    // Ler e escrever por valor não cria referência para o `static mut`.
    unsafe { CHAMADAS += 1 };
}

fn main() {
    contar_chamada();
    contar_chamada();
    // SAFETY: uma thread só. O bloco `unsafe` devolve uma cópia do valor, e
    // o `println!` empresta a cópia, não o `static mut`.
    println!("chamadas: {}", unsafe { CHAMADAS });
    // Quando é preciso um endereço (para passar a C, por exemplo), use `&raw`.
    let p: *const u32 = &raw const CHAMADAS;
    println!("CHAMADAS mora em {p:p}");
}
```

```text
chamadas: 2
CHAMADAS mora em 0x104eacd50
```

Quando você precisa de um endereço, por exemplo para entregar um buffer global a uma função de C, `&raw const`/`&raw mut` dão um ponteiro sem passar por referência. Mesmo assim, o `// SAFETY:` precisa justificar a ausência de acesso simultâneo. Com mais de uma thread, use atômicos ou `Mutex` (Módulo 11), e não `static mut`.

## Structs `packed`

`#[repr(C, packed)]` remove o preenchimento (*padding*) entre os campos, como `__attribute__((packed))` em C. Isso aparece em formatos binários e protocolos. O problema: um campo `u32` pode ficar num endereço que não é múltiplo de 4, e uma referência desalinhada é UB mesmo que nunca seja usada. O compilador barra:

```rust
// ❌ NÃO COMPILA
#[repr(C, packed)]
struct Cabecalho {
    tipo: u8,
    tamanho: u32,
}
fn main() {
    let c = Cabecalho { tipo: 1, tamanho: 512 };
    let r = &c.tamanho;
    println!("{} {}", c.tipo, r);
}
```

```text
error[E0793]: reference to field of packed struct is unaligned
 --> src/main.rs:8:13
  |
8 |     let r = &c.tamanho;
  |             ^^^^^^^^^^
  |
  = note: this struct is 1-byte aligned, but the type of this field may require higher alignment
  = note: creating a misaligned reference is undefined behavior (even if that reference is never dereferenced)
  = help: copy the field contents to a local variable, or replace the reference with a raw pointer and use `read_unaligned`/`write_unaligned` (loads and stores via `*p` must be properly aligned even when using raw pointers)
```

O `help` dá as duas saídas, e o exemplo completo usa as duas:

```rust
let p = &raw const c.tamanho;
// SAFETY: `p` aponta para um campo inicializado de `c`; `read_unaligned`
// não exige alinhamento.
let tamanho = unsafe { p.read_unaligned() };
// Copiar o campo (`{ c.tipo }`) também funciona, porque não cria referência.
println!(
    "tipo {} tamanho {tamanho} (struct de {} bytes)",
    { c.tipo },
    size_of::<Cabecalho>()
);
```

```text
tipo 1 tamanho 512 (struct de 5 bytes)
```

As chaves em `{ c.tipo }` forçam uma cópia antes de o `println!` pegar a referência. São 5 bytes em vez de 8, porque não há preenchimento.

## Ponteiros crus não são `Send`

O Módulo 11 listou `*const T` e `*mut T` como nem `Send` nem `Sync`. O erro aparece assim:

```rust
// ❌ NÃO COMPILA
use std::thread;
fn main() {
    let mut x = 10;
    let p = &raw mut x;
    let h = thread::spawn(move || unsafe { *p += 1 });
    h.join().unwrap();
}
```

```text
error[E0277]: `*mut i32` cannot be sent between threads safely
   --> src/main.rs:5:27
    |
  5 |     let h = thread::spawn(move || unsafe { *p += 1 });
    |             ------------- -------^^^^^^^^^^^^^^^^^^^
    |             |             |
    |             |             `*mut i32` cannot be sent between threads safely
    |             |             within this `{closure@src/main.rs:5:27: 5:34}`
    |             required by a bound introduced by this call
    |
    = help: within `{closure@src/main.rs:5:27: 5:34}`, the trait `Send` is not implemented for `*mut i32`
...
```

O compilador não sabe para onde o ponteiro aponta nem quem mais acessa aquela memória, então não deixa enviá-lo. Aqui ele tem razão duas vezes: `x` vive na pilha do `main`, e a closure `'static` poderia sobreviver a ela. Um tipo que encapsula um ponteiro cru e **sabe** que o acesso é seguro pode prometer isso com `unsafe impl Send`, como na próxima lição.

## Comparando com C e C++

Ponteiros crus do Rust são, de propósito, quase iguais aos de C: aritmética em elementos, conversão livre entre tipos com `as`/`cast`, nulo permitido. A diferença é onde eles aparecem. Em C++, `T*` é o tipo do dia a dia (parâmetros, membros, `this`). Em Rust, quase todo código usa referências e ponteiros inteligentes, e os ponteiros crus ficam restritos às implementações de baixo nível e à fronteira com C.

E há um ponto em que Rust é mais exigente: o compilador barra até casts que em C++ seriam só "estilo ruim":

```rust
// ❌ NÃO COMPILA
fn main() {
    let x = 5;
    let p = &x as *const i32 as *mut i32;
    unsafe { *p = 6 };
    println!("{x}");
}
```

```text
error: assigning to `&T` is undefined behavior, consider using an `UnsafeCell`
 --> src/main.rs:4:14
  |
3 |     let p = &x as *const i32 as *mut i32;
  |             ---------------------------- casting happened here
4 |     unsafe { *p = 6 };
  |              ^^^^^^
  |
  = note: for more information, visit <https://doc.rust-lang.org/book/ch15-05-interior-mutability.html>
  = note: `#[deny(invalid_reference_casting)]` on by default
```

É o equivalente ao `const_cast` do C++ (Módulo 10): escrever através de algo que veio de um `&T` é UB, a menos que o dado esteja dentro de um `UnsafeCell` (lição 03).

Código completo em [`codigo/ponteiros-crus`](codigo/ponteiros-crus/src/main.rs).

## Armadilhas comuns

- **Ponteiro para uma variável que já saiu de escopo.** Um ponteiro cru não tem *lifetime*, então o compilador não reclama de `p = &raw const x` dentro de um bloco e `*p` fora dele. É o primeiro exemplo de UB da lição 04.
- **Criar uma referência sem perceber.** `&c.campo` num `packed`, `println!` num `static mut`, um método que recebe `&self`: todos criam referências, e referências têm exigências que ponteiros não têm. Na dúvida, use `&raw` e as funções de `std::ptr`.
- **`Box::from_raw` duas vezes.** É um double free. Depois de devolver a posse, zere ou descarte o ponteiro.
- **Usar ponteiros onde um índice resolve.** `v[i]` com verificação é quase sempre tão rápido quanto `*p.add(i)`, porque o compilador elimina verificações redundantes. Meça antes de trocar (Módulo 18).

## Resumo

- `*const T`/`*mut T`: endereços sem garantias. Criar é seguro; desreferenciar exige `unsafe`.
- `&raw const`/`&raw mut` criam ponteiros sem passar por referência; derive todos os ponteiros de uma mesma origem.
- `as_ref`/`as_mut` → `Option<&T>`; `add` em elementos; `read`/`write`/`swap` sem referências; `Box::into_raw`/`from_raw` para entregar e devolver a posse; `NonNull` para ponteiros não nulos.
- `static mut` por valor compila; por referência é erro na 2024. Campos `packed`: `read_unaligned` ou cópia.
- Ponteiros crus não são `Send`/`Sync`.

## Exercícios

Veja o exercício 2 em [`exercicios.md`](exercicios.md).

## Leitura complementar

- [The Rust Programming Language, 20.1: Dereferencing a Raw Pointer](https://doc.rust-lang.org/stable/book/ch20-01-unsafe-rust.html#dereferencing-a-raw-pointer)
- [Documentação de `std::ptr`](https://doc.rust-lang.org/std/ptr/index.html) (em especial a seção *Safety*)
- [Documentação dos métodos de ponteiro](https://doc.rust-lang.org/std/primitive.pointer.html)
- [The Rust Reference: Raw borrow operators](https://doc.rust-lang.org/reference/expressions/operator-expr.html#raw-borrow-operators)

---

[← Lição anterior](01-o-que-unsafe-permite.md) · [Índice do módulo](README.md) · [Próxima lição →](03-abstracoes-seguras.md)

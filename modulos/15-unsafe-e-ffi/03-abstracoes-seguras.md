# 03 — Abstrações seguras sobre `unsafe`

**Objetivos.** Ao final, você vai saber:

- escrever uma função segura que usa `unsafe` por dentro, como `split_at_mut`;
- o que é uma invariante e por que a fronteira de segurança é o **módulo**, e não o bloco `unsafe`;
- como `UnsafeCell` permite mutabilidade interior, construindo uma `Cell` simplificada;
- quando `unsafe impl Send`/`Sync` é legítimo, com um `Box` mínimo.

## O padrão: `unsafe` dentro, API segura fora

Quase todo `unsafe` em Rust segue o mesmo desenho: uma operação que o compilador não consegue provar fica numa função pequena, que **verifica tudo o que precisa** e só então usa o superpoder. Quem chama vê uma função comum, sem `unsafe`, que não tem como ser usada errado.

O exemplo clássico é o `split_at_mut` do Módulo 03. Tente escrevê-lo em Rust seguro:

```rust
// ❌ NÃO COMPILA
fn split_at_mut(valores: &mut [i32], meio: usize) -> (&mut [i32], &mut [i32]) {
    let len = valores.len();
    assert!(meio <= len);
    (&mut valores[..meio], &mut valores[meio..])
}
```

```text
error[E0499]: cannot borrow `*valores` as mutable more than once at a time
 --> src/main.rs:4:33
  |
1 | fn split_at_mut(valores: &mut [i32], meio: usize) -> (&mut [i32], &mut [i32]) {
  |                          - let's call the lifetime of this reference `'1`
...
4 |     (&mut valores[..meio], &mut valores[meio..])
  |     ----------------------------^^^^^^^---------
  |     |     |                     |
  |     |     |                     second mutable borrow occurs here
  |     |     first mutable borrow occurs here
  |     returning this value requires that `*valores` is borrowed for `'1`
  |
  = help: use `.split_at_mut(position)` to obtain two mutable non-overlapping sub-slices
```

O código está correto: as duas metades não se sobrepõem. Mas o *borrow checker* raciocina sobre `valores` inteiro, e não sobre faixas de índices. Ele vê dois `&mut` do mesmo slice e recusa. (O `help` sugere a função da std, que é exatamente o que estamos escrevendo.)

Com ponteiros crus:

```rust
use std::slice;

/// Divide `valores` em `[..meio]` e `[meio..]`, duas fatias mutáveis que não
/// se sobrepõem.
///
/// # Panics
///
/// Se `meio > valores.len()`.
pub fn dividir_mut<T>(valores: &mut [T], meio: usize) -> (&mut [T], &mut [T]) {
    let len = valores.len();
    // Esta verificação é o que torna a função segura: sem ela, um `meio`
    // grande demais faria a segunda fatia sair do slice.
    assert!(meio <= len, "meio ({meio}) maior que o tamanho ({len})");
    let ptr = valores.as_mut_ptr();
    // SAFETY: `ptr` é válido para `len` elementos, porque vem de `valores`.
    // Como `meio <= len`, as faixas `[0, meio)` e `[meio, len)` ficam dentro
    // do slice e não se sobrepõem. As duas fatias herdam o lifetime do
    // `&mut` original, então ninguém mais acessa `valores` enquanto elas vivem.
    unsafe {
        (
            slice::from_raw_parts_mut(ptr, meio),
            slice::from_raw_parts_mut(ptr.add(meio), len - meio),
        )
    }
}
```

`slice::from_raw_parts_mut(ptr, n)` monta um `&mut [T]` a partir de um ponteiro e de um tamanho. É `unsafe` porque não tem como saber se aqueles `n` elementos existem e se mais ninguém os acessa. O comentário `SAFETY` responde a cada uma dessas perguntas.

Três coisas tornam a função segura, e vale olhar cada uma:

1. **O `assert!`.** Sem ele, `dividir_mut(&mut v, 100)` criaria uma fatia fora do slice. Uma função segura precisa ser correta para **qualquer** entrada que compile, e não só para as entradas "razoáveis".
2. **A assinatura.** O retorno `(&mut [T], &mut [T])` está ligado, pela elisão, ao lifetime do `&mut [T]` recebido. Enquanto as metades existirem, o `valores` original fica emprestado, e o borrow checker volta a trabalhar para você do lado de fora.
3. **A ausência de sobreposição**, que é o fato que o checker não consegue ver e que o comentário afirma.

```rust
let mut v = [1, 2, 3, 4, 5];
let (a, b) = dividir_mut(&mut v, 2);
a[0] = 10;
b[2] = 50;
assert_eq!(v, [10, 2, 3, 4, 50]);
```

## Invariantes e a fronteira do módulo

Uma *invariante* é uma condição que o tipo promete manter sempre: "`len <= capacidade`", "`pos` nunca passa do fim", "este ponteiro veio de `Box::into_raw`". Código `unsafe` costuma depender delas. Veja este leitor de bytes:

```rust
/// Invariante: `pos <= dados.len()`. Os campos são privados, então só o
/// código deste módulo pode quebrar a invariante.
pub struct Leitor {
    dados: Vec<u8>,
    pos: usize,
}

impl Leitor {
    pub fn proximo(&mut self) -> Option<u8> {
        if self.pos == self.dados.len() {
            return None;
        }
        // SAFETY: pela invariante, `pos <= len`, e acabamos de ver que
        // `pos != len`; então `pos < len` e o índice é válido.
        let b = unsafe { *self.dados.get_unchecked(self.pos) };
        self.pos += 1;
        Some(b)
    }

    /// Avança `n` bytes, sem passar do fim.
    pub fn pular(&mut self, n: usize) {
        self.pos = (self.pos + n).min(self.dados.len());
    }
}
```

O `unsafe` está em `proximo`, mas a correção dele depende de `pular`, uma função **sem nenhum `unsafe`**. Troque a última linha por `self.pos += n;` e o programa continua compilando, sem aviso. Um teste com `pular(100)` seguido de `proximo()` mostra o que acontece:

```text
thread 'buffer::tests::pular_alem_do_fim' (370830) panicked at src/buffer.rs:24:38:
unsafe precondition(s) violated: slice::get_unchecked requires that the index is within the slice

This indicates a bug in the program. This Undefined Behavior check is optional, and cannot be relied on for safety.
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
thread caused non-unwinding panic. aborting.
```

Em modo debug, a std confere algumas pré-condições de funções `unsafe` e aborta o processo quando elas são violadas. A própria mensagem avisa que a verificação é **opcional**: em `--release`, o mesmo teste morreu com `signal: 5, SIGTRAP`, e em outro programa poderia simplesmente ler lixo. O Miri (lição 04) pega o problema nos dois modos.

A lição é a do Rustonomicon: **a fronteira de segurança é o módulo, e não o bloco `unsafe`**. Qualquer código que consegue mexer em `pos` pode quebrar a invariante, e por isso os campos são privados. Ao revisar um `unsafe`, revise também tudo no mesmo módulo que toca os campos de que ele depende.

## `UnsafeCell`: mutabilidade interior por dentro

No Módulo 10, `Cell` e `RefCell` permitiam alterar um valor através de `&self`. E a lição anterior mostrou que escrever através de algo que veio de um `&T` é UB (`invalid_reference_casting`). Como as duas coisas convivem?

Pela única exceção da regra: `std::cell::UnsafeCell<T>`. O compilador sabe que o conteúdo de um `UnsafeCell` pode mudar mesmo quando há `&` para ele, e não faz otimizações supondo o contrário. Seu método `get(&self) -> *mut T` devolve um ponteiro cru para o conteúdo. Toda mutabilidade interior da std (`Cell`, `RefCell`, `Mutex`, os atômicos, `OnceLock`) é construída em cima dele.

Uma `Cell` simplificada:

```rust
use std::cell::UnsafeCell;

/// Valor `Copy` que pode ser trocado através de `&self`.
pub struct Celula<T: Copy> {
    valor: UnsafeCell<T>,
}

impl<T: Copy> Celula<T> {
    pub fn new(valor: T) -> Self {
        Celula { valor: UnsafeCell::new(valor) }
    }

    pub fn get(&self) -> T {
        // SAFETY: `Celula` não é `Sync`, então só esta thread acessa o valor.
        // Nenhuma referência para o conteúdo sai deste método: lemos uma cópia.
        unsafe { *self.valor.get() }
    }

    pub fn set(&self, novo: T) {
        // SAFETY: como em `get`. Nenhum `&T` para o conteúdo existe agora,
        // porque a API nunca entrega um; então escrever não invalida ninguém.
        unsafe { *self.valor.get() = novo };
    }
}
```

```rust
let c = Celula::new(1);
let a = &c;
let b = &c;
a.set(2);
b.set(b.get() + 1);
assert_eq!(c.get(), 3);
```

O raciocínio de segurança tem duas partes, e as duas estão nos comentários:

- **Uma thread só.** `UnsafeCell<T>` não é `Sync`, então `Celula<T>` também não é (auto traits, Módulo 11). O compilador impede que duas threads compartilhem a mesma `Celula`, e não há data race.
- **Nenhuma referência interna escapa.** Se houvesse um método `fn referencia(&self) -> &T`, alguém poderia guardar esse `&T` e chamar `set` depois, mudando o valor por baixo de uma referência compartilhada. É por isso que a `Cell` da std só funciona com cópias (`get` exige `T: Copy`), e o `RefCell` precisa do contador em tempo de execução.

## Implementando `Send` à mão: um `Box` mínimo

O Módulo 11 avisou que `unsafe impl Send` só faz sentido para tipos que encapsulam ponteiros crus. Este é um deles:

```rust
use std::ops::{Deref, DerefMut};
use std::ptr::NonNull;

/// Dono de um `T` no heap.
///
/// Invariante: `ptr` veio de `Box::into_raw` e é liberado só no `Drop`.
pub struct MeuBox<T> {
    ptr: NonNull<T>,
}

impl<T> MeuBox<T> {
    pub fn new(valor: T) -> Self {
        let cru = Box::into_raw(Box::new(valor));
        // SAFETY: `Box::into_raw` nunca devolve ponteiro nulo.
        let ptr = unsafe { NonNull::new_unchecked(cru) };
        MeuBox { ptr }
    }
}

impl<T> Deref for MeuBox<T> {
    type Target = T;
    fn deref(&self) -> &T {
        // SAFETY: pela invariante, `ptr` aponta para um `T` vivo, e o `&T`
        // devolvido empresta `self`, então não vive mais que o `MeuBox`.
        unsafe { self.ptr.as_ref() }
    }
}

impl<T> DerefMut for MeuBox<T> {
    fn deref_mut(&mut self) -> &mut T {
        // SAFETY: como em `deref`; o `&mut self` garante acesso exclusivo.
        unsafe { self.ptr.as_mut() }
    }
}

impl<T> Drop for MeuBox<T> {
    fn drop(&mut self) {
        // SAFETY: `ptr` veio de `Box::into_raw` e o `drop` roda uma vez só.
        unsafe { drop(Box::from_raw(self.ptr.as_ptr())) };
    }
}
```

`NonNull<T>` não é `Send` nem `Sync`, então o compilador não deriva esses traits para `MeuBox<T>`, e `thread::spawn(move || ...)` com um `MeuBox` daria o E0277 da lição anterior. Mas nós sabemos algo que o compilador não sabe: `MeuBox<T>` é o **único dono** do `T`, exatamente como `Box<T>`. Enviá-lo para outra thread é tão seguro quanto enviar o próprio `T`:

```rust
// SAFETY: `MeuBox<T>` tem posse exclusiva do `T`; nada mais aponta para ele.
unsafe impl<T: Send> Send for MeuBox<T> {}
// SAFETY: `&MeuBox<T>` só dá acesso a `&T`, então é seguro se `T: Sync`.
unsafe impl<T: Sync> Sync for MeuBox<T> {}
```

Os *bounds* `T: Send` e `T: Sync` são essenciais. Sem eles, `MeuBox<Rc<i32>>` seria `Send`, e duas threads poderiam mexer no contador do mesmo `Rc`. Um `unsafe impl` errado não quebra o seu código: quebra o de quem usa o seu tipo, em outro lugar, sem nenhum `unsafe` à vista.

```rust
let b = MeuBox::new(vec![1, 2, 3]);
let soma = std::thread::spawn(move || b.iter().sum::<i32>())
    .join()
    .unwrap();
assert_eq!(soma, 6);
```

Código completo, com testes, em [`codigo/abstracoes-seguras`](codigo/abstracoes-seguras/src/lib.rs). Todos os testes passam também no Miri (lição 04):

```text
running 8 tests
test buffer::tests::le_e_pula ... ok
test buffer::tests::pular_alem_do_fim ... ok
test celula::tests::troca_por_referencia_compartilhada ... ok
test dividir::tests::divide_e_altera_as_duas_partes ... ok
test dividir::tests::extremos ... ok
test dividir::tests::meio_invalido - should panic ... ok
test meu_box::tests::envia_para_outra_thread ... ok
test meu_box::tests::usa_como_box ... ok
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.04s
```

## Comparando com C++

Em C++, a mesma ideia existe por convenção: uma classe com membros privados que mantém uma invariante (`std::vector` mantém `size() <= capacity()`). A diferença é o que acontece fora da classe. Em C++, o usuário de `std::vector` ainda pode causar UB com uma API pública: `v[100]`, uma referência guardada antes de um `push_back`, um iterador invalidado. Em Rust, a meta de uma abstração segura é mais forte: **nenhum uso que compile pode causar UB**. `Vec::get_unchecked` existe, mas é `unsafe`, e `v[100]` dá panic em vez de ler lixo.

Essa propriedade tem nome no Rustonomicon: uma API é *sound* (correta) quando nenhum código seguro consegue usá-la para causar UB. Uma única função segura que permita isso torna a crate inteira *unsound*, e isso é tratado como bug grave, com direito a aviso de segurança no RustSec.

## Armadilhas comuns

- **Esquecer uma entrada.** A função segura precisa funcionar para todo valor que compila: índices fora da faixa, slices vazios, `meio == len`, `T` de tamanho zero. Escreva testes para as bordas e rode no Miri.
- **Campos públicos num tipo com invariante.** `pub pos: usize` deixaria qualquer crate quebrar o `Leitor`. Mantenha privados os campos de que o `unsafe` depende.
- **Devolver referências com lifetime errado.** Uma assinatura como `fn get(&self) -> &'static T` sobre dados do `self` compila (o `unsafe` deixa), mas deixa a referência viver mais que o dono. Prefira lifetimes elididos, ligados ao `self`.
- **`unsafe impl Send` sem bounds.** `unsafe impl<T> Send for MeuBox<T>` tornaria `MeuBox<Rc<_>>` enviável. Repita o raciocínio do `Box` da std: `T: Send` para `Send`, `T: Sync` para `Sync`.

## Resumo

- Uma abstração segura verifica tudo o que o `unsafe` exige e expõe uma API que nenhum código seguro consegue usar errado (*sound*).
- `split_at_mut` usa `from_raw_parts_mut` porque o checker não raciocina sobre faixas de índices; o `assert!` e o lifetime do retorno é que tornam a função segura.
- A fronteira de segurança é o módulo: código sem `unsafe` que mexe nos campos também precisa manter as invariantes.
- `UnsafeCell` é a única forma de mudar dados através de `&`; `Cell`, `RefCell`, `Mutex` e atômicos são construídos sobre ele.
- `unsafe impl Send`/`Sync` é para tipos que encapsulam ponteiros crus e sabem que o acesso é seguro, com os bounds certos.

## Exercícios

Veja os exercícios 2 e 3 em [`exercicios.md`](exercicios.md).

## Leitura complementar

- [The Rust Programming Language, 20.1: Creating a Safe Abstraction over Unsafe Code](https://doc.rust-lang.org/stable/book/ch20-01-unsafe-rust.html#creating-a-safe-abstraction-over-unsafe-code)
- [The Rustonomicon: How Safe and Unsafe Interact](https://doc.rust-lang.org/nomicon/safe-unsafe-meaning.html) e [Working with Unsafe](https://doc.rust-lang.org/nomicon/working-with-unsafe.html)
- [Documentação de `UnsafeCell`](https://doc.rust-lang.org/std/cell/struct.UnsafeCell.html)
- [The Rustonomicon: Send and Sync](https://doc.rust-lang.org/nomicon/send-and-sync.html)

---

[← Lição anterior](02-ponteiros-crus.md) · [Índice do módulo](README.md) · [Próxima lição →](04-comportamento-indefinido-e-miri.md)

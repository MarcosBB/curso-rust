# Soluções — Módulo 15

Enunciados em [`modulos/15-unsafe-e-ffi/exercicios.md`](../../modulos/15-unsafe-e-ffi/exercicios.md).

Cada solução é um projeto Cargo com os testes do enunciado. Para conferir:

```bash
cd nome-do-projeto
cargo test
cargo +nightly miri test   # ex2, ex3 e ex5 (o Miri não executa C)
```

## 1. Atualizando para a edition 2024

Projeto: [`ex1-edition-2024/`](ex1-edition-2024/src/lib.rs).

A primeira compilação mostra três erros e um aviso:

```text
error: extern blocks must be unsafe
error: unsafe attribute used without unsafe
error[E0133]: call to unsafe function `abs` is unsafe and requires unsafe block
warning[E0133]: call to unsafe function `strlen` is unsafe and requires unsafe block
```

Os quatro se resolvem como nas lições: `unsafe extern "C"`, `#[unsafe(no_mangle)]`, um bloco `unsafe` em volta de `abs` e outro em volta de `strlen` (dentro da `unsafe fn`, porque na 2024 o corpo dela não é mais um bloco `unsafe` implícito). Aí aparece o quinto:

```text
error: creating a shared reference to mutable static
  --> src/lib.rs:23:15
   |
23 |     unsafe { *&MAIOR_DISTANCIA }
   |               ^^^^^^^^^^^^^^^^ shared reference to mutable static
...
help: use `&raw const` instead to create a raw pointer
   |
23 |     unsafe { *&raw const MAIOR_DISTANCIA }
   |                +++++++++
```

Aplicar o `help` faz tudo compilar e os quatro testes passarem. Mas `distancia` lê e escreve `MAIOR_DISTANCIA` a partir de 8 threads, sem sincronização: é uma data race, UB como o caso 7 da lição 04. O compilador só reclamou da referência; a data race passou porque estava dentro de `unsafe`, que é exatamente onde ele confia em você. (O Miri não ajuda aqui: ele para na chamada a `abs`, uma função de C, com `can't call foreign function`.)

A correção de verdade não tem `unsafe`:

```rust
static MAIOR_DISTANCIA: AtomicI32 = AtomicI32::new(0);

pub fn distancia(a: i32, b: i32) -> i32 {
    let diferenca = a.wrapping_sub(b);
    assert!(diferenca != i32::MIN, "distância não cabe em i32");
    // SAFETY: o único valor proibido para `abs` é `i32::MIN`, excluído acima.
    let d = unsafe { abs(diferenca) };
    MAIOR_DISTANCIA.fetch_max(d, Ordering::Relaxed);
    d
}
```

`fetch_max` faz "ler, comparar e gravar o maior" numa operação atômica só. Com o `if d > X { X = d }` do código original, mesmo trocando por `load`/`store` atômicos, duas threads poderiam ler o mesmo valor antigo e a menor gravar por último. `Relaxed` basta porque o valor não protege nenhum outro dado (Módulo 11).

Sobre `abs`: o caminho mais curto seria `safe fn abs`, como no Rust Book. A solução mantém `abs` como `unsafe` porque `abs(INT_MIN)` é UB em C, e exclui esse caso antes de chamar. O `wrapping_sub` evita que `a - b` dê panic por overflow em debug, e o `assert!` pega o único resultado proibido.

## 2. Dois empréstimos mutáveis

Projeto: [`ex2-fatias-mut/`](ex2-fatias-mut/src/lib.rs).

A tentativa segura cai no erro da lição 03:

```text
error[E0499]: cannot borrow `v[_]` as mutable more than once at a time
 --> src/lib.rs:6:22
  |
1 | pub fn primeiro_e_ultimo_mut<T>(v: &mut [T]) -> Option<(&mut T, &mut T)> {
  |                                    - let's call the lifetime of this reference `'1`
...
6 |     Some((&mut v[0], &mut v[len - 1]))
  |     -----------------^^^^^^^^^^^^^^^--
  |     |     |          |
  |     |     |          second mutable borrow occurs here
  |     |     first mutable borrow occurs here
  |     returning this value requires that `v[_]` is borrowed for `'1`
  |
  = help: use `.split_at_mut(position)` to obtain two mutable non-overlapping sub-slices
```

A solução pega um único ponteiro e deriva tudo dele:

```rust
pub fn primeiro_e_ultimo_mut<T>(v: &mut [T]) -> Option<(&mut T, &mut T)> {
    let len = v.len();
    if len < 2 {
        return None;
    }
    let p = v.as_mut_ptr();
    // SAFETY: `len >= 2`, então os índices 0 e `len - 1` são válidos e
    // diferentes: as duas referências não se sobrepõem. Elas herdam o lifetime
    // do `&mut` recebido, que fica emprestado enquanto elas existirem.
    unsafe { Some((&mut *p, &mut *p.add(len - 1))) }
}
```

O `len < 2` não é só para o caso vazio: com um elemento só, `0` e `len - 1` seriam o mesmo índice, e devolver dois `&mut` para ele seria o caso 3 da lição 04.

`dividir_em_tres` faz o mesmo com três `slice::from_raw_parts_mut`, depois de `assert!(i <= j && j <= len, ...)`. As faixas `[0, i)`, `[i, j)` e `[j, len)` ficam dentro do slice e não se sobrepõem justamente por causa dessa ordem. Os dois testes `should_panic` conferem que a verificação existe. Todos os testes passam também no Miri.

Fora do exercício, as duas funções se escrevem sem `unsafe`: `split_first_mut` + `split_last_mut`, ou `get_disjoint_mut([0, len - 1])`, e dois `split_at_mut` seguidos.

## 3. Caça ao UB

Projeto: [`ex3-caca-ao-ub/`](ex3-caca-ao-ub/src/lib.rs).

Com o código inicial, `cargo test` aborta o processo de testes inteiro:

```text
running 4 tests
thread 'tests::le_no_meio_de_um_buffer' (413609) panicked at src/lib.rs:18:22:
misaligned pointer dereference: address must be a multiple of 0x4 but is 0x16fc5663b
thread caused non-unwinding panic. aborting.
error: test failed, to rerun pass `--lib`
```

Em `--release`, essa verificação de debug some, e o resultado é outro:

```text
test tests::estende_vazio ... ok
test tests::le_no_meio_de_um_buffer ... ok
test tests::incrementa ... ok
test tests::estende ... FAILED
assertion `left == right` failed
  left: [1, 2, 3, 10, -285383890, 10240]
 right: [1, 2, 3, 10, 20, 30]
```

Os números depois do `10` mudam de uma execução para outra (em outra, saiu `[1, 2, 3, 10, 0, 20]`). Três sintomas diferentes, e nenhum aponta para a causa. O Miri aponta:

**`estender_multiplicado`:**

```text
error: Undefined Behavior: in-bounds pointer arithmetic failed: alloc42750 has been freed, so this pointer is dangling
  --> src/lib.rs:7:27
   |
 7 |         let x = unsafe { *inicio.add(i) };
   |                           ^^^^^^^^^^^^^ Undefined Behavior occurred here
...
help: alloc42750 was allocated here:
  --> src/lib.rs:40:21
   |
40 |         let mut v = vec![1, 2, 3];
   |                     ^^^^^^^^^^^^^
help: alloc42750 was deallocated here:
  --> src/lib.rs:8:9
   |
 8 |         v.push(x * fator);
   |         ^^^^^^^^^^^^^^^^^
```

O primeiro `push` estourou a capacidade (3), o `Vec` realocou, e `inicio` ficou apontando para o buffer antigo, já liberado. O `SAFETY` dizia "`i < n`", o que é verdade, mas não era a condição que importava. A correção não precisa de `unsafe`: `let x = v[i];` lê sempre o buffer atual. O `v.reserve(n)` antes do laço evita realocações no meio, mas é só otimização: sem ele, a versão segura continua correta. (Em C++, é o mesmo bug do `push_back` da lição 04.)

**`ler_u32`:**

```text
error: Undefined Behavior: accessing memory based on pointer with alignment 2, but alignment 4 is required
  --> src/lib.rs:18:22
   |
18 |     let x = unsafe { *(bytes.as_ptr() as *const u32) };
```

É o mesmo diagnóstico do debug: desalinhamento. `&pacote[1..]` começa num endereço ímpar, e desreferenciar um `*const u32` exige endereço múltiplo de 4 (em ARM e x86 a leitura desalinhada até funciona no hardware, e por isso o release "passou"; mas é UB, e o compilador pode gerar instruções que supõem alinhamento). A correção segura:

```rust
bytes.first_chunk::<4>().map(|b| u32::from_le_bytes(*b))
```

`first_chunk` devolve `Option<&[u8; 4]>` (o `None` cobre o caso de menos de 4 bytes), e `from_le_bytes` monta o número sem exigência de alinhamento. Com `unsafe`, a alternativa seria `ptr.cast::<u32>().read_unaligned()`.

**`incrementar_dois`:** passa em debug e em release. Só o Miri pega:

```text
error: Undefined Behavior: attempting a read access using <134899> at alloc42463[0x4], but that tag does not exist in the borrow stack for this location
  --> src/lib.rs:29:9
   |
29 |         *a += 1;
   |         ^^^^^^^ this error occurs as part of an access at alloc42729[0x4..0x8]
...
help: <134899> was created by a SharedReadWrite retag at offsets [0x4..0x8]
  --> src/lib.rs:25:13
   |
25 |     let a = &mut v[i] as *mut i32;
   |             ^^^^^^^^^
help: <134899> was later invalidated at offsets [0x4..0x8] by a Unique retag
  --> src/lib.rs:26:13
   |
26 |     let b = &mut v[j] as *mut i32;
   |             ^^^^^^^^^
```

O offset `0x4` é o elemento 1: o erro está na chamada `incrementar_dois(&mut v, 1, 1)`. `&mut v[j]` cria um novo empréstimo exclusivo, que invalida o ponteiro `a` derivado do empréstimo anterior. Com Tree Borrows (`MIRIFLAGS=-Zmiri-tree-borrows`), o Miri também rejeita. A correção é a regra prática da lição 02, derivar os dois de uma mesma origem:

```rust
let p = v.as_mut_ptr();
unsafe {
    *p.add(i) += 1;
    *p.add(j) += 1;
}
```

Aqui também dava para não usar `unsafe` nenhum: `v[i] += 1; v[j] += 1;` faz exatamente o mesmo, inclusive com `i == j`. A solução mantém a versão com ponteiro para mostrar a correção, mas, num código real, a versão segura é a resposta certa.

Com as três correções, `cargo test`, `cargo test --release` e o Miri (nos dois modelos) passam.

## 4. Uma biblioteca C

Projeto: [`ex4-chamando-c/`](ex4-chamando-c/src/lib.rs), com [`build.rs`](ex4-chamando-c/build.rs) e [`csrc/texto.c`](ex4-chamando-c/csrc/texto.c).

O `build.rs` é o mínimo:

```rust
fn main() {
    cc::Build::new().file("csrc/texto.c").compile("texto");
    println!("cargo::rerun-if-changed=csrc/texto.c");
}
```

As declarações espelham o C com os tipos de `std::ffi` (`size_t` vira `usize`, `int` vira `c_int`, `char*` vira `*mut c_char`), e cada wrapper cuida de uma conversão:

- `contar_vogais`: `CString::new(texto)?` acrescenta o `'\0'` e já devolve o `NulError` exigido para `"a\0e"`. O `CString` fica numa variável enquanto o C usa o ponteiro.
- `inverter`: `bytes.as_mut_ptr().cast::<c_char>()` e `bytes.len()`. O `cast` é necessário porque `c_char` é `i8` em algumas plataformas e `u8` em outras; os dois têm o mesmo tamanho e alinhamento, e o C só troca bytes de lugar.
- `maior`: o C usa o padrão "código de status + ponteiro de saída". O wrapper cria um `c_int` local, passa `&raw mut saida` e transforma o par em `Option`: `(r == 0).then_some(saida)`.

O teste de vogais espera 6 em `"Programação em Rust"`, e não 7: o C conta bytes ASCII, e o `ã` em UTF-8 são dois bytes (`0xC3 0xA3`), nenhum deles uma vogal. Uma API C feita para ASCII não "entende" UTF-8, e o wrapper herda essa limitação. Ela poderia ser documentada, ou resolvida no lado Rust.

## 5. Uma pilha para C

Projeto: [`ex5-pilha-para-c/`](ex5-pilha-para-c/src/lib.rs), com [`include/pilha.h`](ex5-pilha-para-c/include/pilha.h) e [`c/main.c`](ex5-pilha-para-c/c/main.c).

`ex_media` mostra a ordem de verificação que toda função exportada deveria seguir: primeiro os ponteiros nulos, depois os valores inválidos, e só então o `unsafe`, que converte o par (ponteiro, tamanho) num slice comum:

```rust
// SAFETY: `v` não é nulo e, pelo contrato, aponta para `n` valores.
let valores = unsafe { std::slice::from_raw_parts(v, n) };
let media = valores.iter().sum::<f64>() / n as f64;
```

Para a pilha, `as_mut()`/`as_ref()` resolvem o nulo e o acesso de uma vez:

```rust
pub unsafe extern "C" fn ex_pilha_empilhar(p: *mut Pilha, x: i64) {
    // SAFETY: pelo contrato, `p` é nulo (`None`) ou uma pilha viva.
    if let Some(p) = unsafe { p.as_mut() } {
        p.itens.push(x);
    }
}
```

Em `ex_pilha_desempilhar`, a ordem importa para cumprir a especificação "não mexe em `*saida`": o `pop` acontece primeiro, e a escrita só no caso `Some`. A liberação é o par do `Box::into_raw` da criação, com `Box::from_raw` dentro de `if !p.is_null()`.

Nenhuma função tem `catch_unwind` porque nenhuma pode dar panic: não há índices, `unwrap` nem divisão por inteiro (`f64` dividido por zero dá `inf`, e o caso `n == 0` já foi excluído). O `Vec::push` pode dar panic por falta de memória, mas isso aborta o processo de qualquer forma.

Todos os testes passam no `cargo test` e no Miri. O desafio opcional, compilado contra a `staticlib`:

```bash
cargo build --release
cc c/main.c -Iinclude target/release/libex5_pilha_para_c.a -o pilha
./pilha
```

```text
media = 3.00
tamanho = 3
desempilhou 300
desempilhou 200
desempilhou 100
```

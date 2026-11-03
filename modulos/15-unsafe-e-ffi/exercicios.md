# Exercícios — Módulo 15

Soluções em [`solucoes/15-unsafe-e-ffi`](../../solucoes/15-unsafe-e-ffi/).

Em todos os exercícios, crie o projeto com `cargo new --lib nome`, acrescente `rust-version = "1.96"` no `Cargo.toml` e cole o código inicial em `src/lib.rs`. O objetivo final é sempre:

- `cargo test` passando;
- `CARGO_BUILD_WARNINGS=deny cargo clippy --all-targets` sem avisos;
- todo bloco `unsafe` com um comentário `// SAFETY:` que explique por que ele está correto;
- nos exercícios 2, 3 e 5, também `cargo +nightly miri test` sem erros (instale o Miri como na [lição 04](04-comportamento-indefinido-e-miri.md#miri)).

## 1. Atualizando para a edition 2024 (fácil, faça compilar)

Lição: [01](01-o-que-unsafe-permite.md), [02](02-ponteiros-crus.md), [05](05-chamando-c.md) e [06](06-expondo-rust-para-c.md).

`cargo new --lib edicao`. Este código foi escrito para a edition 2021 e não compila na 2024. Conserte todos os erros e avisos. Atenção: **seguir cegamente o `help` do compilador não basta**. Um dos testes usa 8 threads, e há um problema que o compilador não aponta como erro.

```rust
use std::ffi::c_char;

// Funções da libc.
extern "C" {
    fn abs(x: i32) -> i32;
    fn strlen(s: *const c_char) -> usize;
}

static mut MAIOR_DISTANCIA: i32 = 0;

/// Distância entre dois inteiros; guarda a maior já vista.
pub fn distancia(a: i32, b: i32) -> i32 {
    let d = abs(a - b);
    unsafe {
        if d > MAIOR_DISTANCIA {
            MAIOR_DISTANCIA = d;
        }
    }
    d
}

pub fn maior_distancia() -> i32 {
    unsafe { *&MAIOR_DISTANCIA }
}

/// Tamanho de uma string C.
///
/// # Safety
///
/// `s` precisa apontar para uma string terminada em '\0'.
pub unsafe fn tamanho_c(s: *const c_char) -> usize {
    strlen(s)
}

/// Exportada para C.
#[no_mangle]
pub extern "C" fn ex1_triplo(x: i32) -> i32 {
    x * 3
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn distancias() {
        assert_eq!(distancia(2, 9), 7);
        assert_eq!(distancia(9, 2), 7);
        assert!(maior_distancia() >= 7);
    }

    #[test]
    fn muitas_threads() {
        let hs: Vec<_> = (0..8)
            .map(|i| std::thread::spawn(move || distancia(0, i * 100)))
            .collect();
        for h in hs {
            h.join().unwrap();
        }
        assert!(maior_distancia() >= 700);
    }

    #[test]
    fn tamanho() {
        assert_eq!(unsafe { tamanho_c(c"cinco".as_ptr()) }, 5);
    }

    #[test]
    fn triplo() {
        assert_eq!(ex1_triplo(14), 42);
    }
}
```

<details>
<summary>Dica</summary>

São quatro problemas que o compilador mostra (bloco `extern`, atributo, chamada a `abs`, operação dentro de `unsafe fn`) e um que só aparece depois deles: a referência ao `static mut`. O `help` sugere `&raw const`, que compila, mas `distancia` continua escrevendo no `static mut` a partir de 8 threads ao mesmo tempo. Qual tipo do Módulo 11 guarda "o maior valor visto" sem `unsafe`? (Procure `fetch_max`.) E pense se `abs` merece ser `safe`: o que `abs(INT_MIN)` faz em C?

</details>

## 2. Dois empréstimos mutáveis (médio, passe nos testes)

Lição: [03](03-abstracoes-seguras.md).

`cargo new --lib fatias`. Implemente as duas funções com ponteiros crus, no estilo do `dividir_mut` da lição 03. Não vale usar `split_at_mut`, `split_first_mut`, `get_disjoint_mut` e afins: a ideia é escrever o `unsafe` e justificá-lo.

```rust
/// Referências mutáveis para o primeiro e o último elemento, ou `None` se o
/// slice tiver menos de dois elementos (seriam o mesmo elemento).
pub fn primeiro_e_ultimo_mut<T>(v: &mut [T]) -> Option<(&mut T, &mut T)> {
    todo!()
}

/// Divide `v` em `[..i]`, `[i..j]` e `[j..]`.
///
/// # Panics
///
/// Se `i > j` ou `j > v.len()`, com uma mensagem contendo "índices inválidos".
pub fn dividir_em_tres<T>(v: &mut [T], i: usize, j: usize) -> (&mut [T], &mut [T], &mut [T]) {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn troca_primeiro_e_ultimo() {
        let mut v = [1, 2, 3, 4];
        if let Some((a, b)) = primeiro_e_ultimo_mut(&mut v) {
            std::mem::swap(a, b);
        }
        assert_eq!(v, [4, 2, 3, 1]);
    }

    #[test]
    fn curto_demais() {
        assert!(primeiro_e_ultimo_mut::<i32>(&mut []).is_none());
        assert!(primeiro_e_ultimo_mut(&mut [1]).is_none());
        let mut dois = [String::from("a"), String::from("b")];
        let (a, b) = primeiro_e_ultimo_mut(&mut dois).unwrap();
        a.push('!');
        b.push('?');
        assert_eq!(dois, ["a!", "b?"]);
    }

    #[test]
    fn tres_partes() {
        let mut v = [0, 1, 2, 3, 4, 5];
        let (a, b, c) = dividir_em_tres(&mut v, 2, 5);
        assert_eq!(a, [0, 1]);
        assert_eq!(b, [2, 3, 4]);
        assert_eq!(c, [5]);
        a[0] = 10;
        b[0] = 20;
        c[0] = 50;
        assert_eq!(v, [10, 1, 20, 3, 4, 50]);
    }

    #[test]
    fn partes_vazias() {
        let mut v = [1, 2, 3];
        let (a, b, c) = dividir_em_tres(&mut v, 0, 0);
        assert_eq!((a.len(), b.len(), c.len()), (0, 0, 3));
        let (a, b, c) = dividir_em_tres(&mut v, 3, 3);
        assert_eq!((a.len(), b.len(), c.len()), (3, 0, 0));
    }

    #[test]
    #[should_panic(expected = "índices inválidos")]
    fn i_maior_que_j() {
        dividir_em_tres(&mut [1, 2, 3], 2, 1);
    }

    #[test]
    #[should_panic(expected = "índices inválidos")]
    fn j_alem_do_fim() {
        dividir_em_tres(&mut [1, 2, 3], 1, 4);
    }
}
```

<details>
<summary>Dica</summary>

Tente primeiro `Some((&mut v[0], &mut v[len - 1]))` e leia o erro (E0499, o mesmo da lição 03). Depois pegue um único `v.as_mut_ptr()` e derive tudo dele: `&mut *p` e `&mut *p.add(len - 1)` para a primeira função, `slice::from_raw_parts_mut` três vezes para a segunda. O comentário `SAFETY` precisa dizer por que os pedaços não se sobrepõem e por que estão dentro do slice.

</details>

## 3. Caça ao UB (médio, passe nos testes e no Miri)

Lição: [04](04-comportamento-indefinido-e-miri.md).

`cargo new --lib caca`. As três funções têm comentários `SAFETY` que parecem razoáveis, e as três têm UB. Rode `cargo test`, depois `cargo test --release`, e compare. Depois rode o Miri em cada teste (`cargo +nightly miri test nome_do_teste`, porque ele para no primeiro UB). Corrija as funções **sem mudar os testes**, até tudo passar no `cargo test` e no Miri. Se puder resolver sem `unsafe`, melhor.

```rust
/// Acrescenta ao fim de `v` uma cópia de cada elemento original, multiplicada por `fator`.
pub fn estender_multiplicado(v: &mut Vec<i32>, fator: i32) {
    let n = v.len();
    let inicio = v.as_ptr();
    for i in 0..n {
        // SAFETY: `i < n`, então o índice está dentro do vetor.
        let x = unsafe { *inicio.add(i) };
        v.push(x * fator);
    }
}

/// Lê um `u32` little-endian do começo de `bytes`, ou `None` se houver menos de 4 bytes.
pub fn ler_u32(bytes: &[u8]) -> Option<u32> {
    if bytes.len() < 4 {
        return None;
    }
    // SAFETY: há pelo menos 4 bytes.
    let x = unsafe { *(bytes.as_ptr() as *const u32) };
    Some(u32::from_le(x))
}

/// Soma 1 a `v[i]` e a `v[j]` (se `i == j`, o elemento ganha 2).
pub fn incrementar_dois(v: &mut [i32], i: usize, j: usize) {
    assert!(i < v.len() && j < v.len(), "índice fora do slice");
    let a = &mut v[i] as *mut i32;
    let b = &mut v[j] as *mut i32;
    // SAFETY: os dois índices foram verificados acima.
    unsafe {
        *a += 1;
        *b += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn estende() {
        let mut v = vec![1, 2, 3];
        estender_multiplicado(&mut v, 10);
        assert_eq!(v, [1, 2, 3, 10, 20, 30]);
    }

    #[test]
    fn estende_vazio() {
        let mut v = Vec::new();
        estender_multiplicado(&mut v, 10);
        assert!(v.is_empty());
    }

    #[test]
    fn le_no_meio_de_um_buffer() {
        let pacote = [0xff, 0x2a, 0x00, 0x00, 0x00, 0xff];
        assert_eq!(ler_u32(&pacote[1..]), Some(42));
        assert_eq!(ler_u32(&pacote[..3]), None);
    }

    #[test]
    fn incrementa() {
        let mut v = [0, 0, 0];
        incrementar_dois(&mut v, 0, 2);
        incrementar_dois(&mut v, 1, 1);
        assert_eq!(v, [1, 2, 1]);
    }
}
```

<details>
<summary>Dica</summary>

- `estender_multiplicado`: o que acontece com o buffer de um `Vec` quando ele cresce além da capacidade? É o mesmo bug do `push_back` em C++ da lição 04.
- `ler_u32`: um `*const u32` precisa estar alinhado a 4 bytes. `&pacote[1..]` está? Procure `u32::from_le_bytes` e `slice::first_chunk`.
- `incrementar_dois`: este passa em debug e em release, e só o Miri reclama. Leia o relatório: qual linha "invalidou" o ponteiro `a`? A regra prática da lição 02 resolve.

</details>

## 4. Uma biblioteca C (médio, passe nos testes)

Lição: [05](05-chamando-c.md).

`cargo new --lib texto-c`. Salve o C abaixo em `csrc/texto.c`. Escreva um `build.rs` que o compile com a crate `cc` (`cargo add --build cc`), as declarações num `mod ffi` privado e os três wrappers seguros.

```c
#include <stddef.h>

/* Conta as vogais ASCII (maiúsculas ou minúsculas) de uma string terminada em '\0'. */
size_t contar_vogais(const char *s) {
    size_t n = 0;
    for (; *s != '\0'; s++) {
        switch (*s) {
        case 'a': case 'e': case 'i': case 'o': case 'u':
        case 'A': case 'E': case 'I': case 'O': case 'U':
            n++;
        }
    }
    return n;
}

/* Inverte os `n` bytes de `buf`, no lugar. */
void inverter(char *buf, size_t n) {
    for (size_t i = 0; i < n / 2; i++) {
        char t = buf[i];
        buf[i] = buf[n - 1 - i];
        buf[n - 1 - i] = t;
    }
}

/* Grava em *saida o maior dos `n` valores. Devolve 0, ou -1 se n == 0. */
int maior(const int *v, size_t n, int *saida) {
    if (n == 0) return -1;
    int m = v[0];
    for (size_t i = 1; i < n; i++) if (v[i] > m) m = v[i];
    *saida = m;
    return 0;
}
```

```rust
use std::ffi::NulError;

/// Número de vogais ASCII. Falha se `texto` tiver um byte zero no meio.
pub fn contar_vogais(texto: &str) -> Result<usize, NulError> {
    todo!()
}

/// Inverte os bytes, no lugar.
pub fn inverter(bytes: &mut [u8]) {
    todo!()
}

/// O maior valor, ou `None` para um slice vazio.
pub fn maior(valores: &[i32]) -> Option<i32> {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vogais() {
        // "ã" não é ASCII: em UTF-8 são dois bytes, e nenhum deles é uma vogal.
        assert_eq!(contar_vogais("Programação em Rust").unwrap(), 6);
        assert_eq!(contar_vogais("").unwrap(), 0);
        assert!(contar_vogais("a\0e").is_err());
    }

    #[test]
    fn inverte() {
        let mut b = *b"abcde";
        inverter(&mut b);
        assert_eq!(&b, b"edcba");
        let mut vazio: [u8; 0] = [];
        inverter(&mut vazio);
    }

    #[test]
    fn maior_valor() {
        assert_eq!(maior(&[3, -1, 7, 2]), Some(7));
        assert_eq!(maior(&[-5]), Some(-5));
        assert_eq!(maior(&[]), None);
    }
}
```

O Miri não roda este exercício (ele não executa C).

<details>
<summary>Dica</summary>

Use `c_char` e `c_int` de `std::ffi` nas declarações. `CString::new` dá o `NulError` de graça (com `?`). Para `inverter`, `bytes.as_mut_ptr().cast::<c_char>()`. Para `maior`, crie um `c_int` local e passe `&raw mut` dele; depois, `(r == 0).then_some(saida)`.

</details>

## 5. Uma pilha para C (difícil, passe nos testes e no Miri)

Lição: [06](06-expondo-rust-para-c.md).

`cargo new --lib pilha-c`. Implemente as funções exportadas. Todas aceitam ponteiros nulos sem quebrar (a especificação de cada uma diz o que fazer). As que recebem ponteiros devem ser `unsafe extern "C" fn` com uma seção `# Safety`.

```rust
/// Grava em `*saida` a média dos `n` valores de `v`.
/// Devolve 0 em caso de sucesso, -1 se algum ponteiro for nulo e -2 se `n == 0`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ex_media(v: *const f64, n: usize, saida: *mut f64) -> i32 {
    todo!()
}

/// Pilha opaca: o C só vê ponteiros para ela.
pub struct Pilha {
    itens: Vec<i64>,
}

/// Cria uma pilha vazia. Libere com `ex_pilha_liberar`.
#[unsafe(no_mangle)]
pub extern "C" fn ex_pilha_nova() -> *mut Pilha {
    todo!()
}

/// Empilha `x`. Não faz nada se `p` for nulo.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ex_pilha_empilhar(p: *mut Pilha, x: i64) {
    todo!()
}

/// Desempilha para `*saida`. Devolve `false` se a pilha estiver vazia ou se
/// algum ponteiro for nulo (e então não mexe em `*saida`).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ex_pilha_desempilhar(p: *mut Pilha, saida: *mut i64) -> bool {
    todo!()
}

/// Quantos itens há na pilha (0 se `p` for nulo).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ex_pilha_tamanho(p: *const Pilha) -> usize {
    todo!()
}

/// Libera a pilha. Aceita nulo.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ex_pilha_liberar(p: *mut Pilha) {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ptr;

    #[test]
    fn media() {
        let v = [1.0, 2.0, 6.0];
        let mut m = 0.0;
        // SAFETY: ponteiros válidos para dados locais.
        unsafe {
            assert_eq!(ex_media(v.as_ptr(), v.len(), &raw mut m), 0);
            assert_eq!(m, 3.0);
            assert_eq!(ex_media(v.as_ptr(), 0, &raw mut m), -2);
            assert_eq!(ex_media(ptr::null(), 3, &raw mut m), -1);
            assert_eq!(ex_media(v.as_ptr(), 3, ptr::null_mut()), -1);
        }
    }

    #[test]
    fn pilha() {
        let p = ex_pilha_nova();
        let mut x = 0;
        // SAFETY: `p` vem de `ex_pilha_nova` e é liberado uma vez, no fim.
        unsafe {
            ex_pilha_empilhar(p, 10);
            ex_pilha_empilhar(p, 20);
            assert_eq!(ex_pilha_tamanho(p), 2);
            assert!(ex_pilha_desempilhar(p, &raw mut x));
            assert_eq!(x, 20);
            assert!(ex_pilha_desempilhar(p, &raw mut x));
            assert_eq!(x, 10);
            assert!(!ex_pilha_desempilhar(p, &raw mut x));
            assert!(!ex_pilha_desempilhar(p, ptr::null_mut()));
            ex_pilha_liberar(p);
        }
    }

    #[test]
    fn nulos_nao_quebram() {
        let mut x = 0;
        // SAFETY: nulo é aceito por todas as funções.
        unsafe {
            ex_pilha_empilhar(ptr::null_mut(), 1);
            assert!(!ex_pilha_desempilhar(ptr::null_mut(), &raw mut x));
            assert_eq!(ex_pilha_tamanho(ptr::null()), 0);
            ex_pilha_liberar(ptr::null_mut());
        }
    }
}
```

**Desafio opcional:** acrescente `crate-type = ["lib", "staticlib"]`, escreva um `pilha.h` e um `main.c` que empilhe 100, 200 e 300 e desempilhe tudo, e compile como na lição 06.

<details>
<summary>Dica</summary>

`Box::into_raw(Box::new(Pilha { itens: Vec::new() }))` na criação; `p.as_mut()` (ou `as_ref()`) devolve `Option<&mut Pilha>`, que trata o nulo de uma vez; `Box::from_raw` na liberação. Para a média, `std::slice::from_raw_parts(v, n)` dá um `&[f64]`, e daí é Rust comum. Grave em `saida` com `saida.write(valor)`.

</details>

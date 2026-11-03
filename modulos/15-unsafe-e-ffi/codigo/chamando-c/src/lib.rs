//! Chamando C a partir de Rust (lição 05 do Módulo 15).
//!
//! `ffi` tem as declarações cruas, que espelham `csrc/estatisticas.c`.
//! As funções públicas do crate embrulham essas chamadas numa API segura.

use std::ffi::{CStr, CString, NulError};

/// Mesmo layout que `Ponto` em C, graças ao `#[repr(C)]`.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Ponto {
    pub x: f64,
    pub y: f64,
}

mod ffi {
    use super::Ponto;
    use std::ffi::c_char;

    // O `unsafe` do bloco diz: "eu garanto que estas assinaturas batem com o
    // código C". O compilador não tem como conferir.
    unsafe extern "C" {
        // Recebe tudo por valor e não tem pré-condição: pode ser `safe`.
        pub safe fn distancia(a: Ponto, b: Ponto) -> f64;
        // As outras recebem ponteiros e ficam `unsafe` (o padrão).
        pub fn media(valores: *const f64, n: usize) -> f64;
        pub fn aplicar(v: *mut i32, n: usize, f: extern "C" fn(i32) -> i32);
        pub fn saudacao(nome: *const c_char) -> *mut c_char;
        pub fn liberar_texto(texto: *mut c_char);
    }
}

/// Média dos valores (0.0 para um slice vazio).
pub fn media(valores: &[f64]) -> f64 {
    // SAFETY: o ponteiro e o tamanho vêm do mesmo slice, válido durante a
    // chamada. A função C só lê `n` elementos e não guarda o ponteiro.
    unsafe { ffi::media(valores.as_ptr(), valores.len()) }
}

/// Distância entre dois pontos. A declaração é `safe`, então não há `unsafe` aqui.
pub fn distancia(a: Ponto, b: Ponto) -> f64 {
    ffi::distancia(a, b)
}

/// Aplica `f` a cada elemento, no lugar, usando o laço escrito em C.
///
/// `f` precisa ser uma função `extern "C"` (closures não servem: veja a lição).
pub fn aplicar(v: &mut [i32], f: extern "C" fn(i32) -> i32) {
    // SAFETY: ponteiro e tamanho vêm do mesmo `&mut [i32]`, exclusivo durante
    // a chamada. `f` é uma função `extern "C"` válida e não dá panic
    // (um panic atravessando C aborta o processo).
    unsafe { ffi::aplicar(v.as_mut_ptr(), v.len(), f) }
}

/// Saudação montada pelo C. Falha se `nome` tiver um byte zero no meio.
pub fn saudacao(nome: &str) -> Result<String, NulError> {
    let nome_c = CString::new(nome)?; // acrescenta o '\0' final
    // SAFETY: `nome_c` é uma string C válida, terminada em '\0', viva até o
    // fim desta função.
    let cru = unsafe { ffi::saudacao(nome_c.as_ptr()) };
    assert!(!cru.is_null(), "saudacao: sem memória");
    // SAFETY: `cru` não é nulo e aponta para uma string terminada em '\0'
    // criada pelo C. Copiamos o conteúdo para uma `String` antes de liberar.
    let texto = unsafe { CStr::from_ptr(cru) }
        .to_string_lossy()
        .into_owned();
    // SAFETY: `cru` veio de `saudacao` e é liberado uma vez, pela função C
    // correspondente (o `free` do C, e não o alocador do Rust).
    unsafe { ffi::liberar_texto(cru) };
    Ok(texto)
}

#[cfg(test)]
mod tests {
    use super::*;

    extern "C" fn quadrado(x: i32) -> i32 {
        x * x
    }

    #[test]
    fn media_basica() {
        assert_eq!(media(&[1.0, 2.0, 3.0, 4.0]), 2.5);
        assert_eq!(media(&[]), 0.0);
    }

    #[test]
    fn distancia_3_4_5() {
        let a = Ponto { x: 0.0, y: 0.0 };
        let b = Ponto { x: 3.0, y: 4.0 };
        assert_eq!(distancia(a, b), 5.0);
    }

    #[test]
    fn aplicar_callback() {
        let mut v = [1, 2, 3];
        aplicar(&mut v, quadrado);
        assert_eq!(v, [1, 4, 9]);
    }

    #[test]
    fn saudacao_ida_e_volta() {
        assert_eq!(saudacao("Ana").unwrap(), "Olá, Ana!");
        assert!(saudacao("a\0b").is_err());
    }
}

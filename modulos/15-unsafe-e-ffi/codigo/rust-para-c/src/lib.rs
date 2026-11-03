//! Uma biblioteca Rust com interface C (lição 06 do Módulo 15).
//!
//! O cabeçalho correspondente está em `include/rust_para_c.h`. Todas as
//! funções exportadas começam com `rc_`, porque C não tem namespaces.

use std::ffi::{CStr, c_char};
use std::panic;

/// Soma simples: tipos primitivos atravessam a fronteira sem conversão.
#[unsafe(no_mangle)]
pub extern "C" fn rc_somar(a: i32, b: i32) -> i32 {
    a.wrapping_add(b)
}

/// Conta as palavras de uma string C em UTF-8.
/// Devolve -1 se `texto` for nulo ou não for UTF-8 válido.
///
/// # Safety
///
/// `texto` precisa ser nulo ou apontar para uma string terminada em '\0',
/// válida durante a chamada.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rc_contar_palavras(texto: *const c_char) -> i64 {
    if texto.is_null() {
        return -1;
    }
    // SAFETY: não é nulo, e o contrato da função garante uma string C válida.
    let texto = unsafe { CStr::from_ptr(texto) };
    match texto.to_str() {
        Ok(s) => s.split_whitespace().count() as i64,
        Err(_) => -1,
    }
}

/// Um tipo Rust que o C só enxerga como ponteiro opaco.
pub struct Contador {
    valor: u64,
    nome: String,
}

/// Cria um contador. Libere com `rc_contador_liberar`.
#[unsafe(no_mangle)]
pub extern "C" fn rc_contador_novo() -> *mut Contador {
    let c = Contador {
        valor: 0,
        nome: String::from("contador"),
    };
    // A posse passa para o C, que devolve o ponteiro em `rc_contador_liberar`.
    Box::into_raw(Box::new(c))
}

/// Soma `n` ao contador e devolve o novo valor (0 se `c` for nulo).
///
/// # Safety
///
/// `c` precisa ser nulo ou um ponteiro devolvido por `rc_contador_novo` que
/// ainda não foi liberado, sem outro acesso simultâneo.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rc_contador_somar(c: *mut Contador, n: u64) -> u64 {
    // SAFETY: pelo contrato, `c` é nulo (`as_mut` devolve `None`) ou válido
    // e exclusivo durante a chamada.
    match unsafe { c.as_mut() } {
        Some(c) => {
            c.valor += n;
            c.valor
        }
        None => 0,
    }
}

/// Libera um contador. Aceita nulo (não faz nada), como o `free` do C.
///
/// # Safety
///
/// `c` precisa ser nulo ou um ponteiro de `rc_contador_novo` ainda não
/// liberado. Depois desta chamada, o ponteiro não pode mais ser usado.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rc_contador_liberar(c: *mut Contador) {
    if c.is_null() {
        return;
    }
    // SAFETY: pelo contrato, `c` veio de `Box::into_raw` e é liberado uma vez.
    let c = unsafe { Box::from_raw(c) };
    println!("[rust] liberando {} com valor {}", c.nome, c.valor);
}

/// Divide `a` por `b` e grava em `*saida`. Devolve 0 em caso de sucesso,
/// -1 se `saida` for nulo e -2 se o código Rust der panic (divisão por zero).
///
/// # Safety
///
/// `saida` precisa ser nulo ou apontar para um `i32` gravável.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rc_dividir(a: i32, b: i32, saida: *mut i32) -> i32 {
    if saida.is_null() {
        return -1;
    }
    // Um panic não pode atravessar uma função `extern "C"`: se escapar daqui,
    // o processo é abortado. `catch_unwind` o transforma num código de erro.
    match panic::catch_unwind(|| a / b) {
        Ok(q) => {
            // SAFETY: `saida` não é nulo e, pelo contrato, é gravável.
            unsafe { saida.write(q) };
            0
        }
        Err(_) => -2,
    }
}

/// Exemplo do que NÃO fazer: um panic dentro de `extern "C"` aborta o processo.
#[unsafe(no_mangle)]
pub extern "C" fn rc_panico() {
    panic!("panic dentro de extern \"C\"");
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ptr;

    #[test]
    fn somar() {
        assert_eq!(rc_somar(2, 3), 5);
    }

    #[test]
    fn contar_palavras() {
        // SAFETY: literais `c"..."` são strings C válidas; nulo é aceito.
        unsafe {
            assert_eq!(rc_contar_palavras(c"um dois  três".as_ptr()), 3);
            assert_eq!(rc_contar_palavras(c"".as_ptr()), 0);
            assert_eq!(rc_contar_palavras(ptr::null()), -1);
            assert_eq!(rc_contar_palavras(c"\xff".as_ptr()), -1);
        }
    }

    #[test]
    fn contador_ciclo_de_vida() {
        let c = rc_contador_novo();
        // SAFETY: `c` vem de `rc_contador_novo` e é liberado uma vez, no fim.
        unsafe {
            assert_eq!(rc_contador_somar(c, 2), 2);
            assert_eq!(rc_contador_somar(c, 3), 5);
            assert_eq!(rc_contador_somar(ptr::null_mut(), 3), 0);
            rc_contador_liberar(c);
            rc_contador_liberar(ptr::null_mut());
        }
    }

    #[test]
    fn dividir_com_e_sem_panic() {
        let mut q = 0;
        // SAFETY: `&raw mut q` aponta para um `i32` local gravável.
        unsafe {
            assert_eq!(rc_dividir(7, 2, &raw mut q), 0);
            assert_eq!(q, 3);
            assert_eq!(rc_dividir(7, 0, &raw mut q), -2);
            assert_eq!(rc_dividir(7, 1, ptr::null_mut()), -1);
        }
    }
}

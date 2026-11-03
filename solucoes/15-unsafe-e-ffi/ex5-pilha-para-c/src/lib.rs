//! Uma pilha de `i64` exportada para C.

/// Grava em `*saida` a média dos `n` valores de `v`.
/// Devolve 0 em caso de sucesso, -1 se algum ponteiro for nulo e -2 se `n == 0`.
///
/// # Safety
///
/// Se não forem nulos, `v` precisa apontar para `n` valores `f64` válidos e
/// `saida` para um `f64` gravável.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ex_media(v: *const f64, n: usize, saida: *mut f64) -> i32 {
    if v.is_null() || saida.is_null() {
        return -1;
    }
    if n == 0 {
        return -2;
    }
    // SAFETY: `v` não é nulo e, pelo contrato, aponta para `n` valores.
    let valores = unsafe { std::slice::from_raw_parts(v, n) };
    let media = valores.iter().sum::<f64>() / n as f64;
    // SAFETY: `saida` não é nulo e, pelo contrato, é gravável.
    unsafe { saida.write(media) };
    0
}

/// Pilha opaca: o C só vê ponteiros para ela.
pub struct Pilha {
    itens: Vec<i64>,
}

/// Cria uma pilha vazia. Libere com `ex_pilha_liberar`.
#[unsafe(no_mangle)]
pub extern "C" fn ex_pilha_nova() -> *mut Pilha {
    Box::into_raw(Box::new(Pilha { itens: Vec::new() }))
}

/// Empilha `x`. Não faz nada se `p` for nulo.
///
/// # Safety
///
/// `p` precisa ser nulo ou vir de `ex_pilha_nova`, sem ter sido liberado.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ex_pilha_empilhar(p: *mut Pilha, x: i64) {
    // SAFETY: pelo contrato, `p` é nulo (`None`) ou uma pilha viva.
    if let Some(p) = unsafe { p.as_mut() } {
        p.itens.push(x);
    }
}

/// Desempilha para `*saida`. Devolve `false` se a pilha estiver vazia ou se
/// algum ponteiro for nulo (e então não mexe em `*saida`).
///
/// # Safety
///
/// `p` como em `ex_pilha_empilhar`; `saida` nulo ou gravável.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ex_pilha_desempilhar(p: *mut Pilha, saida: *mut i64) -> bool {
    if saida.is_null() {
        return false;
    }
    // SAFETY: pelo contrato, `p` é nulo (`None`) ou uma pilha viva.
    let Some(p) = (unsafe { p.as_mut() }) else {
        return false;
    };
    match p.itens.pop() {
        Some(x) => {
            // SAFETY: `saida` não é nulo e, pelo contrato, é gravável.
            unsafe { saida.write(x) };
            true
        }
        None => false,
    }
}

/// Quantos itens há na pilha (0 se `p` for nulo).
///
/// # Safety
///
/// `p` como em `ex_pilha_empilhar`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ex_pilha_tamanho(p: *const Pilha) -> usize {
    // SAFETY: pelo contrato, `p` é nulo (`None`) ou uma pilha viva.
    unsafe { p.as_ref() }.map_or(0, |p| p.itens.len())
}

/// Libera a pilha. Aceita nulo.
///
/// # Safety
///
/// `p` precisa ser nulo ou vir de `ex_pilha_nova`, e não pode ser usado depois.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ex_pilha_liberar(p: *mut Pilha) {
    if !p.is_null() {
        // SAFETY: `p` veio de `Box::into_raw` e é liberado uma única vez.
        drop(unsafe { Box::from_raw(p) });
    }
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

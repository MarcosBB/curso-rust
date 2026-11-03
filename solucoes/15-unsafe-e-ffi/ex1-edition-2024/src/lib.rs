use std::ffi::c_char;
use std::sync::atomic::{AtomicI32, Ordering};

// Funções da libc. O `unsafe` do bloco é a promessa de que as assinaturas
// batem com as da libc.
unsafe extern "C" {
    // `abs` parece inofensiva, mas em C `abs(INT_MIN)` é UB (o resultado não
    // cabe num `int`). Por isso ela fica `unsafe`, e quem chama exclui esse caso.
    fn abs(x: i32) -> i32;
    // `strlen` lê memória através do ponteiro: continua `unsafe`.
    fn strlen(s: *const c_char) -> usize;
}

// Um atômico em vez de `static mut`: várias threads atualizam o valor e o
// compilador garante que não há data race. Nenhum `unsafe` aqui.
static MAIOR_DISTANCIA: AtomicI32 = AtomicI32::new(0);

/// Distância entre dois inteiros; guarda a maior já vista.
pub fn distancia(a: i32, b: i32) -> i32 {
    let diferenca = a.wrapping_sub(b);
    assert!(diferenca != i32::MIN, "distância não cabe em i32");
    // SAFETY: o único valor proibido para `abs` é `i32::MIN`, excluído acima.
    let d = unsafe { abs(diferenca) };
    MAIOR_DISTANCIA.fetch_max(d, Ordering::Relaxed);
    d
}

pub fn maior_distancia() -> i32 {
    MAIOR_DISTANCIA.load(Ordering::Relaxed)
}

/// Tamanho de uma string C.
///
/// # Safety
///
/// `s` precisa apontar para uma string terminada em '\0'.
pub unsafe fn tamanho_c(s: *const c_char) -> usize {
    // SAFETY: o contrato desta função exige o mesmo que `strlen`.
    unsafe { strlen(s) }
}

/// Exportada para C.
#[unsafe(no_mangle)]
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
        // SAFETY: um literal `c"..."` é uma string C válida.
        assert_eq!(unsafe { tamanho_c(c"cinco".as_ptr()) }, 5);
    }

    #[test]
    fn triplo() {
        assert_eq!(ex1_triplo(14), 42);
    }
}

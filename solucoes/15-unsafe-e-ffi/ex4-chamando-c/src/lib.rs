use std::ffi::{CString, NulError, c_char, c_int};

mod ffi {
    use super::{c_char, c_int};

    unsafe extern "C" {
        pub fn contar_vogais(s: *const c_char) -> usize;
        pub fn inverter(buf: *mut c_char, n: usize);
        pub fn maior(v: *const c_int, n: usize, saida: *mut c_int) -> c_int;
    }
}

/// Número de vogais ASCII. Falha se `texto` tiver um byte zero no meio.
pub fn contar_vogais(texto: &str) -> Result<usize, NulError> {
    let c = CString::new(texto)?;
    // SAFETY: `c` é uma string C válida, terminada em '\0', viva durante a chamada.
    Ok(unsafe { ffi::contar_vogais(c.as_ptr()) })
}

/// Inverte os bytes, no lugar.
pub fn inverter(bytes: &mut [u8]) {
    // SAFETY: ponteiro e tamanho vêm do mesmo `&mut [u8]`, exclusivo durante
    // a chamada; `u8` e `c_char` têm o mesmo tamanho e alinhamento.
    unsafe { ffi::inverter(bytes.as_mut_ptr().cast::<c_char>(), bytes.len()) }
}

/// O maior valor, ou `None` para um slice vazio.
pub fn maior(valores: &[i32]) -> Option<i32> {
    let mut saida: c_int = 0;
    // SAFETY: ponteiro e tamanho vêm do mesmo slice (`i32` é o `int` do C nas
    // plataformas suportadas); `saida` é um `c_int` local gravável.
    let r = unsafe { ffi::maior(valores.as_ptr(), valores.len(), &raw mut saida) };
    (r == 0).then_some(saida)
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

//! Uma versão própria de `split_at_mut`.

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn divide_e_altera_as_duas_partes() {
        let mut v = [1, 2, 3, 4, 5];
        let (a, b) = dividir_mut(&mut v, 2);
        a[0] = 10;
        b[2] = 50;
        assert_eq!(a, [10, 2]);
        assert_eq!(b, [3, 4, 50]);
        assert_eq!(v, [10, 2, 3, 4, 50]);
    }

    #[test]
    fn extremos() {
        let mut v = [1, 2, 3];
        let (a, b) = dividir_mut(&mut v, 0);
        assert!(a.is_empty());
        assert_eq!(b.len(), 3);
        let (a, b) = dividir_mut(&mut v, 3);
        assert_eq!(a.len(), 3);
        assert!(b.is_empty());
    }

    #[test]
    #[should_panic(expected = "maior que o tamanho")]
    fn meio_invalido() {
        let mut v = [1, 2, 3];
        dividir_mut(&mut v, 4);
    }
}

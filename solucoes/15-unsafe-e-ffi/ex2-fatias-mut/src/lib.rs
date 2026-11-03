use std::slice;

/// Referências mutáveis para o primeiro e o último elemento, ou `None` se o
/// slice tiver menos de dois elementos (seriam o mesmo elemento).
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

/// Divide `v` em `[..i]`, `[i..j]` e `[j..]`.
///
/// # Panics
///
/// Se `i > j` ou `j > v.len()`.
pub fn dividir_em_tres<T>(v: &mut [T], i: usize, j: usize) -> (&mut [T], &mut [T], &mut [T]) {
    let len = v.len();
    assert!(
        i <= j && j <= len,
        "índices inválidos: {i}, {j} (tamanho {len})"
    );
    let p = v.as_mut_ptr();
    // SAFETY: `0 <= i <= j <= len`, então as faixas `[0, i)`, `[i, j)` e
    // `[j, len)` estão dentro do slice e não se sobrepõem. O ponteiro vem de
    // `v`, emprestado de forma exclusiva pelo lifetime do resultado.
    unsafe {
        (
            slice::from_raw_parts_mut(p, i),
            slice::from_raw_parts_mut(p.add(i), j - i),
            slice::from_raw_parts_mut(p.add(j), len - j),
        )
    }
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

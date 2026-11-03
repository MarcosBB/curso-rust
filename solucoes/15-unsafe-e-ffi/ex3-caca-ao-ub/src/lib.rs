/// Acrescenta ao fim de `v` uma cópia de cada elemento original, multiplicada por `fator`.
pub fn estender_multiplicado(v: &mut Vec<i32>, fator: i32) {
    // O bug original: `v.as_ptr()` guardado antes dos `push`. Quando o vetor
    // cresce, ele realoca, e o ponteiro passa a apontar para memória liberada.
    // Indexar a cada volta não precisa de `unsafe` e lê sempre o buffer atual.
    let n = v.len();
    v.reserve(n);
    for i in 0..n {
        let x = v[i];
        v.push(x * fator);
    }
}

/// Lê um `u32` little-endian do começo de `bytes`, ou `None` se houver menos de 4 bytes.
pub fn ler_u32(bytes: &[u8]) -> Option<u32> {
    // O bug original: `*(ptr as *const u32)` exige endereço múltiplo de 4, e
    // `&pacote[1..]` não é. `first_chunk` devolve `Option<&[u8; 4]>` e
    // `from_le_bytes` monta o número sem nenhuma exigência de alinhamento.
    bytes.first_chunk::<4>().map(|b| u32::from_le_bytes(*b))
}

/// Soma 1 a `v[i]` e a `v[j]` (se `i == j`, o elemento ganha 2).
pub fn incrementar_dois(v: &mut [i32], i: usize, j: usize) {
    assert!(i < v.len() && j < v.len(), "índice fora do slice");
    // O bug original: `&mut v[j]` cria um novo empréstimo exclusivo do slice
    // inteiro, o que invalida o ponteiro `a`, criado antes a partir de `&mut v[i]`.
    // A correção: derivar os dois ponteiros de uma mesma origem.
    let p = v.as_mut_ptr();
    // SAFETY: `i` e `j` são menores que `v.len()` (verificado acima), então
    // `p.add(i)` e `p.add(j)` ficam dentro do slice. Os dois vêm de `p` e
    // nenhuma referência ao slice é criada entre os acessos; se `i == j`,
    // os dois acessos são sequenciais, pelo mesmo ponteiro de origem.
    unsafe {
        *p.add(i) += 1;
        *p.add(j) += 1;
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

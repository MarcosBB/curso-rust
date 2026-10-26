//! Solução do exercício 3 do Módulo 09: nenhum `allow`, um `expect` com motivo.

use std::num::ParseIntError;

/// Lê valores separados por vírgula.
///
/// # Errors
///
/// Devolve o [`ParseIntError`] do primeiro valor que não for um número.
pub fn ler_valores(texto: &str) -> Result<Vec<u32>, ParseIntError> {
    texto.split(',').map(|v| v.trim().parse()).collect()
}

pub fn soma(valores: &[u32]) -> u32 {
    valores.iter().sum()
}

/// A média inteira, ou 0 para a lista vazia.
pub fn media(valores: &[u32]) -> u32 {
    match u32::try_from(valores.len()) {
        Ok(n) if n > 0 => soma(valores) / n,
        _ => 0,
    }
}

pub fn dobra(valores: &mut [u32]) {
    for v in valores.iter_mut() {
        *v *= 2;
    }
}

pub fn ponto(x: i32, y: i32, z: i32) -> (i32, i32, i32) {
    (x, y, z)
}

#[expect(
    dead_code,
    reason = "o formato antigo ainda é lido pelo script de migração, que fica fora desta crate"
)]
fn formato_antigo(valores: &[u32]) -> String {
    format!("{valores:?}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn le_e_calcula() {
        let v = ler_valores("2, 4,9").unwrap();
        assert_eq!(soma(&v), 15);
        assert_eq!(media(&v), 5);
        assert_eq!(media(&[]), 0);
    }

    #[test]
    fn dobra_no_lugar() {
        let mut v = [1, 2, 3];
        dobra(&mut v);
        assert_eq!(v, [2, 4, 6]);
        assert_eq!(ponto(1, 2, 3), (1, 2, 3));
    }
}

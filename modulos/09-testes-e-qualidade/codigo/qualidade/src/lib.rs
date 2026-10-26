//! Exemplo das lições 05 e 06 do Módulo 09: lints configurados no `Cargo.toml`
//! e no `clippy.toml`, e atributos `#[expect]` com motivo.

use std::num::ParseIntError;

/// Lê notas separadas por vírgula, como `"7, 8,10"`.
///
/// # Errors
///
/// Devolve o [`ParseIntError`] da primeira nota que não for um número.
pub fn ler_notas(texto: &str) -> Result<Vec<u32>, ParseIntError> {
    texto.split(',').map(|n| n.trim().parse()).collect()
}

/// A média inteira (arredondada para baixo), ou `None` se não houver notas.
pub fn media(notas: &[u32]) -> Option<u32> {
    if notas.is_empty() {
        return None;
    }
    let soma: u64 = notas.iter().map(|&n| u64::from(n)).sum();
    #[expect(
        clippy::cast_possible_truncation,
        reason = "a média de valores u32 sempre cabe em u32"
    )]
    Some((soma / notas.len() as u64) as u32)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn le_e_calcula() {
        // `unwrap` em teste: liberado pelo clippy.toml.
        let notas = ler_notas("7, 8,10").unwrap();
        assert_eq!(media(&notas), Some(8));
    }

    #[test]
    fn nota_invalida() {
        assert!(ler_notas("7,x").is_err());
    }

    #[test]
    fn sem_notas() {
        assert_eq!(media(&[]), None);
    }
}

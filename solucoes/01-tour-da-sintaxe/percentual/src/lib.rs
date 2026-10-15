/// Quanto `parte` representa de `total`, em porcentagem inteira (arredondada para baixo).
///
/// Devolve `None` se `total` for zero, se `parte * 100` estourar `u32`
/// ou se o resultado não couber em `u8`.
pub fn percentual(parte: u32, total: u32) -> Option<u8> {
    if total == 0 {
        return None;
    }
    match parte.checked_mul(100) {
        Some(vezes_cem) => u8::try_from(vezes_cem / total).ok(),
        None => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn metade() {
        assert_eq!(percentual(1, 2), Some(50));
    }

    #[test]
    fn arredonda_para_baixo() {
        assert_eq!(percentual(1, 3), Some(33));
    }

    #[test]
    fn total_zero() {
        assert_eq!(percentual(3, 0), None);
    }

    #[test]
    fn duzentos_por_cento_cabe() {
        assert_eq!(percentual(2, 1), Some(200));
    }

    #[test]
    fn trezentos_por_cento_nao_cabe() {
        assert_eq!(percentual(3, 1), None);
    }

    #[test]
    fn multiplicacao_estoura() {
        assert_eq!(percentual(u32::MAX, u32::MAX), None);
    }
}

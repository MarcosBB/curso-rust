//! Tarifas cobradas pelo banco (interno à crate).

const TARIFA_FIXA: i64 = 2;

/// Tarifa de um saque: fixa mais 1% do valor.
pub(crate) fn tarifa_saque(valor: i64) -> i64 {
    TARIFA_FIXA + valor / 100
}

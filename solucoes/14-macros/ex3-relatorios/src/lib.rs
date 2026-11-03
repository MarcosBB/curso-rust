//! Exercício 3: uma macro exportada que usa uma função auxiliar da crate.

pub mod formato {
    pub fn moeda(valor: f64) -> String {
        format!("R$ {valor:.2}")
    }
}

/// `linha!("rótulo", valor)` monta `"rótulo: R$ valor"`.
///
/// `#[macro_export]` publica a macro na raiz da crate (`relatorios::linha!`)
/// e `$crate` faz o caminho de `moeda` funcionar em qualquer crate.
#[macro_export]
macro_rules! linha {
    ($rotulo:expr, $valor:expr) => {
        format!("{}: {}", $rotulo, $crate::formato::moeda($valor))
    };
}

pub fn resumo(total: f64) -> String {
    linha!("total", total)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resumo_formata() {
        assert_eq!(resumo(99.9), "total: R$ 99.90");
    }
}

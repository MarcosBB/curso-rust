//! Exercício 2: `maximo!` recursiva.

/// O maior de um ou mais valores que implementam `Ord`.
#[macro_export]
macro_rules! maximo {
    // Caso base: um valor só (com vírgula final opcional).
    ($x:expr $(,)?) => {
        $x
    };
    // Passo: o primeiro contra o máximo do resto.
    ($x:expr, $($resto:expr),+ $(,)?) => {
        ::std::cmp::max($x, $crate::maximo!($($resto),+))
    };
}

#[cfg(test)]
mod tests {
    #[test]
    fn um_valor() {
        assert_eq!(maximo!(7), 7);
    }

    #[test]
    fn varios_valores() {
        assert_eq!(maximo!(1, 5, 2), 5);
        assert_eq!(maximo!(3, 9, 4, 9, 1,), 9);
        assert_eq!(maximo!("pera", "uva", "banana"), "uva");
    }

    #[test]
    fn avalia_cada_argumento_uma_vez() {
        let mut chamadas = 0;
        let mut proximo = || {
            chamadas += 1;
            chamadas
        };
        assert_eq!(maximo!(proximo(), proximo(), proximo()), 3);
        assert_eq!(chamadas, 3);
    }
}

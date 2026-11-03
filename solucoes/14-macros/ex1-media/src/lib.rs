//! Exercício 1: `media!` com repetição, vírgula final opcional e erro para `media!()`.

/// Média aritmética de um ou mais números, como `f64`.
#[macro_export]
macro_rules! media {
    () => {
        compile_error!("media! precisa de pelo menos um valor")
    };
    ($($x:expr),+ $(,)?) => {{
        // O array avalia cada expressão uma vez só, na ordem.
        let valores = [$($x as f64),+];
        valores.iter().sum::<f64>() / valores.len() as f64
    }};
}

#[cfg(test)]
mod tests {
    #[test]
    fn um_valor() {
        assert_eq!(media!(4.0), 4.0);
    }

    #[test]
    fn varios_valores() {
        assert_eq!(media!(1.0, 2.0, 3.0), 2.0);
        assert_eq!(media!(1, 2), 1.5);
    }

    #[test]
    fn virgula_final() {
        assert_eq!(media!(10.0, 20.0,), 15.0);
    }

    #[test]
    fn expressoes_e_avaliacao_unica() {
        let mut chamadas = 0;
        let mut proximo = || {
            chamadas += 1;
            chamadas as f64
        };
        // 1.0 e 2.0: cada chamada roda uma vez.
        assert_eq!(media!(proximo(), proximo()), 1.5);
        assert_eq!(chamadas, 2);
        assert_eq!(media!(2.0 * 3.0, 1.0 + 1.0), 4.0);
    }
}

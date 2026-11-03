//! Extension traits e uma trait selada.
//!
//! Outro crate não consegue criar uma moeda nova:
//!
//! ```compile_fail,E0277
//! use ex3_extensoes::moeda::Moeda;
//! struct Iene;
//! impl Moeda for Iene {
//!     const CODIGO: &'static str = "JPY";
//!     const CASAS: u32 = 0;
//! }
//! ```

pub mod ext {
    pub trait SliceExt<T> {
        /// O segundo maior valor **distinto**.
        fn segundo_maior(&self) -> Option<&T>
        where
            T: Ord;

        /// Quantas vezes `x` aparece.
        fn contagem_de(&self, x: &T) -> usize
        where
            T: PartialEq;
    }

    impl<T> SliceExt<T> for [T] {
        fn segundo_maior(&self) -> Option<&T>
        where
            T: Ord,
        {
            let maior = self.iter().max()?;
            self.iter().filter(|x| *x < maior).max()
        }

        fn contagem_de(&self, x: &T) -> usize
        where
            T: PartialEq,
        {
            self.iter().filter(|y| *y == x).count()
        }
    }

    pub trait OptionExt<T> {
        fn ou_erro(self, msg: &str) -> Result<T, String>;
    }

    impl<T> OptionExt<T> for Option<T> {
        fn ou_erro(self, msg: &str) -> Result<T, String> {
            self.ok_or_else(|| msg.to_string())
        }
    }
}

pub mod moeda {
    mod selo {
        pub trait Selado {}
    }

    pub trait Moeda: selo::Selado {
        const CODIGO: &'static str;
        /// Casas decimais da unidade menor (centavos: 2).
        const CASAS: u32;
    }

    pub struct Real;
    pub struct Dolar;
    pub struct Iene;

    impl selo::Selado for Real {}
    impl selo::Selado for Dolar {}
    impl selo::Selado for Iene {}

    impl Moeda for Real {
        const CODIGO: &'static str = "BRL";
        const CASAS: u32 = 2;
    }
    impl Moeda for Dolar {
        const CODIGO: &'static str = "USD";
        const CASAS: u32 = 2;
    }
    impl Moeda for Iene {
        const CODIGO: &'static str = "JPY";
        const CASAS: u32 = 0;
    }

    /// `formatar::<Real>(1234)` -> "12.34 BRL"; `formatar::<Iene>(500)` -> "500 JPY".
    pub fn formatar<M: Moeda>(unidades_menores: i64) -> String {
        if M::CASAS == 0 {
            return format!("{unidades_menores} {}", M::CODIGO);
        }
        let base = 10_i64.pow(M::CASAS);
        let sinal = if unidades_menores < 0 { "-" } else { "" };
        let abs = unidades_menores.abs();
        format!(
            "{sinal}{}.{:0largura$} {}",
            abs / base,
            abs % base,
            M::CODIGO,
            largura = M::CASAS as usize
        )
    }
}

#[cfg(test)]
mod tests {
    use super::ext::{OptionExt, SliceExt};
    use super::moeda::{Dolar, Iene, Real, formatar};

    #[test]
    fn segundo_maior() {
        assert_eq!([3, 9, 4, 9].segundo_maior(), Some(&4));
        assert_eq!(["b", "a"].segundo_maior(), Some(&"a"));
        assert_eq!([7, 7].segundo_maior(), None);
        assert_eq!(Vec::<u8>::new().segundo_maior(), None);
    }

    #[test]
    fn contagem() {
        assert_eq!([1, 2, 1, 1].contagem_de(&1), 3);
        let nomes = [String::from("ana"), String::from("bia")];
        assert_eq!(nomes.contagem_de(&String::from("bia")), 1);
        // f64 é PartialEq: funciona
        assert_eq!([0.5, 0.5].contagem_de(&0.5), 2);
    }

    #[test]
    fn option_ou_erro() {
        assert_eq!(Some(3).ou_erro("faltou"), Ok(3));
        assert_eq!(None::<i32>.ou_erro("faltou"), Err(String::from("faltou")));
    }

    #[test]
    fn moedas() {
        assert_eq!(formatar::<Real>(1234), "12.34 BRL");
        assert_eq!(formatar::<Dolar>(-5), "-0.05 USD");
        assert_eq!(formatar::<Iene>(500), "500 JPY");
    }
}

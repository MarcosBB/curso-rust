//! Lição 05: blanket impls, extension traits e sealed traits.

pub mod texto {
    /// Extension trait: métodos novos para um tipo que não é nosso (`str`).
    pub trait TextoExt {
        /// Ignora maiúsculas, espaços e pontuação.
        fn eh_palindromo(&self) -> bool;
        /// "o rato roeu" -> "O Rato Roeu".
        fn capitalizar_palavras(&self) -> String;
    }

    impl TextoExt for str {
        fn eh_palindromo(&self) -> bool {
            let letras: Vec<char> = self
                .chars()
                .filter(|c| c.is_alphanumeric())
                .flat_map(char::to_lowercase)
                .collect();
            letras.iter().eq(letras.iter().rev())
        }

        fn capitalizar_palavras(&self) -> String {
            self.split_whitespace()
                .map(|p| {
                    let mut cs = p.chars();
                    match cs.next() {
                        Some(primeira) => primeira.to_uppercase().chain(cs).collect(),
                        None => String::new(),
                    }
                })
                .collect::<Vec<String>>()
                .join(" ")
        }
    }
}

pub mod iter {
    use std::collections::HashMap;
    use std::hash::Hash;

    /// Extension trait com blanket impl: vale para **todo** iterador.
    pub trait IteratorExt: Iterator {
        /// Quantas vezes cada item aparece.
        fn contagem(self) -> HashMap<Self::Item, usize>
        where
            Self: Sized,
            Self::Item: Hash + Eq,
        {
            let mut mapa = HashMap::new();
            for item in self {
                *mapa.entry(item).or_insert(0) += 1;
            }
            mapa
        }

        /// O item mais frequente (empate: qualquer um deles).
        fn moda(self) -> Option<Self::Item>
        where
            Self: Sized,
            Self::Item: Hash + Eq,
        {
            self.contagem()
                .into_iter()
                .max_by_key(|&(_, n)| n)
                .map(|(item, _)| item)
        }
    }

    impl<I: Iterator> IteratorExt for I {}
}

pub mod formato {
    /// O módulo `selo` é privado: ninguém fora deste crate consegue nomear `Selado`.
    mod selo {
        pub trait Selado {}
    }

    /// Sealed trait: quem está fora do crate pode **usar**, mas não **implementar**.
    pub trait Formato: selo::Selado {
        fn extensao(&self) -> &'static str;
        fn serializar(&self, campos: &[(&str, &str)]) -> String;
    }

    pub struct Json;
    pub struct Csv;

    impl selo::Selado for Json {}
    impl selo::Selado for Csv {}

    impl Formato for Json {
        fn extensao(&self) -> &'static str {
            "json"
        }

        fn serializar(&self, campos: &[(&str, &str)]) -> String {
            let pares: Vec<String> = campos
                .iter()
                .map(|(k, v)| format!("{k:?}: {v:?}"))
                .collect();
            format!("{{{}}}", pares.join(", "))
        }
    }

    impl Formato for Csv {
        fn extensao(&self) -> &'static str {
            "csv"
        }

        fn serializar(&self, campos: &[(&str, &str)]) -> String {
            let cab: Vec<&str> = campos.iter().map(|(k, _)| *k).collect();
            let val: Vec<&str> = campos.iter().map(|(_, v)| *v).collect();
            format!("{}\n{}", cab.join(","), val.join(","))
        }
    }

    /// Funções genéricas sobre a trait selada funcionam normalmente.
    pub fn nome_do_arquivo(base: &str, f: &impl Formato) -> String {
        format!("{base}.{}", f.extensao())
    }
}

#[cfg(test)]
mod tests {
    use super::formato::{Csv, Formato, Json, nome_do_arquivo};
    use super::iter::IteratorExt;
    use super::texto::TextoExt;

    #[test]
    fn palindromos() {
        assert!("Ame a ema".eh_palindromo());
        assert!("A torre da derrota!".eh_palindromo());
        assert!(!"Rust".eh_palindromo());
        // Vale para `String` também, pela deref coercion:
        assert!(String::from("arara").eh_palindromo());
    }

    #[test]
    fn capitaliza() {
        assert_eq!("o  rato roeu".capitalizar_palavras(), "O Rato Roeu");
        assert_eq!("ética".capitalizar_palavras(), "Ética");
    }

    #[test]
    fn contagem_em_qualquer_iterador() {
        let c = "banana".chars().contagem();
        assert_eq!(c[&'a'], 3);
        assert_eq!(c[&'n'], 2);
        assert_eq!([1, 1, 2].iter().moda(), Some(&1));
        assert_eq!(Vec::<u8>::new().into_iter().moda(), None);
    }

    #[test]
    fn formatos_selados() {
        let campos = [("nome", "Ana"), ("idade", "30")];
        assert_eq!(
            Json.serializar(&campos),
            r#"{"nome": "Ana", "idade": "30"}"#
        );
        assert_eq!(Csv.serializar(&campos), "nome,idade\nAna,30");
        assert_eq!(nome_do_arquivo("dados", &Csv), "dados.csv");
    }
}

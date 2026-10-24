//! Exercício 5: dyn compatibility, resolvida de duas formas.
//!
//! O código original tinha `fn clonar(&self) -> Self` na trait, o que
//! impede `Box<dyn Comando>` (E0038).

/// Parte (a): `where Self: Sized` tira `clonar` da vtable.
/// `Box<dyn Comando>` volta a funcionar, mas `clonar` só pode ser
/// chamado em tipos concretos.
pub mod parte_a {
    pub trait Comando {
        fn nome(&self) -> String;
        fn executar(&self, pilha: &mut Vec<i64>);

        fn clonar(&self) -> Self
        where
            Self: Sized;
    }

    #[derive(Debug, PartialEq)]
    pub struct Empilhar(pub i64);

    #[derive(Debug, PartialEq)]
    pub struct Somar;

    impl Comando for Empilhar {
        fn nome(&self) -> String {
            format!("empilhar {}", self.0)
        }

        fn executar(&self, pilha: &mut Vec<i64>) {
            pilha.push(self.0);
        }

        fn clonar(&self) -> Self {
            Empilhar(self.0)
        }
    }

    impl Comando for Somar {
        fn nome(&self) -> String {
            String::from("somar")
        }

        fn executar(&self, pilha: &mut Vec<i64>) {
            let b = pilha.pop().unwrap_or(0);
            let a = pilha.pop().unwrap_or(0);
            pilha.push(a + b);
        }

        fn clonar(&self) -> Self {
            Somar
        }
    }

    pub fn rodar(programa: &[Box<dyn Comando>]) -> Vec<i64> {
        let mut pilha = Vec::new();
        for c in programa {
            c.executar(&mut pilha);
        }
        pilha
    }
}

/// Parte (b): `clonar` devolve `Box<dyn Comando>`. O tipo de retorno tem
/// tamanho conhecido (é um fat pointer), então o método cabe na vtable e
/// dá para clonar um programa inteiro de trait objects.
pub mod parte_b {
    pub trait Comando {
        fn nome(&self) -> String;
        fn executar(&self, pilha: &mut Vec<i64>);
        fn clonar(&self) -> Box<dyn Comando>;
    }

    #[derive(Debug, Clone, PartialEq)]
    pub struct Empilhar(pub i64);

    #[derive(Debug, Clone, PartialEq)]
    pub struct Somar;

    impl Comando for Empilhar {
        fn nome(&self) -> String {
            format!("empilhar {}", self.0)
        }

        fn executar(&self, pilha: &mut Vec<i64>) {
            pilha.push(self.0);
        }

        fn clonar(&self) -> Box<dyn Comando> {
            Box::new(self.clone())
        }
    }

    impl Comando for Somar {
        fn nome(&self) -> String {
            String::from("somar")
        }

        fn executar(&self, pilha: &mut Vec<i64>) {
            let b = pilha.pop().unwrap_or(0);
            let a = pilha.pop().unwrap_or(0);
            pilha.push(a + b);
        }

        fn clonar(&self) -> Box<dyn Comando> {
            Box::new(self.clone())
        }
    }

    pub fn rodar(programa: &[Box<dyn Comando>]) -> Vec<i64> {
        let mut pilha = Vec::new();
        for c in programa {
            c.executar(&mut pilha);
        }
        pilha
    }

    /// O programa seguido de uma cópia dele mesmo.
    pub fn repetir(programa: &[Box<dyn Comando>]) -> Vec<Box<dyn Comando>> {
        let mut saida = Vec::new();
        for _ in 0..2 {
            for c in programa {
                saida.push(c.clonar());
            }
        }
        saida
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn parte_a() {
        use crate::parte_a::*;
        let programa: Vec<Box<dyn Comando>> = vec![
            Box::new(Empilhar(2)),
            Box::new(Empilhar(3)),
            Box::new(Somar),
        ];
        assert_eq!(rodar(&programa), [5]);
        assert_eq!(programa[2].nome(), "somar");
        // Num tipo concreto, `clonar` continua disponível.
        assert_eq!(Empilhar(7).clonar(), Empilhar(7));
    }

    #[test]
    fn parte_b() {
        use crate::parte_b::*;
        let programa: Vec<Box<dyn Comando>> = vec![
            Box::new(Empilhar(2)),
            Box::new(Empilhar(3)),
            Box::new(Somar),
        ];
        let dobrado = repetir(&programa);
        assert_eq!(dobrado.len(), 6);
        assert_eq!(rodar(&dobrado), [5, 5]);
        assert_eq!(dobrado[3].nome(), "empilhar 2");
    }
}

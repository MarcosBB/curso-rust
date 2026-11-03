//! Tipos que usam `#[derive(Construtor)]`, e os testes do exercício 5.

use construtor_derive::Construtor;

#[derive(Debug, PartialEq, Construtor)]
pub struct Usuario {
    pub nome: String,
    pub idade: u32,
    #[construtor(padrao)]
    pub amigos: Vec<String>,
}

#[derive(Debug, PartialEq, Construtor)]
pub struct Par<T: Clone> {
    pub esquerda: T,
    pub direita: T,
}

#[derive(Debug, PartialEq, Construtor)]
pub struct Vazio {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn um_parametro_por_campo() {
        let u = Usuario::new("Ana".to_string(), 30);
        assert_eq!(u.nome, "Ana");
        assert_eq!(u.idade, 30);
        assert!(u.amigos.is_empty());
    }

    #[test]
    fn funciona_com_genericos() {
        let p = Par::new(1, 2);
        assert_eq!(
            p,
            Par {
                esquerda: 1,
                direita: 2
            }
        );
        let q = Par::new("a", "b");
        assert_eq!(q.direita, "b");
    }

    #[test]
    fn struct_sem_campos() {
        assert_eq!(Vazio::new(), Vazio {});
    }
}

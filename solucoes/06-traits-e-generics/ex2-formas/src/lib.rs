//! Exercício 2: trait com método padrão, `Display` à mão e `dyn`.

use std::f64::consts::PI;
use std::fmt;

/// `Display` como supertrait: toda forma sabe se mostrar,
/// e os métodos padrão podem usar `{self}`.
pub trait Forma: fmt::Display {
    fn area(&self) -> f64;
    fn perimetro(&self) -> f64;

    fn descricao(&self) -> String {
        format!(
            "{self}: área {:.2}, perímetro {:.2}",
            self.area(),
            self.perimetro()
        )
    }
}

pub struct Retangulo {
    pub largura: f64,
    pub altura: f64,
}

pub struct Circulo {
    pub raio: f64,
}

impl fmt::Display for Retangulo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "retângulo {}x{}", self.largura, self.altura)
    }
}

impl fmt::Display for Circulo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "círculo de raio {}", self.raio)
    }
}

impl Forma for Retangulo {
    fn area(&self) -> f64 {
        self.largura * self.altura
    }

    fn perimetro(&self) -> f64 {
        2.0 * (self.largura + self.altura)
    }
}

impl Forma for Circulo {
    fn area(&self) -> f64 {
        PI * self.raio * self.raio
    }

    fn perimetro(&self) -> f64 {
        2.0 * PI * self.raio
    }

    // Substitui o método padrão.
    fn descricao(&self) -> String {
        format!("{self}: área {:.2}", self.area())
    }
}

/// A forma de maior área, ou `None` se a lista estiver vazia.
pub fn maior_area(formas: &[Box<dyn Forma>]) -> Option<&dyn Forma> {
    let mut maior: Option<&dyn Forma> = None;
    for f in formas {
        match maior {
            Some(m) if m.area() >= f.area() => {}
            _ => maior = Some(f.as_ref()),
        }
    }
    maior
}

/// Soma das áreas de formas do mesmo tipo (versão genérica).
pub fn area_total<T: Forma>(formas: &[T]) -> f64 {
    let mut total = 0.0;
    for f in formas {
        total += f.area();
    }
    total
}

#[cfg(test)]
mod tests {
    use super::*;

    fn exemplos() -> Vec<Box<dyn Forma>> {
        vec![
            Box::new(Retangulo {
                largura: 2.0,
                altura: 3.0,
            }),
            Box::new(Circulo { raio: 1.5 }),
            Box::new(Retangulo {
                largura: 1.0,
                altura: 1.0,
            }),
        ]
    }

    #[test]
    fn display() {
        let r = Retangulo {
            largura: 2.0,
            altura: 3.5,
        };
        assert_eq!(r.to_string(), "retângulo 2x3.5");
        assert_eq!(Circulo { raio: 1.0 }.to_string(), "círculo de raio 1");
    }

    #[test]
    fn descricao_padrao_e_substituida() {
        let formas = exemplos();
        assert_eq!(
            formas[0].descricao(),
            "retângulo 2x3: área 6.00, perímetro 10.00"
        );
        assert_eq!(formas[1].descricao(), "círculo de raio 1.5: área 7.07");
    }

    #[test]
    fn maior_area_entre_tipos_diferentes() {
        let formas = exemplos();
        let maior = maior_area(&formas).unwrap();
        assert_eq!(maior.to_string(), "círculo de raio 1.5");
        assert!(maior_area(&[]).is_none());
    }

    #[test]
    fn area_total_generica() {
        let quadrados = [
            Retangulo {
                largura: 1.0,
                altura: 1.0,
            },
            Retangulo {
                largura: 2.0,
                altura: 2.0,
            },
        ];
        assert_eq!(area_total(&quadrados), 5.0);
    }
}

//! Exemplos das lições 06 e 07: trait objects, fat pointers, upcasting,
//! `Any` e as duas formas de usar a mesma trait (genérica e `dyn`).

use std::any::Any;
use std::f64::consts::PI;
use std::fmt::Debug;

/// `Debug` e `Any` como supertraits: todo `dyn Forma` pode virar
/// `&dyn Debug` ou `&dyn Any` (trait upcasting, Rust 1.86).
trait Forma: Debug + Any {
    fn area(&self) -> f64;

    fn nome(&self) -> String {
        String::from("forma")
    }
}

#[derive(Debug)]
struct Quadrado {
    lado: f64,
}

#[derive(Debug)]
struct Circulo {
    raio: f64,
}

impl Forma for Quadrado {
    fn area(&self) -> f64 {
        self.lado * self.lado
    }

    fn nome(&self) -> String {
        format!("quadrado de lado {}", self.lado)
    }
}

impl Forma for Circulo {
    fn area(&self) -> f64 {
        PI * self.raio * self.raio
    }

    fn nome(&self) -> String {
        format!("círculo de raio {}", self.raio)
    }
}

/// Genérica: todos os elementos são do MESMO tipo `T`.
/// Uma cópia por tipo (monomorfização), chamadas diretas.
fn area_total<T: Forma>(formas: &[T]) -> f64 {
    let mut total = 0.0;
    for f in formas {
        total += f.area();
    }
    total
}

/// Com `dyn`: cada elemento pode ser de um tipo diferente.
/// Uma cópia só, chamadas pela vtable.
fn area_total_dyn(formas: &[Box<dyn Forma>]) -> f64 {
    let mut total = 0.0;
    for f in formas {
        total += f.area();
    }
    total
}

fn imprimir_debug(x: &dyn Debug) {
    println!("debug: {x:?}");
}

fn main() {
    // Coleção heterogênea: só funciona com trait objects.
    let formas: Vec<Box<dyn Forma>> = vec![
        Box::new(Quadrado { lado: 2.0 }),
        Box::new(Circulo { raio: 1.0 }),
    ];
    for f in &formas {
        println!("{:<20} área {:.2}", f.nome(), f.area());
    }
    println!("total (dyn): {:.2}", area_total_dyn(&formas));

    // Coleção homogênea: a versão genérica basta.
    let quadrados = [Quadrado { lado: 1.0 }, Quadrado { lado: 3.0 }];
    println!("total (genérico): {:.2}", area_total(&quadrados));

    // Tamanhos: referência comum x fat pointer.
    println!(
        "&Quadrado: {} bytes, &dyn Forma: {} bytes, Box<dyn Forma>: {} bytes, &[u8]: {} bytes",
        size_of::<&Quadrado>(),
        size_of::<&dyn Forma>(),
        size_of::<Box<dyn Forma>>(),
        size_of::<&[u8]>()
    );

    // Upcasting: &dyn Forma -> &dyn Debug.
    let primeira: &dyn Forma = &*formas[0];
    imprimir_debug(primeira);

    // Upcasting para &dyn Any e downcast de volta ao tipo concreto.
    for f in &formas {
        let qualquer: &dyn Any = &**f;
        match qualquer.downcast_ref::<Circulo>() {
            Some(c) => println!("é um Circulo, raio {}", c.raio),
            None => println!("não é um Circulo"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generica_e_dyn_concordam() {
        let a = area_total(&[Quadrado { lado: 2.0 }]);
        let b = area_total_dyn(&[Box::new(Quadrado { lado: 2.0 })]);
        assert_eq!(a, b);
    }

    #[test]
    fn fat_pointer_tem_dois_ponteiros() {
        assert_eq!(size_of::<&dyn Forma>(), 2 * size_of::<usize>());
        assert_eq!(size_of::<&Quadrado>(), size_of::<usize>());
    }

    #[test]
    fn downcast() {
        let f: Box<dyn Forma> = Box::new(Circulo { raio: 2.0 });
        let a: &dyn Any = &*f;
        assert!(a.is::<Circulo>());
        assert!(a.downcast_ref::<Quadrado>().is_none());
    }
}

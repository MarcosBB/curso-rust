//! Exercício 3: fachada com `pub use`.
//!
//! Os módulos são privados; a API é reexportada aqui na raiz.
//!
//! ```
//! use formas::{Area, Circulo, Retangulo, area_total};
//!
//! let c = Circulo { raio: 1.0 };
//! let r = Retangulo { largura: 2.0, altura: 3.0 };
//! assert_eq!(r.area(), 6.0);
//! assert!(area_total(&[&c, &r]) > 9.0);
//! ```
//!
//! O caminho interno não faz parte da API:
//!
//! ```compile_fail
//! let c = formas::circulo::Circulo { raio: 1.0 };
//! ```

mod area;
mod circulo;
mod retangulo;

pub use area::{Area, area_total};
pub use circulo::Circulo;
pub use retangulo::Retangulo;

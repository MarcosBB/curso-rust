//! Lição 01: sobrecarga de operadores com as traits de `std::ops`.

use std::fmt;
use std::iter::Sum;
use std::ops::{Add, AddAssign, Index, IndexMut, Mul, Neg, Sub};

// ---------------------------------------------------------------------------
// Um tipo pequeno e `Copy`: operadores por valor.
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Vetor {
    pub x: f64,
    pub y: f64,
}

impl Vetor {
    pub fn new(x: f64, y: f64) -> Self {
        Vetor { x, y }
    }
}

impl Add for Vetor {
    type Output = Vetor;

    fn add(self, outro: Vetor) -> Vetor {
        Vetor::new(self.x + outro.x, self.y + outro.y)
    }
}

impl Sub for Vetor {
    type Output = Vetor;

    fn sub(self, outro: Vetor) -> Vetor {
        Vetor::new(self.x - outro.x, self.y - outro.y)
    }
}

impl Neg for Vetor {
    type Output = Vetor;

    fn neg(self) -> Vetor {
        Vetor::new(-self.x, -self.y)
    }
}

// Vetor * escalar
impl Mul<f64> for Vetor {
    type Output = Vetor;

    fn mul(self, k: f64) -> Vetor {
        Vetor::new(self.x * k, self.y * k)
    }
}

// escalar * Vetor: `f64` é da std, mas `Vetor` é nosso, então a orphan rule deixa.
impl Mul<Vetor> for f64 {
    type Output = Vetor;

    fn mul(self, v: Vetor) -> Vetor {
        v * self
    }
}

impl AddAssign for Vetor {
    fn add_assign(&mut self, outro: Vetor) {
        self.x += outro.x;
        self.y += outro.y;
    }
}

// Permite `vetores.into_iter().sum::<Vetor>()`.
impl Sum for Vetor {
    fn sum<I: Iterator<Item = Vetor>>(iter: I) -> Vetor {
        iter.fold(Vetor::default(), |acc, v| acc + v)
    }
}

impl fmt::Display for Vetor {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}

// ---------------------------------------------------------------------------
// Um tipo que é dono de memória: operadores por referência.
// ---------------------------------------------------------------------------

/// Coeficientes do menor para o maior grau: `[1, 2]` é `1 + 2x`.
#[derive(Debug, Clone, PartialEq)]
pub struct Polinomio(pub Vec<i64>);

impl Add<&Polinomio> for &Polinomio {
    type Output = Polinomio;

    fn add(self, outro: &Polinomio) -> Polinomio {
        let n = self.0.len().max(outro.0.len());
        let coef = (0..n)
            .map(|i| self.0.get(i).unwrap_or(&0) + outro.0.get(i).unwrap_or(&0))
            .collect();
        Polinomio(coef)
    }
}

// A versão por valor reaproveita a por referência.
impl Add for Polinomio {
    type Output = Polinomio;

    fn add(self, outro: Polinomio) -> Polinomio {
        &self + &outro
    }
}

// ---------------------------------------------------------------------------
// Indexação: `m[(linha, coluna)]`.
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub struct Matriz {
    colunas: usize,
    dados: Vec<f64>,
}

impl Matriz {
    pub fn zeros(linhas: usize, colunas: usize) -> Self {
        Matriz {
            colunas,
            dados: vec![0.0; linhas * colunas],
        }
    }
}

impl Index<(usize, usize)> for Matriz {
    type Output = f64;

    fn index(&self, (l, c): (usize, usize)) -> &f64 {
        assert!(c < self.colunas, "coluna {c} fora da matriz");
        &self.dados[l * self.colunas + c]
    }
}

impl IndexMut<(usize, usize)> for Matriz {
    fn index_mut(&mut self, (l, c): (usize, usize)) -> &mut f64 {
        assert!(c < self.colunas, "coluna {c} fora da matriz");
        &mut self.dados[l * self.colunas + c]
    }
}

fn main() {
    let a = Vetor::new(1.0, 2.0);
    let b = Vetor::new(3.0, 4.0);
    println!("a + b = {}", a + b);
    println!("a - b = {}", a - b);
    println!("-a = {}", -a);
    println!("a * 2 = {}, 2 * a = {}", a * 2.0, 2.0 * a);

    let mut posicao = Vetor::default();
    posicao += a;
    posicao += b;
    println!("posicao = {posicao}");

    let total: Vetor = [a, b, Vetor::new(0.5, 0.5)].into_iter().sum();
    println!("soma = {total}");

    let p = Polinomio(vec![1, 2]);
    let q = Polinomio(vec![0, 0, 3]);
    let r = &p + &q;
    println!("{p:?} + {q:?} = {r:?}");

    let mut m = Matriz::zeros(2, 3);
    m[(1, 2)] = 7.5;
    m[(0, 0)] += 1.0;
    println!("m[(1, 2)] = {}, m[(0, 0)] = {}", m[(1, 2)], m[(0, 0)]);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn aritmetica_de_vetor() {
        let a = Vetor::new(1.0, 2.0);
        let b = Vetor::new(3.0, 4.0);
        assert_eq!(a + b, Vetor::new(4.0, 6.0));
        assert_eq!(b - a, Vetor::new(2.0, 2.0));
        assert_eq!(-a, Vetor::new(-1.0, -2.0));
        assert_eq!(a * 3.0, 3.0 * a);
    }

    #[test]
    fn add_assign_e_sum() {
        let mut v = Vetor::default();
        v += Vetor::new(1.0, 1.0);
        assert_eq!(v, Vetor::new(1.0, 1.0));
        let s: Vetor = vec![v, v, v].into_iter().sum();
        assert_eq!(s, Vetor::new(3.0, 3.0));
    }

    #[test]
    fn polinomio_por_referencia_nao_move() {
        let p = Polinomio(vec![1, 2]);
        let q = Polinomio(vec![0, 0, 3]);
        assert_eq!(&p + &q, Polinomio(vec![1, 2, 3]));
        // `p` e `q` continuam valendo:
        assert_eq!(p + q, Polinomio(vec![1, 2, 3]));
    }

    #[test]
    fn indice_de_matriz() {
        let mut m = Matriz::zeros(2, 2);
        m[(1, 0)] = 4.0;
        assert_eq!(m[(1, 0)], 4.0);
        assert_eq!(m[(0, 1)], 0.0);
    }

    #[test]
    #[should_panic(expected = "coluna 5 fora da matriz")]
    fn coluna_fora_da_matriz() {
        let m = Matriz::zeros(2, 2);
        let _ = m[(0, 5)];
    }
}

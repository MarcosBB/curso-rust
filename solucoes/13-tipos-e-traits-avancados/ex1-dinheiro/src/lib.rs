use std::fmt;
use std::iter::Sum;
use std::ops::{Add, AddAssign, Mul, Neg, Sub};

/// Valor em centavos de real.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct Dinheiro(pub i64);

impl Dinheiro {
    pub fn reais(reais: i64, centavos: i64) -> Self {
        Dinheiro(reais * 100 + centavos)
    }
}

impl Add for Dinheiro {
    type Output = Dinheiro;

    fn add(self, outro: Dinheiro) -> Dinheiro {
        Dinheiro(self.0 + outro.0)
    }
}

impl Sub for Dinheiro {
    type Output = Dinheiro;

    fn sub(self, outro: Dinheiro) -> Dinheiro {
        Dinheiro(self.0 - outro.0)
    }
}

impl Neg for Dinheiro {
    type Output = Dinheiro;

    fn neg(self) -> Dinheiro {
        Dinheiro(-self.0)
    }
}

impl Mul<i64> for Dinheiro {
    type Output = Dinheiro;

    fn mul(self, k: i64) -> Dinheiro {
        Dinheiro(self.0 * k)
    }
}

impl Mul<Dinheiro> for i64 {
    type Output = Dinheiro;

    fn mul(self, d: Dinheiro) -> Dinheiro {
        d * self
    }
}

impl AddAssign for Dinheiro {
    fn add_assign(&mut self, outro: Dinheiro) {
        self.0 += outro.0;
    }
}

impl Sum for Dinheiro {
    fn sum<I: Iterator<Item = Dinheiro>>(iter: I) -> Dinheiro {
        iter.fold(Dinheiro::default(), |acc, d| acc + d)
    }
}

impl<'a> Sum<&'a Dinheiro> for Dinheiro {
    fn sum<I: Iterator<Item = &'a Dinheiro>>(iter: I) -> Dinheiro {
        iter.copied().sum()
    }
}

impl fmt::Display for Dinheiro {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let sinal = if self.0 < 0 { "-" } else { "" };
        let abs = self.0.unsigned_abs();
        write!(f, "{sinal}R$ {},{:02}", abs / 100, abs % 100)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn soma_e_subtracao() {
        let a = Dinheiro::reais(10, 50);
        let b = Dinheiro(275);
        assert_eq!(a + b, Dinheiro(1325));
        assert_eq!(a - b, Dinheiro(775));
        assert_eq!(-a, Dinheiro(-1050));
    }

    #[test]
    fn multiplica_dos_dois_lados() {
        let preco = Dinheiro(199);
        assert_eq!(preco * 3, Dinheiro(597));
        assert_eq!(3 * preco, Dinheiro(597));
    }

    #[test]
    fn acumula_no_lugar() {
        let mut total = Dinheiro::default();
        total += Dinheiro(100);
        total += Dinheiro(-30);
        assert_eq!(total, Dinheiro(70));
    }

    #[test]
    fn soma_de_iterador() {
        let itens = [Dinheiro(100), Dinheiro(250), Dinheiro(5)];
        let por_valor: Dinheiro = itens.iter().copied().sum();
        let por_ref: Dinheiro = itens.iter().sum();
        assert_eq!(por_valor, Dinheiro(355));
        assert_eq!(por_ref, Dinheiro(355));
    }

    #[test]
    fn formata_em_reais() {
        assert_eq!(Dinheiro(1234).to_string(), "R$ 12,34");
        assert_eq!(Dinheiro(5).to_string(), "R$ 0,05");
        assert_eq!(Dinheiro(-5).to_string(), "-R$ 0,05");
        assert_eq!(Dinheiro(-100_000).to_string(), "-R$ 1000,00");
    }
}

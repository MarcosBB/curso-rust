//! `moeda` 2.0: mesma ideia, com formatação (mudança incompatível: novo `Display`
//! e o campo agora é privado, criado por `Real::centavos`).

use std::fmt;

/// Um valor em reais, guardado em centavos.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Real(i64);

impl Real {
    /// Cria um valor a partir de centavos.
    pub fn centavos(c: i64) -> Real {
        Real(c)
    }
}

impl fmt::Display for Real {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "R$ {},{:02}", self.0 / 100, self.0 % 100)
    }
}

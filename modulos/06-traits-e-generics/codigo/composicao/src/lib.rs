//! Exemplos da lição 07: composição em vez de herança.
//!
//! `ContaBytes` embrulha qualquer `Write` e também é um `Write`: repassa
//! cada escrita para o de dentro e conta os bytes no caminho. É o padrão
//! decorador, feito com composição + uma trait, sem herança.

use std::fmt;
use std::io::{self, Write};

pub struct ContaBytes<W: Write> {
    interno: W,
    total: usize,
}

impl<W: Write> ContaBytes<W> {
    pub fn new(interno: W) -> Self {
        Self { interno, total: 0 }
    }

    pub fn total(&self) -> usize {
        self.total
    }

    /// Devolve o `Write` de dentro, consumindo o embrulho.
    pub fn into_inner(self) -> W {
        self.interno
    }
}

impl<W: Write> Write for ContaBytes<W> {
    // Os dois métodos obrigatórios de `Write`. Todos os outros
    // (`write_all`, `write_fmt`, que o `writeln!` usa...) são métodos
    // padrão construídos em cima destes.
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let n = self.interno.write(buf)?;
        self.total += n;
        Ok(n)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.interno.flush()
    }
}

/// Supertrait: quem implementa `Moldura` precisa implementar `Display`.
/// Não é herança: `Moldura` só pode USAR o `Display`, não o recebe pronto.
pub trait Moldura: fmt::Display {
    fn emoldurar(&self) -> String {
        let texto = self.to_string();
        let borda = "-".repeat(texto.chars().count() + 4);
        format!("{borda}\n| {texto} |\n{borda}")
    }
}

pub struct Titulo(pub String);

impl fmt::Display for Titulo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Moldura for Titulo {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn conta_e_repassa() {
        let mut saida = ContaBytes::new(Vec::new());
        writeln!(saida, "olá").unwrap(); // "olá" tem 4 bytes em UTF-8, mais o \n
        write!(saida, "{}", 42).unwrap();
        assert_eq!(saida.total(), 7);
        assert_eq!(saida.into_inner(), "olá\n42".as_bytes());
    }

    #[test]
    fn decoradores_se_empilham() {
        let mut duplo = ContaBytes::new(ContaBytes::new(Vec::new()));
        write!(duplo, "abc").unwrap();
        assert_eq!(duplo.total(), 3);
        assert_eq!(duplo.into_inner().total(), 3);
    }

    #[test]
    fn moldura() {
        let t = Titulo(String::from("Rust"));
        assert_eq!(t.emoldurar(), "--------\n| Rust |\n--------");
    }
}

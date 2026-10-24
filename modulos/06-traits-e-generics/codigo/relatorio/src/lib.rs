//! Exemplos da lição 03: trait bounds, `where` e `impl Trait`.
//!
//! A ideia central: funções que escrevem em `&mut impl Write` funcionam com
//! o stdout no programa e com um `Vec<u8>` nos testes.

use std::fmt::Display;
use std::io::{self, BufRead, Write};

/// Escreve as linhas numeradas em qualquer destino que implemente `Write`.
pub fn relatorio(linhas: &[&str], out: &mut impl Write) -> io::Result<()> {
    for (i, linha) in linhas.iter().enumerate() {
        writeln!(out, "{}: {linha}", i + 1)?;
    }
    Ok(())
}

/// Conta as linhas de qualquer fonte que implemente `BufRead`
/// (um arquivo com `BufReader`, o stdin, ou um `&[u8]` nos testes).
pub fn contar_linhas<R: BufRead>(fonte: R) -> io::Result<usize> {
    let mut total = 0;
    for linha in fonte.lines() {
        linha?;
        total += 1;
    }
    Ok(total)
}

/// Um par de valores do mesmo tipo.
pub struct Par<T> {
    pub x: T,
    pub y: T,
}

// Para todo `T`.
impl<T> Par<T> {
    pub fn new(x: T, y: T) -> Self {
        Self { x, y }
    }
}

// Só para os `T` que sabem se comparar e se mostrar.
impl<T: Display + PartialOrd> Par<T> {
    pub fn maior(&self) -> String {
        if self.x >= self.y {
            format!("x = {}", self.x)
        } else {
            format!("y = {}", self.y)
        }
    }
}

/// Bounds na cláusula `where`: mesma coisa que `<T: Display + Clone>`,
/// mas mais fácil de ler quando a lista cresce.
pub fn repetir<T>(valor: T, vezes: usize) -> Vec<String>
where
    T: Display + Clone,
{
    let mut saida = Vec::new();
    for _ in 0..vezes {
        saida.push(valor.clone().to_string());
    }
    saida
}

/// `impl Trait` no retorno: "algum tipo que implementa `Display`".
/// O tipo concreto (aqui, `String`) fica escondido de quem chama.
pub fn rotulo(nome: &str, valor: i32) -> impl Display {
    format!("{nome}={valor}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn relatorio_em_memoria() {
        let mut buf: Vec<u8> = Vec::new();
        relatorio(&["alfa", "beta"], &mut buf).unwrap();
        assert_eq!(String::from_utf8(buf).unwrap(), "1: alfa\n2: beta\n");
    }

    #[test]
    fn contar_linhas_de_bytes() {
        // `&[u8]` implementa `BufRead`: serve de "arquivo" nos testes.
        let texto: &[u8] = b"um\ndois\ntres\n";
        assert_eq!(contar_linhas(texto).unwrap(), 3);
    }

    #[test]
    fn impl_condicional() {
        assert_eq!(Par::new(3, 7).maior(), "y = 7");
        assert_eq!(Par::new("b", "a").maior(), "x = b");
    }

    #[test]
    fn where_e_impl_trait() {
        assert_eq!(repetir('x', 3), ["x", "x", "x"]);
        assert_eq!(rotulo("porta", 8080).to_string(), "porta=8080");
    }
}

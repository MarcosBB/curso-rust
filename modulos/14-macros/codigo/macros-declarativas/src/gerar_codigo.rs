//! Lição 03: macros que geram itens (tipos e `impl`s), não só expressões.

use std::fmt;
use std::ops::Add;

/// Gera um *newtype* para cada unidade, com `Display` e `Add`.
macro_rules! unidades {
    ($($nome:ident($tipo:ty) => $sufixo:literal),+ $(,)?) => {
        $(
            #[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
            pub struct $nome(pub $tipo);

            impl fmt::Display for $nome {
                fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                    write!(f, "{} {}", self.0, $sufixo)
                }
            }

            impl Add for $nome {
                type Output = $nome;
                fn add(self, outro: $nome) -> $nome {
                    $nome(self.0 + outro.0)
                }
            }
        )+
    };
}

unidades! {
    Metros(f64) => "m",
    Segundos(f64) => "s",
    Bytes(u64) => "B",
}

/// Exemplos da lição, impressos por `cargo run`.
pub fn demonstrar() {
    let trajeto = Metros(120.0) + Metros(30.5);
    let tempo = Segundos(12.0);
    println!("{trajeto} em {tempo}; arquivo de {}", Bytes(2048));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unidades_somam_e_formatam() {
        assert_eq!(Metros(1.0) + Metros(2.0), Metros(3.0));
        assert_eq!(Bytes(10).to_string(), "10 B");
    }
}

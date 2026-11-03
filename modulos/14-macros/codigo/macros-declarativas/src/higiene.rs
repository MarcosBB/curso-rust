//! Lição 04: higiene, `$crate` e `#[macro_export]`.

/// Usada pela macro `registrar!`. Precisa ser `pub`: o código gerado roda na
/// crate de quem chama a macro, que pode ser outra.
#[doc(hidden)]
pub fn formatar_registro(nivel: &str, mensagem: &str) -> String {
    format!("[{nivel}] {mensagem}")
}

/// `registrar!(info, "texto {}", x)`: monta uma linha de registro.
///
/// `#[macro_export]` publica a macro na **raiz** da crate:
/// `macros_declarativas::registrar!`. O `$crate` garante que o caminho da
/// função auxiliar funciona em qualquer crate que use a macro.
#[macro_export]
macro_rules! registrar {
    ($nivel:ident, $($arg:tt)+) => {
        $crate::higiene::formatar_registro(stringify!($nivel), &format!($($arg)+))
    };
}

/// Cria uma variável local. Por causa da higiene, `x` não vaza para fora.
macro_rules! com_temporaria {
    ($e:expr) => {{
        let x = 10;
        x + $e
    }};
}

/// Recebe o nome do chamador: aí o identificador é dele, e vale lá fora.
macro_rules! declarar {
    ($nome:ident = $valor:expr) => {
        let $nome = $valor;
    };
}

/// Exemplos da lição, impressos por `cargo run`.
pub fn demonstrar() {
    let x = 1;
    // O `x` de dentro da macro (10) e o daqui (1) não se confundem.
    println!("com_temporaria!(x * 2) = {}", com_temporaria!(x * 2));

    declarar!(total = 40 + 2);
    println!("total = {total}");

    println!("{}", crate::registrar!(aviso, "disco em {}%", 91));

    // Uma macro de outro módulo, pelo caminho criado com `pub(crate) use`.
    println!(
        "crate::basico::quadrado!(7) = {}",
        crate::basico::quadrado!(7)
    );
}

#[cfg(test)]
mod tests {
    #[test]
    fn higiene_separa_os_x() {
        let x = 1;
        assert_eq!(com_temporaria!(x), 11);
        declarar!(y = 5);
        assert_eq!(y, 5);
    }
}

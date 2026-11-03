//! Lição 03: repetições e recursão.

/// `hashmap!{ "a" => 1, "b" => 2 }`, com vírgula final opcional.
macro_rules! hashmap {
    ($($chave:expr => $valor:expr),* $(,)?) => {{
        // Com `hashmap!{}` vazio, o `mut` sobra: o `allow` cala o aviso.
        #[allow(unused_mut)]
        let mut mapa = ::std::collections::HashMap::new();
        $(
            mapa.insert($chave, $valor);
        )*
        mapa
    }};
}

/// Soma um ou mais números: `+` como separador **dentro** da expansão.
macro_rules! soma {
    ($primeiro:expr $(, $resto:expr)* $(,)?) => {
        $primeiro $(+ $resto)*
    };
}

/// Conta os argumentos por recursão: cada passo tira um token.
macro_rules! contar {
    () => { 0usize };
    ($primeiro:tt $($resto:tt)*) => { 1usize + contar!($($resto)*) };
}

/// Repetição aninhada: uma `Vec<Vec<_>>` a partir de linhas separadas por `;`.
macro_rules! matriz {
    ($($($x:expr),+);+ $(;)?) => {
        vec![$(vec![$($x),+]),+]
    };
}

/// Um comando da mini-linguagem de `comandos!`.
#[derive(Debug, PartialEq)]
pub enum Comando {
    Andar(u32),
    Girar,
    Parar,
}

/// *tt muncher*: consome a entrada pedaço por pedaço, acumulando o resultado.
macro_rules! comandos {
    // Fim da entrada: devolve o que foi acumulado.
    (@acc [$($feito:expr),*]) => {
        vec![$($feito),*]
    };
    (@acc [$($feito:expr),*] andar $n:literal; $($resto:tt)*) => {
        comandos!(@acc [$($feito,)* $crate::repeticoes::Comando::Andar($n)] $($resto)*)
    };
    (@acc [$($feito:expr),*] girar; $($resto:tt)*) => {
        comandos!(@acc [$($feito,)* $crate::repeticoes::Comando::Girar] $($resto)*)
    };
    (@acc [$($feito:expr),*] parar; $($resto:tt)*) => {
        comandos!(@acc [$($feito,)* $crate::repeticoes::Comando::Parar] $($resto)*)
    };
    // Ponto de entrada: começa com o acumulador vazio.
    ($($entrada:tt)*) => {
        comandos!(@acc [] $($entrada)*)
    };
}

/// Exemplos da lição, impressos por `cargo run`.
pub fn demonstrar() {
    let precos = hashmap! {
        "café" => 18.5,
        "pão" => 0.9,
    };
    let mut nomes: Vec<_> = precos.keys().collect();
    nomes.sort();
    println!("{nomes:?}");

    println!("soma!(1, 2, 3) = {}", soma!(1, 2, 3));
    const N: usize = contar!(a b c d);
    println!("contar!(a b c d) = {N}");
    println!("{:?}", matriz![1, 2, 3; 4, 5, 6]);
    println!("{:?}", comandos!(andar 3; girar; andar 1; parar;));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hashmap_aceita_virgula_final_e_vazio() {
        let m = hashmap! { 1 => "um", 2 => "dois", };
        assert_eq!(m[&2], "dois");
        let vazio: std::collections::HashMap<i32, i32> = hashmap! {};
        assert!(vazio.is_empty());
    }

    #[test]
    fn soma_e_contar() {
        assert_eq!(soma!(5), 5);
        assert_eq!(soma!(1, 2, 3,), 6);
        assert_eq!(contar!(), 0);
        // Um array cujo tamanho é calculado pela macro, na compilação.
        let a = [0u8; contar!(x y z)];
        assert_eq!(a.len(), 3);
    }

    #[test]
    fn matriz_aninhada() {
        assert_eq!(matriz![1, 2; 3, 4;], vec![vec![1, 2], vec![3, 4]]);
    }

    #[test]
    fn comandos_vira_vec() {
        use Comando::*;
        assert_eq!(comandos!(andar 2; girar;), vec![Andar(2), Girar]);
        let vazio: Vec<Comando> = comandos!();
        assert!(vazio.is_empty());
    }
}

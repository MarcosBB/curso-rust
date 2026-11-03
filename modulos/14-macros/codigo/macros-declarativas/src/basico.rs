//! Lição 02: regras, fragmentos e a avaliação dupla.

/// Ingênua: `$x` aparece duas vezes, então a expressão roda duas vezes.
macro_rules! quadrado_ingenuo {
    ($x:expr) => {
        $x * $x
    };
}

/// Avalia `$x` uma vez só: o `match` guarda o valor em `v`.
macro_rules! quadrado {
    ($x:expr) => {
        match $x {
            v => v * v,
        }
    };
}

/// Várias regras: a primeira que casar vence.
macro_rules! cor {
    (vermelho) => {
        [255, 0, 0]
    };
    (verde) => {
        [0, 255, 0]
    };
    (rgb($r:expr, $g:expr, $b:expr)) => {
        [$r, $g, $b]
    };
    ($outra:ident) => {
        compile_error!(concat!("cor desconhecida: ", stringify!($outra)))
    };
}

/// Mostra o código que recebeu e o valor dele, como um `dbg!` simplificado.
macro_rules! mostrar {
    ($e:expr) => {
        format!("{} = {:?}", stringify!($e), $e)
    };
}

// Macros de `macro_rules!` só valem no texto *abaixo* da definição. Este `use`
// transforma `quadrado!` num caminho normal, `crate::basico::quadrado!`.
pub(crate) use quadrado;

/// Conta quantas vezes é chamada, para mostrar a avaliação dupla.
pub fn proximo(contador: &mut i32) -> i32 {
    *contador += 1;
    *contador
}

/// Exemplos da lição, impressos por `cargo run`.
pub fn demonstrar() {
    println!("quadrado!(1 + 2) = {}", quadrado!(1 + 2));

    let mut c = 0;
    let ingenuo = quadrado_ingenuo!(proximo(&mut c));
    println!("quadrado_ingenuo!(proximo()) = {ingenuo}, proximo() rodou {c} vezes");

    let mut c = 0;
    let certo = quadrado!(proximo(&mut c));
    println!("quadrado!(proximo()) = {certo}, proximo() rodou {c} vez");

    let laranja: [u8; 3] = cor!(rgb(249, 115, 22));
    println!("{:?} {laranja:?}", cor!(verde));
    println!("{}", mostrar!(2 * 21));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quadrado_respeita_a_precedencia() {
        assert_eq!(quadrado!(1 + 2), 9);
        assert_eq!(quadrado_ingenuo!(1 + 2), 9);
    }

    #[test]
    fn quadrado_avalia_uma_vez() {
        let mut c = 0;
        assert_eq!(quadrado_ingenuo!(proximo(&mut c)), 2); // 1 * 2
        assert_eq!(c, 2);

        let mut c = 0;
        assert_eq!(quadrado!(proximo(&mut c)), 1);
        assert_eq!(c, 1);
    }

    #[test]
    fn cor_escolhe_a_regra() {
        assert_eq!(cor!(vermelho), [255, 0, 0]);
        assert_eq!(cor!(rgb(1, 2, 3)), [1, 2, 3]);
    }

    #[test]
    fn mostrar_usa_o_codigo_fonte() {
        assert_eq!(mostrar!(1 + 1), "1 + 1 = 2");
    }
}

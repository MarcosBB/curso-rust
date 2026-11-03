//! Lições 07 e 08: variância, `T: 'a`, HRTB e captura em `impl Trait`.

use std::collections::BTreeSet;
use std::fmt::Debug;

// ---------------------------------------------------------------------------
// Covariância: `&'static str` serve onde se espera `&'a str`.
// ---------------------------------------------------------------------------

fn maior<'a>(v: &[&'a str]) -> &'a str {
    v.iter().copied().max_by_key(|s| s.len()).unwrap_or("")
}

// ---------------------------------------------------------------------------
// Invariância: atrás de `&mut`, o lifetime não pode encolher.
// ---------------------------------------------------------------------------

fn adicionar<'a>(v: &mut Vec<&'a str>, s: &'a str) {
    v.push(s);
}

/// A correção do exemplo da lição: o `Vec` é dono dos textos.
fn registrar(nomes: &mut Vec<String>) {
    let local = String::from("bia");
    nomes.push(local);
}

// ---------------------------------------------------------------------------
// `T: 'a` e `'a: 'b`.
// ---------------------------------------------------------------------------

fn empacotar<'a, T: Debug + 'a>(x: T) -> Box<dyn Debug + 'a> {
    Box::new(x)
}

/// `'longo: 'curto`: "'longo vive pelo menos tanto quanto 'curto".
fn primeiro_ou<'curto, 'longo: 'curto>(v: &'longo [String], padrao: &'curto str) -> &'curto str {
    v.first().map(String::as_str).unwrap_or(padrao)
}

// ---------------------------------------------------------------------------
// HRTB.
// ---------------------------------------------------------------------------

/// Uma closure que devolve um pedaço do argumento precisa deste bound.
fn com_hrtb<F>(f: F) -> F
where
    F: for<'a> Fn(&'a str) -> &'a str,
{
    f
}

/// Qualquer coleção que, emprestada, itera `&i32`.
fn resumo<C>(c: C) -> (i32, usize)
where
    for<'a> &'a C: IntoIterator<Item = &'a i32>,
{
    let soma = (&c).into_iter().sum();
    let n = (&c).into_iter().count();
    (soma, n)
}

/// Aplica a mesma transformação a vários textos com lifetimes diferentes.
fn aplicar_em_todos(textos: &[&str], f: impl Fn(&str) -> &str) -> Vec<String> {
    textos.iter().map(|t| f(t).to_string()).collect()
}

// ---------------------------------------------------------------------------
// Captura de lifetimes em `impl Trait` (edition 2024) e `use<..>`.
// ---------------------------------------------------------------------------

/// Sem o `+ use<>`, o iterador "prenderia" `v` emprestado.
fn indices_pares(v: &[i32]) -> impl Iterator<Item = usize> + use<> {
    (0..v.len()).step_by(2)
}

/// Este realmente empresta `v`: a captura padrão é a certa.
fn pares(v: &[i32]) -> impl Iterator<Item = i32> {
    v.iter().copied().filter(|x| x % 2 == 0)
}

/// Com parâmetro de tipo, ele precisa aparecer na lista.
fn repetir<T: Clone>(x: &T, n: usize) -> impl Iterator<Item = T> + use<T> {
    std::iter::repeat_n(x.clone(), n)
}

fn main() {
    let fixos: Vec<&'static str> = vec!["ana", "beatriz"];
    let local = String::from("carolina");
    let mistos: Vec<&str> = vec![fixos[0], &local];
    println!("{} {}", maior(&fixos), maior(&mistos));

    let mut curtos: Vec<&str> = vec!["ana"];
    adicionar(&mut curtos, &local);
    println!("{curtos:?}");

    let mut nomes = vec![String::from("ana")];
    registrar(&mut nomes);
    println!("{nomes:?}");

    let texto = String::from("emprestado");
    let caixa = empacotar(&texto);
    println!("{caixa:?}");

    let lista = vec![String::from("primeiro")];
    println!(
        "{} {}",
        primeiro_ou(&lista, "nada"),
        primeiro_ou(&[], "nada")
    );

    let primeira = com_hrtb(|s| s.split(' ').next().unwrap_or(""));
    let nome = String::from("Bia Lima");
    println!("{} {}", primeira("Ana Souza"), primeira(&nome));

    println!("{:?}", resumo(vec![1, 2, 3]));
    println!("{:?}", resumo([10, 20]));
    println!("{:?}", resumo(BTreeSet::from([5, 5, 7])));

    println!(
        "{:?}",
        aplicar_em_todos(&["a-b", "c-d"], |s| s.split('-').next().unwrap())
    );

    let mut v = vec![10, 20, 30, 40];
    let it = indices_pares(&v);
    v.push(50);
    println!("{:?}", it.collect::<Vec<_>>());
    println!("{:?}", pares(&v).collect::<Vec<_>>());

    let mut s = String::from("oi");
    let rep = repetir(&s, 3);
    s.push('!');
    println!("{:?} {s}", rep.collect::<Vec<_>>());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn covariancia() {
        let fixos: Vec<&'static str> = vec!["a", "bbb"];
        assert_eq!(maior(&fixos), "bbb");
    }

    #[test]
    fn hrtb_com_varias_colecoes() {
        assert_eq!(resumo(vec![1, 2, 3]), (6, 3));
        assert_eq!(resumo([4]), (4, 1));
        assert_eq!(resumo(BTreeSet::from([1, 1, 2])), (3, 2));
    }

    #[test]
    fn closure_com_hrtb() {
        let f = com_hrtb(|s| s.trim());
        let dono = String::from("  x  ");
        assert_eq!(f(&dono), "x");
        assert_eq!(f(" y "), "y");
    }

    #[test]
    fn use_vazio_nao_prende_o_emprestimo() {
        let mut v = vec![1, 2, 3];
        let it = indices_pares(&v);
        v.clear();
        assert_eq!(it.collect::<Vec<_>>(), [0, 2]);
    }

    #[test]
    fn primeiro_ou_padrao() {
        assert_eq!(primeiro_ou(&[], "x"), "x");
    }
}

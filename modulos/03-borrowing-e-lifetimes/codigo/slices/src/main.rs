//! Exemplos da lição 03: slices como fat pointers, `&str`/`&[T]` em
//! parâmetros, `&mut [T]`, `split_at_mut` e `get_disjoint_mut`.

use std::mem::size_of;

/// Devolve a primeira palavra. Aceita `&String`, `&str` literal ou um pedaço.
fn first_word(s: &str) -> &str {
    match s.find(' ') {
        Some(i) => &s[..i],
        None => s,
    }
}

/// Soma os elementos de qualquer sequência contígua de `i32`.
fn sum(xs: &[i32]) -> i32 {
    xs.iter().sum()
}

/// Altera os elementos, mas não o tamanho: slice não tem `push`.
fn double_all(xs: &mut [i32]) {
    for x in xs.iter_mut() {
        *x *= 2;
    }
}

/// Soma a metade da direita na da esquerda, elemento a elemento.
fn fold_halves(xs: &mut [i32]) {
    let mid = xs.len() / 2;
    let (left, right) = xs.split_at_mut(mid);
    for (l, r) in left.iter_mut().zip(right.iter()) {
        *l += *r;
    }
}

/// Troca os elementos `i` e `j`, se forem posições válidas e diferentes.
fn swap_disjoint(xs: &mut [i32], i: usize, j: usize) -> bool {
    match xs.get_disjoint_mut([i, j]) {
        Ok([a, b]) => {
            std::mem::swap(a, b);
            true
        }
        Err(_) => false,
    }
}

fn main() {
    println!("String:    {} bytes", size_of::<String>());
    println!("&String:   {} bytes", size_of::<&String>());
    println!("&str:      {} bytes", size_of::<&str>());
    println!("&[i32]:    {} bytes", size_of::<&[i32]>());
    println!("&Vec<i32>: {} bytes", size_of::<&Vec<i32>>());

    let dono = String::from("olá mundo");
    println!("{}", first_word(&dono));
    println!("{}", first_word(&dono[5..]));
    println!("{}", first_word("borrow checker"));

    let v = vec![1, 2, 3];
    let arr = [10, 20, 30, 40];
    println!("{} {} {}", sum(&v), sum(&arr), sum(&arr[1..3]));

    let mut w = vec![1, 2, 3, 4];
    double_all(&mut w[1..3]);
    println!("{w:?}");

    let mut h = [1, 2, 3, 10, 20, 30];
    fold_halves(&mut h);
    println!("{h:?}");

    let mut s = [1, 2, 3, 4];
    println!("{} {s:?}", swap_disjoint(&mut s, 0, 3));
    println!("{} {s:?}", swap_disjoint(&mut s, 1, 1));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn first_word_points_into_original() {
        let s = String::from("olá mundo");
        let w = first_word(&s);
        assert_eq!(w, "olá");
        assert_eq!(w.as_ptr(), s.as_ptr());
    }

    #[test]
    fn halves() {
        let mut h = [1, 2, 10, 20];
        fold_halves(&mut h);
        assert_eq!(h, [11, 22, 10, 20]);
    }

    #[test]
    fn disjoint_rejects_same_index() {
        let mut s = [1, 2];
        assert!(!swap_disjoint(&mut s, 0, 0));
        assert!(!swap_disjoint(&mut s, 0, 5));
        assert!(swap_disjoint(&mut s, 0, 1));
        assert_eq!(s, [2, 1]);
    }
}

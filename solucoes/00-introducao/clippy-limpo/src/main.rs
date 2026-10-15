use std::f64::consts::PI;

// Correções, na ordem em que o clippy as aponta:
// 1. `3.1415` -> `PI` (erro `approx_constant`) e sem `mut` (`unused_mut`).
// 2. `vec![1, 2, 3]` -> array `[1, 2, 3]` (`useless_vec`).
// 3. `for i in 0..v.len()` + `v[i]` -> `for item in &v` (`needless_range_loop`).
// 4. `v.len() == 0` -> `v.is_empty()` (`len_zero`)...
// 5. ...mas, com um array de tamanho fixo, `is_empty()` é sempre falso
//    (`const_is_empty`). O `if` inteiro era código morto e foi removido.

fn main() {
    let x = PI;
    let v = [1, 2, 3];
    for item in &v {
        println!("{item}");
    }
    println!("{}", x * 2.0);
}

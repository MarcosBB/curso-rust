//! Exemplos das lições 01 e 02: referências, NLL, reborrow,
//! two-phase borrows e como evitar iterator invalidation.

/// Recebe uma referência compartilhada: lê sem tomar posse.
// `&String` de propósito, como na lição 01. A lição 03 mostra por que
// `&str` é melhor (e por que o clippy avisa com `ptr_arg`).
#[allow(clippy::ptr_arg)]
fn calculate_length(s: &String) -> usize {
    s.len()
}

/// Recebe uma referência exclusiva: pode alterar o valor do chamador.
fn add_suffix(s: &mut String) {
    s.push_str(", Esq.");
}

fn add_one(v: &mut Vec<i32>) {
    v.push(1);
}

/// Junta primeiro, aplica depois: nenhum empréstimo de `v` está vivo no `extend`.
fn append_tens_of_odds(v: &mut Vec<i32>) {
    let novos: Vec<i32> = v.iter().filter(|x| *x % 2 == 1).map(|x| x * 10).collect();
    v.extend(novos);
}

fn main() {
    // & e &mut
    let s1 = String::from("olá");
    let len = calculate_length(&s1);
    println!("{s1} tem {len} bytes");

    let mut name = String::from("Fulano");
    add_suffix(&mut name);
    println!("{name}");

    // NLL: o empréstimo termina no último uso de `first`
    let mut v = vec![1, 2, 3];
    let first = &v[0];
    println!("primeiro: {first}");
    v.push(4);
    println!("{v:?}");

    // reborrow: passar `r` duas vezes não consome a referência
    let mut w = Vec::new();
    let r = &mut w;
    add_one(r);
    add_one(r);
    r.push(2);
    println!("{w:?}");

    // two-phase borrow: `v.len()` é avaliado antes do `&mut v` do push
    let mut z = vec![10, 20];
    z.push(z.len() as i32);
    println!("{z:?}");

    // iterator invalidation evitada
    let mut nums = vec![1, 2, 3];
    append_tens_of_odds(&mut nums);
    println!("{nums:?}");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn suffix_is_appended() {
        let mut s = String::from("Ana");
        add_suffix(&mut s);
        assert_eq!(s, "Ana, Esq.");
    }

    #[test]
    fn tens_of_odds() {
        let mut v = vec![1, 2, 3];
        append_tens_of_odds(&mut v);
        assert_eq!(v, [1, 2, 3, 10, 30]);
    }
}

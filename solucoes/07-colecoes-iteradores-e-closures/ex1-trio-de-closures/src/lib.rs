// (a) Devolve uma closure que multiplica por `k`.
// Erro: E0373. Sem `move`, a closure guardaria `&k`, e `k` morre no fim
// da função. Com `move`, ela leva uma cópia de `k`.
// Trait: Fn (só lê o capturado).
pub fn multiplicador(k: i32) -> impl Fn(i32) -> i32 {
    move |x| x * k
}

// (b) Conta as vogais (sem acento) de um texto.
// Erro: E0596. A closure altera `total`, então chamá-la exige `&mut`
// nela mesma: `let mut conta`.
// Trait: FnMut (altera o capturado).
pub fn conta_vogais(texto: &str) -> usize {
    let mut total = 0;
    let mut conta = |c: char| {
        if "aeiou".contains(c) {
            total += 1;
        }
    };
    for c in texto.chars() {
        conta(c);
    }
    total
}

// (c) Cumprimenta duas vezes.
fn chamar_duas_vezes<F: Fn() -> String>(f: F) -> String {
    f() + &f()
}

// Erro: E0525. `let n = nome;` move `nome` para fora do corpo, e a
// closure vira só FnOnce, mas `chamar_duas_vezes` pede Fn. A correção é
// só ler `nome` (o `move` continua: a closure é dona de `nome`, mas não
// o entrega a ninguém).
// Trait: Fn (antes da correção, só FnOnce).
pub fn saudacao_dupla(nome: String) -> String {
    let saudar = move || format!("oi, {nome}! ");
    chamar_duas_vezes(saudar)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_multiplicador() {
        let triplo = multiplicador(3);
        assert_eq!(triplo(4), 12);
        assert_eq!(triplo(-1), -3);
    }

    #[test]
    fn b_vogais() {
        assert_eq!(conta_vogais("rust e legal"), 4);
        assert_eq!(conta_vogais("xyz"), 0);
    }

    #[test]
    fn c_saudacao() {
        assert_eq!(saudacao_dupla(String::from("Ana")), "oi, Ana! oi, Ana! ");
    }
}

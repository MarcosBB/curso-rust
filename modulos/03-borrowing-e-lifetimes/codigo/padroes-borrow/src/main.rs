//! Exemplos da lição 07: correções idiomáticas para erros de empréstimo.

use std::collections::HashMap;

/// "Get ou insere" com a Entry API: uma única busca, um único empréstimo.
fn get_or_default(map: &mut HashMap<u32, String>, k: u32) -> &mut String {
    map.entry(k).or_default()
}

/// Contagem de palavras: o caso de uso mais comum de `entry`.
fn word_count(text: &str) -> HashMap<&str, u32> {
    let mut counts = HashMap::new();
    for word in text.split_whitespace() {
        *counts.entry(word).or_insert(0) += 1;
    }
    counts
}

/// Copia o valor pequeno (`usize`) em vez de guardar a referência.
fn push_and_report(names: &mut Vec<String>, new: &str) -> usize {
    let largest_len = names.iter().map(|s| s.len()).max().unwrap_or(0);
    names.push(new.to_string());
    largest_len
}

/// Junta primeiro, aplica depois.
fn duplicate_expensive(prices: &mut Vec<u32>, limit: u32) {
    let expensive: Vec<u32> = prices.iter().copied().filter(|&p| p > limit).collect();
    prices.extend(expensive);
}

/// Índice em vez de referência: o índice não empresta nada.
fn bump_max(prices: &mut [u32]) {
    let idx = prices
        .iter()
        .enumerate()
        .max_by_key(|&(_, p)| *p)
        .map(|(i, _)| i);
    if let Some(i) = idx {
        prices[i] += 1;
    }
}

struct Player {
    name: String,
    score: u32,
}

impl Player {
    /// Devolve uma informação com posse em vez de emprestar `self` inteiro.
    fn label(&self) -> String {
        format!("{} ({})", self.name, self.score)
    }
}

fn main() {
    let mut m = HashMap::new();
    get_or_default(&mut m, 1).push_str("um");
    get_or_default(&mut m, 1).push('!');
    println!("{m:?}");

    let mut pairs: Vec<_> = word_count("a b a c b a").into_iter().collect();
    pairs.sort();
    println!("{pairs:?}");

    let mut names = vec![String::from("Ana"), String::from("Bartolomeu")];
    let len = push_and_report(&mut names, "Zé");
    println!("{len} {names:?}");

    let mut prices = vec![10, 50, 30];
    duplicate_expensive(&mut prices, 20);
    bump_max(&mut prices);
    println!("{prices:?}");

    // Emprestar campos, não o struct: `name` e `score` são lugares diferentes.
    let mut p = Player {
        name: String::from("Ana"),
        score: 0,
    };
    let name = &p.name;
    p.score += 10;
    println!("{name}: {}", p.score);
    println!("{}", p.label());

    // retain e take: APIs que fazem a mutação "por dentro".
    let mut v = vec![1, 2, 3, 4, 5, 6];
    v.retain(|x| x % 2 == 0);
    let mut slot = Some(String::from("token"));
    let taken = slot.take();
    println!("{v:?} {taken:?} {slot:?}");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn entry_inserts_once() {
        let mut m = HashMap::new();
        get_or_default(&mut m, 7).push('a');
        get_or_default(&mut m, 7).push('b');
        assert_eq!(m[&7], "ab");
        assert_eq!(m.len(), 1);
    }

    #[test]
    fn counts_words() {
        let c = word_count("x y x");
        assert_eq!(c["x"], 2);
        assert_eq!(c["y"], 1);
    }

    #[test]
    fn duplicates_and_bumps() {
        let mut p = vec![10, 50, 30];
        duplicate_expensive(&mut p, 20);
        assert_eq!(p, [10, 50, 30, 50, 30]);
        bump_max(&mut p);
        assert_eq!(p.iter().filter(|&&x| x == 51).count(), 1);
    }
}

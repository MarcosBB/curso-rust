use std::collections::{BTreeSet, HashMap};

/// `[x * x for x in xs if x % 2 == 0]`
pub fn quadrados_pares(xs: &[i32]) -> Vec<i32> {
    xs.iter().filter(|&&x| x % 2 == 0).map(|x| x * x).collect()
}

/// `{p: len(p) for p in palavras}` (tamanho em caracteres)
pub fn tamanhos(palavras: &[&str]) -> HashMap<String, usize> {
    palavras
        .iter()
        .map(|p| (p.to_string(), p.chars().count()))
        .collect()
}

/// `{c.lower() for c in texto if c.isalpha()}`
pub fn letras_unicas(texto: &str) -> BTreeSet<char> {
    texto
        .chars()
        .filter(|c| c.is_alphabetic())
        .flat_map(char::to_lowercase)
        .collect()
}

/// `[(i, j) for i in range(n) for j in range(i + 1, n)]`
pub fn pares(n: u32) -> Vec<(u32, u32)> {
    // `move`: a closure interna captura `i`, que pertence a uma chamada
    // da closure externa e morreria antes do iterador interno terminar.
    (0..n)
        .flat_map(|i| (i + 1..n).map(move |j| (i, j)))
        .collect()
}

/// `Counter(texto.lower().split()).most_common(3)`, com desempate
/// alfabético (o `HashMap` não tem ordem).
pub fn top3(texto: &str) -> Vec<(String, usize)> {
    let mut contagem: HashMap<String, usize> = HashMap::new();
    for p in texto.split_whitespace() {
        *contagem.entry(p.to_lowercase()).or_insert(0) += 1;
    }
    let mut pares: Vec<(String, usize)> = contagem.into_iter().collect();
    pares.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    pares.truncate(3);
    pares
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn t_quadrados_pares() {
        assert_eq!(quadrados_pares(&[1, 2, 3, 4, -6]), [4, 16, 36]);
        assert!(quadrados_pares(&[1, 3]).is_empty());
    }

    #[test]
    fn t_tamanhos() {
        let t = tamanhos(&["sol", "maçã", "sol"]);
        assert_eq!(t.len(), 2);
        assert_eq!(t["maçã"], 4);
        assert_eq!(t.get("sol"), Some(&3));
    }

    #[test]
    fn t_letras_unicas() {
        let l: String = letras_unicas("Ana, ÁGUA 42!").into_iter().collect();
        assert_eq!(l, "agnuá");
    }

    #[test]
    fn t_pares() {
        assert_eq!(pares(3), [(0, 1), (0, 2), (1, 2)]);
        assert_eq!(pares(5).len(), 10);
        assert!(pares(1).is_empty());
    }

    #[test]
    fn t_top3() {
        let texto = "o rato roeu a roupa do rei de Roma o Rato fugiu a";
        assert_eq!(
            top3(texto),
            [
                ("a".to_string(), 2),
                ("o".to_string(), 2),
                ("rato".to_string(), 2)
            ]
        );
        assert_eq!(top3("um"), [("um".to_string(), 1)]);
    }
}

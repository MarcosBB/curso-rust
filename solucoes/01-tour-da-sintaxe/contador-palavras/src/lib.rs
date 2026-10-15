pub mod texto;

use std::collections::HashMap;

/// Conta quantas vezes cada palavra aparece, sem diferenciar maiúsculas
/// e ignorando a pontuação nas pontas.
pub fn contar(texto: &str) -> HashMap<String, usize> {
    let mut contagem = HashMap::new();
    for palavra in texto::palavras(texto) {
        *contagem.entry(palavra).or_insert(0) += 1;
    }
    contagem
}

/// Os pares (palavra, quantidade) em ordem alfabética.
pub fn em_ordem(contagem: &HashMap<String, usize>) -> Vec<(String, usize)> {
    let mut pares = Vec::new();
    for (palavra, quantidade) in contagem {
        pares.push((palavra.clone(), *quantidade));
    }
    pares.sort();
    pares
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn conta_repetidas() {
        let c = contar("a b a");
        assert_eq!(c.len(), 2);
        assert_eq!(c.get("a"), Some(&2));
        assert_eq!(c.get("b"), Some(&1));
        assert_eq!(c.get("z"), None);
    }

    #[test]
    fn ignora_maiusculas_e_pontuacao() {
        let c = contar("Oi, oi! OI?");
        assert_eq!(c.len(), 1, "deveria virar uma palavra só: {c:?}");
        assert_eq!(c["oi"], 3);
    }

    #[test]
    fn texto_vazio() {
        assert!(contar("").is_empty());
        assert!(contar("  ... --- ").is_empty());
    }

    #[test]
    fn ordem_alfabetica() {
        let c = contar("b a b");
        assert_eq!(
            em_ordem(&c),
            vec![(String::from("a"), 1), (String::from("b"), 2)]
        );
    }
}
